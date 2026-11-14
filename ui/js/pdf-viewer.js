// The Summary tab's page view: the report as the PDF that will be saved, with
// zoom, page navigation and lazy rendering.
//
// It deliberately has no imports: the same file is loaded by the app and by the
// standalone check that renders it in a plain WebKitGTK view.

import {
  backingScale,
  clampZoom,
  fitPage,
  fitWidth,
  keptPages,
  pageLabel,
  stepZoom,
} from "./pdf-math.js";

const PDF_JS = "vendor/pdfjs/pdf.min.js";
const PDF_WORKER = "vendor/pdfjs/pdf.worker.min.js";

let library = null;

/** Load the viewer library once, from the app's own bundle: no network. */
async function pdfjs() {
  if (window.pdfjsLib) return window.pdfjsLib;
  if (!library) {
    library = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = new URL(PDF_JS, document.baseURI).href;
      script.addEventListener("load", () => resolve(window.pdfjsLib));
      script.addEventListener("error", () =>
        reject(new Error("the PDF viewer could not be loaded")),
      );
      document.head.appendChild(script);
    });
  }
  const loaded = await library;
  loaded.GlobalWorkerOptions.workerSrc = new URL(PDF_WORKER, document.baseURI).href;
  return loaded;
}

/**
 * Open the bytes. pdf.js prefers its worker; where the app's own protocol will
 * not start one, the worker script is loaded into this thread and pdf.js takes
 * its main-thread path. Either way the failure is reported, never hidden.
 */
async function openDocument(lib, bytes) {
  const options = {
    data: bytes,
    // pdf.js 3.11 predates CVE-2024-4367's fix; this is the mitigation it
    // documents, and the fonts in a plan are always embedded.
    isEvalSupported: false,
    useSystemFonts: true,
  };
  try {
    return await lib.getDocument(options).promise;
  } catch (first) {
    try {
      await loadWorkerOnMainThread();
      return await lib.getDocument(options).promise;
    } catch (second) {
      throw new Error(`${first.message || first} (${second.message || second})`);
    }
  }
}

let workerScript = null;
function loadWorkerOnMainThread() {
  if (window.pdfjsWorker) return Promise.resolve();
  if (!workerScript) {
    workerScript = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = new URL(PDF_WORKER, document.baseURI).href;
      script.addEventListener("load", () => resolve());
      script.addEventListener("error", () =>
        reject(new Error("the PDF worker could not be loaded")),
      );
      document.head.appendChild(script);
    });
  }
  return workerScript;
}

/**
 * Build the page view.
 *
 * `onChange` receives `{ pages, zoom, label, busy }` whenever the view's state
 * changes, so the screen can draw its own controls.
 */
