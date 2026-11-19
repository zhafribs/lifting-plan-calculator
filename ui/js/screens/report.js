// Summary: the printable calculation report, typeset with the bundled KaTeX.

import { el, svgIcon, debounce, escapeHtml } from "../dom.js";
import { api, errorText } from "../api.js";
import { card, field, textInput } from "../components.js";
import { captureWorkingDiagram } from "../graph/snapshot.js";
import { createPdfViewer } from "../pdf-viewer.js";

export function createReportScreen(ctx) {
  const { store, catalog, toast } = ctx;

  // A new plan starts with the crane check alone: that is the section every plan
  // needs. The rest are opt-in, so nothing unticked can look like an omission.
  const sections = { crane: true, outrigger: false, uniform: false, nonuniform: false, tandem: false };
  let lastHtml = "";
  let lastSavedPath = null;
  const DEFAULT_FOOTER =
    "Prepared with Lifting Plan Calculator by Zhafri Syazwi — zhafribs@gmail.com";
  let footerText = DEFAULT_FOOTER;
  // The issue stamp belongs to the session, not to each keystroke.
  const now = new Date();
  const issuedAt = [
    String(now.getDate()).padStart(2, "0"),
    String(now.getMonth() + 1).padStart(2, "0"),
    now.getFullYear(),
  ].join("-");
  // Sign-off lines live here, not in the lift state: they are printed on the
  // plan and never reach the engine.
  const signoff = { reference: "", prepared_by: "", checked_by: "", approved_by: "" };

  const planFields = [
    ["reference", "Plan reference"],
    ["prepared_by", "Prepared by"],
    ["checked_by", "Checked by"],
    ["approved_by", "Approved by"],
  ];

  const checkBoxes = {};
  const checkNodes = {
    crane: ["Crane", "Capacity check against the load chart"],
    outrigger: ["Outrigger Pad Size", "Bearing area from the BS 8004 ground table"],
    uniform: ["Uniform Load", "The selected sling arrangement"],
    nonuniform: ["Nonuniform Load", "The asymmetric bridle"],
    tandem: ["Tandem", "The two-crane lift"],
  };
  for (const [key, [title, desc]] of Object.entries(checkNodes)) {
    const input = el("input", { type: "checkbox" });
    input.checked = sections[key];
    input.addEventListener("change", () => {
      sections[key] = input.checked;
      schedulePreview(0);
      schedulePdf(0);
    });
    checkBoxes[key] = input;
  }

  const diagramInput = el("input", { type: "checkbox" });
  diagramInput.addEventListener("change", () => {
    if (!diagramInput.checked) {
      schedulePreview(0);
      schedulePdf(0);
      return;
    }
    const url = captureWorkingDiagram();
    if (!url) {
      diagramInput.checked = false;
      toast(
        "The working range diagram could not be drawn; check the Graph tab inputs.",
        "error",
      );
      return;
    }
    schedulePreview(0);
    schedulePdf(0);
  });
  const diagramRow = el(
    "label",
    { class: "check" },
    diagramInput,
    el(
      "span",
      {},
      el("div", { class: "t", text: "Working range diagram" }),
      el("div", { class: "d", text: "The current crane and jib configuration" }),
    ),
  );

  // The preview runs under the app's Content-Security-Policy: scripts only from
  // the app's own origin, no inline script and no inline handler. A `sandbox`
  // attribute was tried and rejected: an opaque origin makes `script-src 'self'`
  // match nothing, so the report's own KaTeX pass would not run.
  const frame = el("iframe", { title: "Report preview" });
  const frameWrap = el("div", { class: "report-frame" }, frame);

  // The frame reports its KaTeX pass by message: an opaque-origin document
  // cannot be read from here.
  window.addEventListener("message", (event) => {
    const data = event.data;
    if (!data || (data.lpcReport !== "ready" && data.lpcReport !== "error")) return;
    window.uilog &&
      window.uilog(
        data.lpcReport === "error"
          ? `report katex error: ${data.error}`
          : `report preview ready: ${data.equations} typeset equations`,
      );
  });

  // The page view: the report as the PDF that will be saved, so the operator
  // sees the real page breaks and can zoom into a figure. It is rendered on
  // demand — opening the view, a section change, or leaving a field — never on
  // every keystroke, because each render is a full print.
  let previewMode = "pages";
  let pdfBusy = false;
  let lastPdfHtml = "";
  let pdfStale = false;

  const pdfPageLabel = el("span", { class: "faint pdf-page-label", text: "No pages" });
  const pdfStatus = el("span", { class: "faint pdf-status" });
  const viewer = createPdfViewer({
    onChange: ({ pages, label, zoom, busy }) => {
      pdfPageLabel.textContent = label;
      pdfStatus.textContent = busy || pdfBusy ? "Rendering…" : "";
      zoomLabel.textContent = `${Math.round(zoom * 100)}%`;
      for (const control of zoomControls) control.disabled = pages === 0;
    },
  });
  const pdfWrap = el("div", { class: "report-pdf" }, viewer.element);

  const zoomLabel = el("span", { class: "zoom-label", text: "100%" });
  const zoomControls = [];
  function zoomButton(label, title, onClick) {
    const button = el("button", { class: "btn small", title, onClick });
    button.textContent = label;
    zoomControls.push(button);
    return button;
  }
  const zoomOut = zoomButton("\u2212", "Zoom out", () => viewer.zoomBy(-1));
  const zoomIn = zoomButton("+", "Zoom in", () => viewer.zoomBy(1));
  const fitButton = zoomButton("Fit width", "Fit the page to the pane", () => viewer.setFit("width"));
  const refreshButton = zoomButton("Refresh", "Redraw from the current inputs", () => {
    if (previewMode === "pages") refreshPdf({ force: true });
  });
  // Native pinch and Ctrl+wheel mean zoom, not browser zoom.
  viewer.element.addEventListener(
    "wheel",
    (event) => {
      if (!event.ctrlKey) return;
      event.preventDefault();
      viewer.zoomBy(event.deltaY > 0 ? -1 : 1);
    },
    { passive: false },
  );

  const modeButtons = {};
  function previewButton(mode, label, title) {
    const button = el("button", { class: "seg", title, onClick: () => setPreviewMode(mode) });
    button.textContent = label;
    modeButtons[mode] = button;
    return button;
  }
  const pagesButton = previewButton("pages", "Pages", "The report as the PDF that will be saved");
  const liveButton = previewButton("live", "Live", "The report as it is being typed");

  // --- Page margins -------------------------------------------------------
  // Millimetres, session-only like the plan itself: the print, the footer and
  // the exported HTML all take these four numbers, so the page cannot be
  // printed with one margin and labelled with another.
  let margins = { top: 25.4, right: 25.4, bottom: 25.4, left: 25.4 };
  const MARGIN_MIN = 5;
  const MARGIN_MAX = 50;
  const marginInputs = {};
  const marginHint = el("span", { class: "margin-hint" });
  const marginError = el("p", { class: "margin-error", role: "alert", hidden: true });

  const marginSides = [
    ["top", "Top"],
    ["right", "Right"],
    ["bottom", "Bottom"],
    ["left", "Left"],
  ];
  const marginGrid = el(
    "div",
    { class: "margin-grid" },
    ...marginSides.map(([key, label]) => {
      const input = el("input", {
        type: "number",
        min: String(MARGIN_MIN),
        max: String(MARGIN_MAX),
        step: "1",
        value: String(margins[key]),
      });
      input.addEventListener("input", updateMarginHint);
      marginInputs[key] = input;
      return field({ label, unit: "mm", input });
    }),
  );
  const applyMargins = el(
    "button",
    { class: "btn primary small", onClick: () => applyPageMargins() },
    "Apply",
  );
  function presetMargins(mm) {
    for (const [key] of marginSides) marginInputs[key].value = String(mm);
    updateMarginHint();
  }
  const marginPanel = el(
    "div",
    {
      class: "margin-panel",
      role: "dialog",
      "aria-label": "Page margins",
      hidden: true,
    },
    el("h4", { text: "Page margins" }),
    marginGrid,
    el(
      "div",
      { class: "margin-presets" },
      el("button", { class: "btn small", onClick: () => presetMargins(25.4) }, "1 inch"),
      el("button", { class: "btn small", onClick: () => presetMargins(20) }, "20 mm"),
      el("button", { class: "btn small", onClick: () => presetMargins(12.7) }, "0.5 inch"),
      el("button", { class: "btn small", onClick: () => presetMargins(10) }, "10 mm"),
    ),
    marginHint,
    marginError,
    el(
      "div",
      { class: "margin-actions" },
      el("button", { class: "btn small", onClick: () => closeMargins() }, "Cancel"),
      applyMargins,
    ),
  );
  const marginButton = el(
    "button",
    {
      class: "btn small",
      title: "The printed page's margins, in millimetres",
      "aria-haspopup": "dialog",
      "aria-expanded": "false",
      onClick: () => toggleMargins(),
    },
    "Margins",
  );
  const marginControl = el("div", { class: "margin-control" }, marginButton, marginPanel);

  function readMargins() {
    const next = {};
    for (const [key] of marginSides) next[key] = Number(marginInputs[key].value);
    return next;
  }

  /** Why these four numbers cannot be printed, or "" when they can. */
  function marginProblem(next) {
    for (const [key, label] of marginSides) {
      const value = next[key];
      if (!Number.isFinite(value)) return `Enter a number for ${label.toLowerCase()}.`;
      if (value < MARGIN_MIN || value > MARGIN_MAX) {
        return `Each margin must be between ${MARGIN_MIN} and ${MARGIN_MAX} mm.`;
      }
    }
    const width = 210 - next.left - next.right;
    const height = 297 - next.top - next.bottom;
    if (width < 60 || height < 60) {
      return "That leaves too little of the sheet. Keep at least 60 mm of text width and height.";
    }
    return "";
  }

  function updateMarginHint() {
    const next = readMargins();
    const problem = marginProblem(next);
    const width = Math.max(0, 210 - next.left - next.right);
    const height = Math.max(0, 297 - next.top - next.bottom);
    marginHint.textContent = `Text area: ${width.toFixed(0)} × ${height.toFixed(0)} mm on A4`;
    marginError.textContent = problem;
    marginError.hidden = problem === "";
    applyMargins.disabled = problem !== "";
  }

  function toggleMargins() {
    if (marginPanel.hidden) {
      for (const [key] of marginSides) marginInputs[key].value = String(margins[key]);
      updateMarginHint();
      marginPanel.hidden = false;
      marginButton.setAttribute("aria-expanded", "true");
      marginInputs.top.focus();
      marginInputs.top.select();
    } else {
      closeMargins();
    }
  }

  function closeMargins() {
    if (marginPanel.hidden) return;
    marginPanel.hidden = true;
    marginButton.setAttribute("aria-expanded", "false");
  }

  async function applyPageMargins() {
    const next = readMargins();
    if (marginProblem(next)) return;
    margins = next;
    closeMargins();
    // The page view carries the margin in its cache key; a forced refresh makes
    // the change visible without waiting for the next keystroke.
    await rebuild();
    if (previewMode === "pages") refreshPdf({ force: true });
  }

  document.addEventListener("mousedown", (event) => {
    if (!marginPanel.hidden && !marginControl.contains(event.target)) closeMargins();
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !marginPanel.hidden) {
      closeMargins();
      marginButton.focus();
    }
  });

  const previewBar = el(
    "div",
    { class: "preview-bar" },
    el("div", { class: "segmented" }, pagesButton, liveButton),
    el("div", { class: "preview-tools" }, zoomOut, zoomLabel, zoomIn, fitButton),
    pdfPageLabel,
    pdfStatus,
    el("span", { class: "spacer" }),
    marginControl,
    refreshButton,
  );

  // The live frame carries the report at its real print size: 7 pt on a page of
  // the chosen margins. Unscaled in a column it is unreadable, so the body is
  // laid out at the print text width and zoomed to fill the pane — the same
  // width the page view shows.
  function fitLive() {
    if (frameWrap.hidden) return;
    const doc = frame.contentDocument;
    if (!doc || !doc.body) return;
    const textWidthPx = ((210 - margins.left - margins.right) * 96) / 25.4;
    const scale = Math.min(2, Math.max(0.35, frameWrap.clientWidth / textWidthPx));
    doc.body.style.width = `${textWidthPx}px`;
    doc.body.style.margin = "0 auto";
    doc.body.style.zoom = String(scale);
  }

  frame.addEventListener("load", fitLive);
  new ResizeObserver(fitLive).observe(frameWrap);

  const previewPane = el("div", { class: "preview-pane" }, previewBar, frameWrap, pdfWrap);
  frameWrap.hidden = true;
  previewPane.classList.add("is-pages");
  modeButtons.pages.classList.add("on");

  window.addEventListener(
    "resize",
    debounce(() => {
      if (previewMode === "pages") viewer.refit();
    }, 150),
  );

  const saveButton = el(
    "button",
    {
      class: "btn primary wide",
      onClick: async () => {
        try {
          const html = await ensureHtml();
          const path = await api.saveReport(html, "lifting-plan.html");
          if (path) {
            lastSavedPath = path;
            toast(`Report saved to ${path}`, "ok");
          }
        } catch (error) {
          toast(errorText(error), "error");
        }
      },
    },
    svgIcon("file", 15),
    "Save as HTML...",
  );

  const printButton = el(
    "button",
    {
      class: "btn wide",
      onClick: async () => {
        try {
          await ensureHtml();
          frame.contentWindow.focus();
          frame.contentWindow.print();
        } catch (error) {
          toast("Printing is not available here — save the file and print it from a browser instead.", "error");
        }
      },
    },
    svgIcon("print", 15),
    "Print",
  );

  const pdfButton = el(
    "button",
    {
      class: "btn wide",
      onClick: async () => {
        try {
          const html = await ensureHtml();
          // The footer is drawn onto the PDF's pages after printing, so the
          // export needs the same reference and credit the preview showed.
          const path = await api.savePdf(
            html,
            "lifting-plan.pdf",
            {
              reference: signoff.reference,
              center: footerText,
            },
            margins,
          );
          if (path) {
            toast(`PDF saved to ${path}`, "ok");
          }
        } catch (error) {
          toast(errorText(error), "error");
        }
      },
    },
    svgIcon("file", 15),
    "Save as PDF...",
  );

  const openButton = el(
    "button",
    {
      class: "btn wide",
      onClick: async () => {
        try {
          let path = lastSavedPath;
          if (!path) {
            const html = await ensureHtml();
            path = await api.saveReport(html, "lifting-plan.html");
            if (path) lastSavedPath = path;
          }
          if (path) await api.openReportFile(path);
        } catch (error) {
          toast(errorText(error), "error");
        }
      },
    },
    svgIcon("external", 15),
    "Open in browser",
  );

  const root = el(
    "div",
    { class: "page" },
    el(
      "div",
      { class: "report-layout" },
      el(
        "div",
        { class: "stack sticky pane" },
        card({
          title: "Report sections",
          body: el(
            "div",
            { class: "stack" },
            ...Object.entries(checkNodes).flatMap(([key, [title, desc]]) => {
              const row = el("label", { class: "check" }, checkBoxes[key], el("span", {}, el("div", { class: "t", text: title }), el("div", { class: "d", text: desc })));
              return key === "crane" ? [row, diagramRow] : [row];
            }),
          ),
          foot: el("span", { class: "faint", text: "The preview follows the other tabs live." }),
        }),
        card({
          title: "Plan details",
          body: el(
            "div",
            { class: "stack" },
            ...planFields.map(([key, label]) => {
              const input = textInput({
                value: signoff[key],
                placeholder: key === "reference" ? "LP-YYYY-NNN" : "",
                onInput: (value) => {
                  signoff[key] = value;
                  schedulePreview(0);
                },
              });
              // The page view is a print, so it redraws when the field is
              // finished with, not on every keystroke.
              input.addEventListener("blur", () => schedulePdf(0));
              return field({ label, input });
            }),
          ),
          foot: el("span", { class: "faint", text: `Issued ${issuedAt}` }),
        }),
        card({
          title: "Footer",
          body: el(
            "div",
            { class: "stack" },
            field({
              label: "Centre text",
              input: (() => {
                const input = textInput({
                  value: footerText,
                  onInput: (value) => {
                    footerText = value;
                    schedulePreview(0);
                  },
                });
                input.addEventListener("blur", () => schedulePdf(0));
                return input;
              })(),
            }),
          ),
          foot: el("span", { class: "faint", text: "Printed at the centre of every page." }),
        }),
        card({
          title: "Export",
          body: el(
            "div",
            { class: "stack" },
            el("p", {
              class: "faint small",
              text: "Working-range diagram: use Use in report in the Graph tab.",
            }),
            saveButton,
            pdfButton,
            printButton,
            openButton,
          ),
          foot: el("span", { class: "faint", text: "Save as PDF writes A4 with 1-inch margins, with the footer on every page. Saved HTML keeps a copy of the KaTeX assets beside it; when printing it, keep Margins on Default (1 inch)." }),
        }),
      ),
      previewPane,
    ),
  );

  const schedulePreview = debounce(() => {
    // Typing refreshes the live frame at once; the page view follows a moment
    // after the typing stops, so it catches up on its own.
    rebuild().then(() => {
      if (previewMode === "pages") schedulePdf();
    });
  }, 250);

  async function buildNow() {
    const include = Object.entries(sections)
      .filter(([, on]) => on)
      .map(([key]) => key);
    try {
      const diagram = diagramInput.checked ? captureWorkingDiagram() : null;
      if (diagramInput.checked && !diagram) diagramInput.checked = false;
      const html = await api.buildReport(
        store.state,
        include,
        issuedAt,
        signoff,
        diagram,
        footerText,
        margins,
      );
      lastHtml = html;
      // srcdoc pages do not resolve the wrapper's relative asset paths, so the
      // stylesheet and scripts are pinned against the app's own base URL.
      const katexBase = new URL("vendor/katex/", document.baseURI).href;
      frame.srcdoc = html.split('"vendor/katex/').join(`"${katexBase}`);
    } catch (error) {
      frame.srcdoc = `<p style="font-family: sans-serif; color: #9b1c1c; padding: 16px">${escapeHtml(errorText(error))}</p>`;
    }
  }

  let building = null;

  // One build at a time: a reader who leaves a field while the report is still
  // being rebuilt waits for that rebuild rather than redrawing the old one.
  function rebuild() {
    if (!building) {
      building = buildNow().finally(() => {
        building = null;
      });
    }
    return building;
  }

  async function ensureHtml() {
    if (building) await building;
    if (!lastHtml) await rebuild();
    return lastHtml;
  }

  function footerParams() {
    return { reference: signoff.reference, center: footerText };
  }

  function setPreviewMode(mode) {
    if (mode === previewMode) return;
    previewMode = mode;
    for (const [key, button] of Object.entries(modeButtons)) {
      button.classList.toggle("on", key === mode);
    }
    frameWrap.hidden = mode !== "live";
    pdfWrap.hidden = mode !== "pages";
    if (mode === "live") fitLive();
    previewPane.classList.toggle("is-pages", mode === "pages");
    if (mode === "pages") refreshPdf();
  }

  async function refreshPdf({ force = false } = {}) {
    if (previewMode !== "pages") return;
    if (pdfBusy) {
      pdfStale = true;
      return;
    }
    pdfBusy = true;
    pdfStatus.textContent = "Rendering…";
    // A first render opens at fit-width, so the view keeps tracking the pane.
    // Only a re-render puts the reader back where they were.
    const hadPages = viewer.pages > 0;
    const zoomBefore = hadPages ? viewer.zoom : 0;
    const scrollBefore = hadPages ? viewer.element.scrollTop : 0;
    try {
      const html = await ensureHtml();
      if (!force && html === lastPdfHtml) return;
      const bytes = await api.previewPdf(html, footerParams(), margins);
      // Tauri returns the raw response as an ArrayBuffer; accept a typed array
      // as well so the viewer does not depend on that detail.
      const data = bytes instanceof ArrayBuffer || ArrayBuffer.isView(bytes)
        ? new Uint8Array(bytes.buffer ?? bytes, bytes.byteOffset ?? 0, bytes.byteLength ?? undefined)
        : new Uint8Array(bytes);
      await viewer.show(data);
      lastPdfHtml = html;
      // Keep the reader where they were: same zoom, same scroll.
      if (zoomBefore > 0 && Math.abs(zoomBefore - viewer.zoom) > 0.001) {
        viewer.setZoom(zoomBefore);
      }
      if (scrollBefore > 0) viewer.element.scrollTop = scrollBefore;
    } catch (error) {
      const message = errorText(error);
      // A page view needs the print path. When it is unavailable, say why and
      // fall back to the live preview rather than showing an empty pane.
      if (!/superseded/i.test(message)) {
        toast(`Page view unavailable: ${message}`, "error");
        setPreviewMode("live");
      }
    } finally {
      pdfBusy = false;
      pdfStatus.textContent = "";
      if (pdfStale) {
        pdfStale = false;
        schedulePdf();
      }
    }
  }

  // A page view costs a print, so it waits for a pause in the typing and never
  // queues a second render behind a running one.
  const schedulePdf = debounce(() => {
    if (previewMode === "pages") refreshPdf();
  }, 900);

  function setSections(list) {
    for (const key of Object.keys(sections)) sections[key] = list.includes(key);
    for (const [key, input] of Object.entries(checkBoxes)) input.checked = sections[key];
    schedulePreview(0);
  }

  function sync() {
    // The preview depends on the whole state, so it is refreshed on every
    // publish; nothing else here is stateful.
    schedulePreview();
  }

  // The page view costs a print: it is drawn when the tab is opened, not at
  // launch and not on every keystroke.
  function show() {
    if (previewMode === "pages") refreshPdf();
  }

  return { root, sync, show, render: () => {}, setSections };
}
