// Settings: the appearance, and where its colours come from.

import { el } from "../dom.js";
import { card, field, selectInput } from "../components.js";

export function createSettingsScreen(ctx) {
  // Where the accent came from, in plain words: a desktop palette tints the
  // whole shell, a desktop accent only colours the controls.
  function accentSummary() {
    const source = ctx.accentSource ? ctx.accentSource() : "default";
    if (source === "wallpaper") {
      const colour = ctx.accent ? ctx.accent() : "";
      return `Accent: ${colour} from the desktop's palette (Noctalia, pywal or wallust); the surfaces are tinted to match.`;
    }
    if (source === "desktop") {
      const colour = ctx.accent ? ctx.accent() : "";
      return `Accent: ${colour} from the desktop's own accent setting.`;
    }
    return "Accent: the app's own blue \u2014 this desktop publishes no accent and no wallpaper palette.";
  }

  const root = el(
    "div",
    { class: "page", style: { maxWidth: "760px" } },
    card({
      title: "Appearance",
      body: el(
        "div",
        {},
        el("p", {
          class: "muted small",
          style: { margin: "0 0 10px" },
          text:
            "The palette follows the desktop's light or dark preference and borrows its " +
            "accent colour when one is published, so it sits alongside the rest of the " +
            "system. Otherwise it uses its own neutral pair.",
        }),
        el("p", { class: "faint small accent-summary", style: { margin: "0 0 10px" }, text: accentSummary() }),
        field({
          label: "Colour scheme",
          hint: "Matching the desktop is what most desktops expect; the other two override it.",
          input: selectInput({
            options: [
              { key: "system", label: "Match the desktop" },
              { key: "light", label: "Light" },
              { key: "dark", label: "Dark" },
            ],
            value: ctx.themePreference ? ctx.themePreference() : "system",
            onChange: (value) => ctx.setThemePreference && ctx.setThemePreference(value),
          }),
        }),
      ),
    }),
  );

  // The palette is re-read while the app runs, so the caption is refreshed when
  // the screen is opened rather than frozen at startup.
  return {
    root,
    sync: () => {
      const caption = root.querySelector(".accent-summary");
      if (caption) caption.textContent = accentSummary();
    },
    render: () => {},
  };
}
