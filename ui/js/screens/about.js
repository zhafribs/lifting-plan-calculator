// About: what the app does, who built it, and the update path.

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
      if (ctx.platform === "linux" && info.appimage) {
        buttons.unshift(
          el("button", { class: "btn primary small", onClick: () => runAppImageUpdate(info) }, "Download & update"),
        );
      }
      if (ctx.platform === "linux" && info.package) {
        buttons.unshift(
          el("button", { class: "btn primary small", onClick: () => runPackageUpdate(info) }, "Download & install"),
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

  async function runAppImageUpdate(info) {
    updateStatus.textContent = "Downloading the new AppImage\u2026";
    try {
      await api.installUpdate(info.appimage);
      updateStatus.textContent = "The new AppImage is ready \u2014 the app will close, replace the old file and restart.";
      setTimeout(() => api.quitApp(), 1500);
    } catch (error) {
      updateStatus.textContent = errorText(error);
    }
  }

  // --- the Arch package -----------------------------------------------------
  // It installs through pkexec, so a desktop without polkit cannot ask for the
  // password. The app then offers what to install, and always leaves the manual
  // command for an operator who would rather not use the button at all.

  function commandRow(command) {
    const code = el("code", {
      text: command,
      style: {
        display: "block",
        marginTop: "6px",
        padding: "7px 9px",
        background: "var(--surface-2)",
        border: "1px solid var(--border)",
        borderRadius: "7px",
        userSelect: "all",
        whiteSpace: "pre-wrap",
        font: "12px ui-monospace, monospace",
      },
    });
    const copy = el(
      "button",
      {
        class: "btn ghost small",
        onClick: async () => {
          try {
            await navigator.clipboard.writeText(command);
            copy.textContent = "Copied";
          } catch {
            const range = document.createRange();
            range.selectNodeContents(code);
            const selection = window.getSelection();
            selection.removeAllRanges();
            selection.addRange(range);
          }
        },
      },
      "Copy",
    );
    return el(
      "div",
      { style: { marginTop: "8px" } },
      el("div", { class: "row", style: { alignItems: "center", gap: "8px" } }, code, copy),
    );
  }

  function showPackageFailure(result) {
    const parts = [el("div", { class: "faint small", text: "Nothing was installed." })];
    if (result.setup) {
      parts.push(
        el("div", { class: "faint small", style: { marginTop: "6px" }, text: result.setup.reason }),
        el("div", {
          class: "faint small",
          style: { marginTop: "2px" },
          text: "Run this, then press Download & install again:",
        }),
      );
      for (const command of result.setup.commands) parts.push(commandRow(command));
    }
    parts.push(
      el("div", {
        class: "faint small",
        style: { marginTop: "12px" },
        text: "Or update by hand \u2014 the file is already downloaded and verified:",
      }),
      commandRow(result.command),
    );
    updateStatus.replaceChildren(...parts);
  }

  async function runPackageUpdate(info) {
    updateStatus.textContent = "Downloading the package\u2026";
    try {
      const result = await api.installPackageUpdate(info.package);
      if (result.installed) {
        updateStatus.textContent =
          "The package is installed \u2014 the app will close; open it again to run the new version.";
        setTimeout(() => api.quitApp(), 1500);
      } else {
        showPackageFailure(result);
      }
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
            "On Windows it can fetch and start the release's installer. On Linux it can " +
            "download the new AppImage, replace the running file and restart, and the " +
            "Arch package installs itself through the system's polkit prompt.",
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
        el("p", { class: "faint small", style: { margin: "10px 2px 0" }, text: "The assistants wrote code; the engineering decisions and the checks are the authors'. The figures are only as good as the inputs and the load chart: check both, and check the plan you sign, against the source." }),
      ),
    }),
    el("p", { class: "faint small", style: { textAlign: "center", margin: "18px 0" }, text: "This desktop build opens straight into the calculator and needs no sign-in; all calculations run locally on this machine." }),
  );

  return { root, sync: () => {}, render: () => {} };
}