export function createPdfViewer({ onChange } = {}) {
  const pages = document.createElement("div");
  pages.className = "pdf-pages";
  const root = document.createElement("div");
  root.className = "pdf-view";
  root.tabIndex = 0;
  root.appendChild(pages);

  let lib = null;
  let doc = null;
  let token = 0;
  // Which document the canvases belong to. A render that was already running
  // when the document was replaced must not record itself against the new one.
  let documentToken = 0;
  let zoom = 1;
  let fit = "width";
  let current = 0;
  let entries = [];
  let rendered = new Map();
  let tasks = new Map();
  let observer = null;

  const notify = (busy = false) =>
    onChange?.({
      pages: doc ? doc.numPages : 0,
      zoom,
      label: pageLabel(current + 1, doc ? doc.numPages : 0),
      busy,
    });

  function clearCanvas(index) {
    cancel(index);
    const entry = entries[index];
    if (!entry) return;
    entry.canvas.width = 0;
    entry.canvas.height = 0;
    entry.canvas.style.width = "0";
    entry.canvas.style.height = "0";
    rendered.delete(index);
  }

  function cancel(index) {
    const task = tasks.get(index);
    if (task) {
      try {
        task.cancel();
      } catch (ignored) {
        /* already finished */
      }
      tasks.delete(index);
    }
  }

  function layout() {
    for (const entry of entries) {
      entry.frame.style.width = `${entry.width * zoom}px`;
      entry.frame.style.height = `${entry.height * zoom}px`;
    }
  }

  async function render(index) {
    if (!doc || rendered.get(index) === zoom) return;
    const entry = entries[index];
    if (!entry) return;
    const mine = documentToken;
    cancel(index);
    const page = await doc.getPage(index + 1);
    if (mine !== documentToken) return;
    if (rendered.get(index) === zoom) return;
    const scale = backingScale(zoom, window.devicePixelRatio);
    const viewport = page.getViewport({ scale });
    const canvas = entry.canvas;
    canvas.width = Math.max(1, Math.round(viewport.width));
    canvas.height = Math.max(1, Math.round(viewport.height));
    canvas.style.width = `${entry.width * zoom}px`;
    canvas.style.height = `${entry.height * zoom}px`;
    const context = canvas.getContext("2d", { alpha: false });
    const task = page.render({ canvasContext: context, viewport });
    tasks.set(index, task);
    try {
      await task.promise;
      // The page may have been evicted while it was drawing; a zeroed canvas is
      // not a rendered page, so it is not remembered as one.
      if (canvas.width > 0 && canvas.height > 0 && mine === documentToken) {
        rendered.set(index, zoom);
      }
    } catch (error) {
      if (error?.name !== "RenderingCancelledException") throw error;
    } finally {
      if (tasks.get(index) === task) tasks.delete(index);
    }
  }

  function observe() {
    observer?.disconnect();
    if (!doc) return;
    observer = new IntersectionObserver(
      (records) => {
        for (const record of records) {
          const index = Number(record.target.dataset.page) - 1;
          if (record.isIntersecting) {
            render(index).catch(() => {});
          } else {
            clearCanvas(index);
          }
        }
        const visible = entries.findIndex((entry) => {
          const box = entry.frame.getBoundingClientRect();
          return box.bottom > 0 && box.top < window.innerHeight;
        });
        if (visible >= 0 && visible !== current) {
          current = visible;
          notify();
        }
      },
      { root, rootMargin: "600px 0px" },
    );
    for (const entry of entries) observer.observe(entry.frame);
  }

  function evictOutside(radius = 2) {
    const keep = keptPages(current, entries.length, radius);
    for (const index of rendered.keys()) {
      if (!keep.has(index) && !entries[index].frame.matches(":hover")) cancel(index);
    }
  }

  async function show(bytes) {
    const mine = ++token;
    notify(true);
    let next;
    try {
      lib ??= await pdfjs();
      next = await openDocument(lib, bytes);
    } catch (error) {
      notify(false);
      throw new Error(error.message || String(error));
    }
    if (mine !== token) {
      next.destroy();
      return;
    }
    if (doc) {
      try {
        doc.destroy();
      } catch (ignored) {
        /* the previous document is already gone */
      }
    }
    doc = next;
    documentToken += 1;

    pages.replaceChildren();
    rendered = new Map();
    tasks = new Map();
    current = 0;
    entries = [];
    for (let index = 0; index < doc.numPages; index += 1) {
      const frame = document.createElement("div");
      frame.className = "pdf-page";
      frame.dataset.page = String(index + 1);
      const canvas = document.createElement("canvas");
      frame.appendChild(canvas);
      pages.appendChild(frame);
      entries.push({ frame, canvas, width: 0, height: 0 });
    }

    // Page sizes come from the document, so the placeholders are the real A4
    // boxes before anything is drawn.
    await Promise.all(
      entries.map(async (entry, index) => {
        const page = await doc.getPage(index + 1);
        const box = page.getViewport({ scale: 1 });
        entry.width = box.width;
        entry.height = box.height;
        entry.frame.style.width = `${box.width}px`;
        entry.frame.style.height = `${box.height}px`;
      }),
    );
    if (mine !== token) return;

    fit = "width";
    zoom = fitWidth(root.clientWidth, entries[0]?.width || 595);
    layout();
    observe();
    notify();
  }

  function applyZoom(next, nextFit = "custom") {
    zoom = clampZoom(next);
    fit = nextFit;
    layout();
    for (const index of [...rendered.keys()]) {
      rendered.delete(index);
      cancel(index);
    }
    if (doc) {
      for (const entry of entries) observer?.unobserve(entry.frame);
      observe();
    }
    notify();
  }

  function setZoom(next) {
    applyZoom(next);
  }

  function zoomBy(direction) {
    applyZoom(stepZoom(zoom, direction));
  }

  function setFit(mode) {
    if (!doc) return;
    const width = root.clientWidth;
    const height = root.clientHeight;
    const page = entries[current] || entries[0];
    if (!page) return;
    applyZoom(
      mode === "page"
        ? fitPage(width, height, page)
        : fitWidth(width, page.width),
      mode,
    );
  }

  /** Re-fit after the window or pane changes size, without losing the zoom the
   * operator chose. */
  function refit() {
    if (doc && fit === "width") applyZoom(fitWidth(root.clientWidth, entries[0]?.width || 595), "width");
    else if (doc && fit === "page") setFit("page");
  }

  const onScroll = () => evictOutside();
  root.addEventListener("scroll", onScroll, { passive: true });

  return {
    element: root,
    show,
    setZoom,
    zoomBy,
    setFit,
    refit,
    get zoom() {
      return zoom;
    },
    get pages() {
      return doc ? doc.numPages : 0;
    },
    destroy() {
      token += 1;
      observer?.disconnect();
      for (const index of [...tasks.keys()]) cancel(index);
      doc?.destroy();
      doc = null;
      root.removeEventListener("scroll", onScroll);
    },
  };
}
