// Summary: the printable calculation report, typeset with the bundled KaTeX.

import { el, svgIcon, debounce } from "../dom.js";
import { api, errorText } from "../api.js";
import { card, field, textInput } from "../components.js";
import { captureWorkingDiagram } from "../graph/snapshot.js";

export function createReportScreen(ctx) {
  const { store, catalog, toast } = ctx;

  const sections = { crane: true, uniform: true, nonuniform: true, tandem: true };
  let lastHtml = "";
  let lastSavedPath = null;
  const DEFAULT_FOOTER =
    "Prepared with Lifting Plan Calculator by Zhafri Syazwi ✉ zhafribs@gmail.com";
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

  const diagramInput = el("input", { type: "checkbox" });
  diagramInput.addEventListener("change", () => {
    if (!diagramInput.checked) {
      schedulePreview(0);
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

  const frame = el("iframe", { title: "Report preview" });
  const frameWrap = el("div", { class: "report-frame" }, frame);

  // One scrollable: stacked (the same max-width the layout stacks at), the
  // Summary tab reads as a single document — the frame opens to the report's
  // full height and the screen owns the only scrollbar. Side by side the
  // frame keeps its viewport-sized box and its own scroll, which is the one
  // scrollable there. 980px is the stack cut shared with the CSS breakpoints.
  const stackedQuery = window.matchMedia("(max-width: 980px)");
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

  const pdfButton = el(
    "button",
    {
      class: "btn wide",
      onClick: async () => {
        try {
          const html = await ensureHtml();
          const path = await api.savePdf(html, "lifting-plan.pdf");
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
            ...planFields.map(([key, label]) =>
              field({
                label,
                input: textInput({
                  value: signoff[key],
                  placeholder: key === "reference" ? "LP-YYYY-NNN" : "",
                  onInput: (value) => {
                    signoff[key] = value;
                    schedulePreview(0);
                  },
                }),
              }),
            ),
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
              input: textInput({
                value: footerText,
                onInput: (value) => {
                  footerText = value;
                  schedulePreview(0);
                },
              }),
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
          foot: el("span", { class: "faint", text: "Save as PDF writes A4 with 1-inch margins (Chromium on Linux, Edge on Windows). Saved HTML keeps a copy of the KaTeX assets beside it; when printing it, keep Margins on Default (1 inch)." }),
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
      const diagram = diagramInput.checked ? captureWorkingDiagram() : null;
      if (diagramInput.checked && !diagram) diagramInput.checked = false;
      const html = await api.buildReport(store.state, include, issuedAt, signoff, diagram, footerText);
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
