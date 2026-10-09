// Summary: the printable calculation report, typeset with the bundled KaTeX.

import { el, svgIcon, debounce } from "../dom.js";
import { api, errorText } from "../api.js";
import { card } from "../components.js";

export function createReportScreen(ctx) {
  const { store, catalog, toast } = ctx;

  const sections = { crane: true, uniform: true, nonuniform: true, tandem: true };
  let lastHtml = "";
  let lastSavedPath = null;

  const checkBoxes = {};
  const checkNodes = {
    crane: ["Crane", "Capacity check against the load chart"],
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
    });
    checkBoxes[key] = input;
  }

  const frame = el("iframe", { title: "Report preview" });
  const frameWrap = el("div", { class: "report-frame" }, frame);

  // One scrollable: stacked (the same max-width the layout stacks at), the
  // Summary tab reads as a single document — the frame opens to the report's
  // full height and the screen owns the only scrollbar. Side by side the
  // frame keeps its viewport-sized box and its own scroll, which is the one
  // scrollable there.
  const stackedQuery = window.matchMedia("(max-width: 1000px)");
  function fitFrame() {
    if (!stackedQuery.matches) {
      frameWrap.style.height = "";
      return;
    }
    try {
      const doc = frame.contentDocument;
      if (!doc || !doc.documentElement) return;
      const content = Math.max(
        doc.documentElement.scrollHeight,
        doc.body ? doc.body.scrollHeight : 0,
      );
      if (content > 0) {
        // Border-box: the wrapper's height includes its 1px top and bottom
        // border, which the iframe (height: 100%) does not get to use.
        frameWrap.style.height = `${Math.ceil(content) + 2}px`;
      }
    } catch (error) {
      /* unreadable until the next load or resize — retried then */
    }
  }
  window.addEventListener("resize", debounce(() => fitFrame(), 150));

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
        { class: "stack sticky" },
        card({
          title: "Report sections",
          body: el(
            "div",
            { class: "stack" },
            ...Object.entries(checkNodes).map(([key, [title, desc]]) => {
              const row = el("label", { class: "check" }, checkBoxes[key], el("span", {}, el("div", { class: "t", text: title }), el("div", { class: "d", text: desc })));
              return row;
            }),
          ),
          foot: el("span", { class: "faint", text: "The preview follows the other tabs live." }),
        }),
        card({
          title: "Export",
          body: el("div", { class: "stack" }, saveButton, printButton, openButton),
          foot: el("span", { class: "faint", text: "Saved HTML keeps a copy of the KaTeX assets beside it, so it prints with the equations typeset." }),
        }),
      ),
      frameWrap,
    ),
  );

  const schedulePreview = debounce(() => {
    buildNow();
  }, 250);

  async function buildNow() {
    const include = Object.entries(sections)
      .filter(([, on]) => on)
      .map(([key]) => key);
    try {
      const html = await api.buildReport(store.state, include);
      lastHtml = html;
      // srcdoc pages do not resolve the wrapper's relative asset paths, so the
      // stylesheet and scripts are pinned against the app's own base URL.
      const katexBase = new URL("vendor/katex/", document.baseURI).href;
      frame.srcdoc = html.split('"vendor/katex/').join(`"${katexBase}`);
      // A quiet self-check: the log records whether the equations actually
      // typeset, which is the one thing the window itself cannot show.
      frame.onload = () => {
        try {
          const doc = frame.contentDocument;
          const error = doc && doc.body ? doc.body.getAttribute("data-katex-error") : null;
          const nodes = doc ? doc.querySelectorAll(".katex").length : 0;
          window.uilog &&
            window.uilog(
              error ? `report katex error: ${error}` : `report preview ready: ${nodes} typeset equations`,
            );
        } catch (ignored) {
          /* the preview is still visible even when it cannot be inspected */
        }
        // Fit once the document is parsed, then once more when the KaTeX
        // fonts settle — their metrics shift the typeset line heights.
        fitFrame();
        const fonts = frame.contentDocument && frame.contentDocument.fonts;
        if (fonts && fonts.ready) fonts.ready.then(fitFrame, () => {});
      };
    } catch (error) {
      frame.srcdoc = `<p style="font-family: sans-serif; color: #9b1c1c; padding: 16px">${errorText(error)}</p>`;
    }
  }

  async function ensureHtml() {
    if (!lastHtml) await buildNow();
    return lastHtml;
  }

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

  return { root, sync, render: () => {}, setSections };
}
