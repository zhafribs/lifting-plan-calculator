// About: what the app does, who built it, and the warning that matters.

import { el } from "../dom.js";
import { card, kvRows } from "../components.js";

export function createAboutScreen(ctx) {
  const { catalog } = ctx;

  const root = el(
    "div",
    { class: "page", style: { maxWidth: "760px" } },
    el(
      "div",
      { class: "about-hero" },
      el("img", { src: "assets/crane-icon.png", alt: "" }),
      el("h2", { text: catalog.app_name }),
      el("div", { class: "tagline", text: "Sizes a crane lift and prints the working — every figure with the formula and the values that produced it." }),
      el("span", { class: "pill", text: `Version ${catalog.version}` }),
    ),
    card({
      title: "What this does",
      body: el(
        "div",
        {},
        el("p", { class: "muted small", style: { margin: "0 0 8px" }, text: "It totals the load at the hook, checks the crane against its own load chart, and sizes the slings for uniform, off-centre and two-crane lifts." }),
        el("p", { class: "faint small", style: { margin: 0 }, text: "Every figure comes with the formula and the values that produced it, so a reviewer can check the arithmetic rather than take it on trust." }),
      ),
    }),
    card({
      title: "Developer",
      body: el(
        "div",
        {},
        kvRows([
          ["Name", catalog.contact_name],
          ["Email", catalog.contact_email],
          ["Phone", catalog.contact_phone],
        ]),
        el("p", { class: "faint small", style: { margin: "10px 2px 0" }, text: "The figures are checked against the crane's own load chart before a lift is made — a lifting plan is only as good as the person who checks it." }),
      ),
    }),
    card({
      title: "Built with",
      body: el(
        "div",
        {},
        kvRows([
          ["Assistants", "Google AI, opencode, DeepSeek v4.1-flash"],
          ["Desktop stack", "Tauri (Rust engine) with a bundled KaTeX report renderer"],
        ]),
        el("p", { class: "faint small", style: { margin: "10px 2px 0" }, text: "The assistants wrote code; the engineering decisions, the checks and the responsibility for the figures are the developer's." }),
      ),
    }),
    el("p", { class: "faint small", style: { textAlign: "center", margin: "18px 0" }, text: "This desktop build opens straight into the calculator and needs no sign-in; all calculations run locally on this machine." }),
  );

  return { root, sync: () => {}, render: () => {} };
}
