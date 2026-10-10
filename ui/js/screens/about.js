// About: what the app does, who built it, and the warning that matters.

import { el } from "../dom.js";
import { card, kvRows } from "../components.js";
import { api, errorText } from "../api.js";

const REPO_URL = "https://github.com/zhafribs/lifting-plan-calculator";

export function createAboutScreen(ctx) {
  const { catalog } = ctx;

  const updateStatus = el("div", { class: "faint small", style: { margin: "10px 2px 0" } });
  const updateButtons = el("div", { class: "row wrap" });

  function renderUpdateButtons(info) {
    const buttons = [
      el(
        "button",
        { class: "btn ghost small", onClick: () => api.openExternal(REPO_URL).catch(() => {}) },
        "Visit repository",
      ),
      el("button", { class: "btn ghost small", onClick: runUpdateCheck }, "Check for updates"),
    ];
    if (info && info.newer) {
      buttons.unshift(
        el(
          "button",
          { class: "btn ghost small", onClick: () => api.openExternal(info.release_url).catch(() => {}) },
          "Release notes",
        ),
      );
      if (ctx.platform === "windows" && info.installer) {
        buttons.unshift(
          el("button", { class: "btn primary small", onClick: () => runInstaller(info) }, "Download & install"),
        );
      }
    }
    updateButtons.replaceChildren(...buttons);
  }

  async function runUpdateCheck() {
    updateStatus.textContent = "Checking GitHub\u2026";
    try {
      const info = await api.checkUpdate();
      updateStatus.textContent = info.newer
        ? `Version ${info.latest} is available \u2014 you have ${info.current}.`
        : `You have the newest version (${info.current}).`;
      renderUpdateButtons(info);
    } catch (error) {
      updateStatus.textContent = errorText(error);
      renderUpdateButtons(null);
    }
  }

  async function runInstaller(info) {
    updateStatus.textContent = "Downloading the installer\u2026";
    try {
      await api.installUpdate(info.installer);
      updateStatus.textContent = "The installer has started \u2014 the app will close so it can update.";
      setTimeout(() => api.quitApp(), 1500);
    } catch (error) {
      updateStatus.textContent = errorText(error);
    }
  }

  renderUpdateButtons(null);

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
      title: "Updates",
      body: el(
        "div",
        {},
        el("p", {
          class: "muted small",
          style: { margin: "0 0 10px" },
          text:
            "The app checks GitHub for a newer release at launch, and on demand here. " +
            "On Windows it can fetch and start the release's installer; the AppImage " +
            "is only reminded — its release page carries the new file.",
        }),
        updateButtons,
        updateStatus,
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
