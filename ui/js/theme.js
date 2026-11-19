// The shell's appearance: which palette to paint, and the accent that goes with
// it. The scheme follows the desktop unless the operator chose one in About; the
// accent is the desktop's own when it publishes one, ours otherwise.
//
// Pure, so the rules can be tested without a webview.

import { accentVars } from "./windows-theme.js";

export const THEME_PREFERENCES = ["system", "light", "dark"];

// The accent each palette falls back to when the desktop does not answer.
const FALLBACK_ACCENT = { dark: "#5cb3ff", light: "#2b6cb8" };

export function normalisePreference(value) {
  return THEME_PREFERENCES.includes(value) ? value : "system";
}

/** "system" resolves against the desktop; an explicit choice always wins. */
export function resolveScheme(preference, systemDark) {
  const choice = normalisePreference(preference);
  if (choice === "light" || choice === "dark") return choice;
  return systemDark ? "dark" : "light";
}

/** The accent tokens for a scheme, with our fallback when the desktop has none. */
export function themeAccent(hex, scheme) {
  const fallback = FALLBACK_ACCENT[scheme === "dark" ? "dark" : "light"];
  const valid = typeof hex === "string" && /^#?[0-9a-f]{6}$/i.test(hex.trim());
  return accentVars(valid ? hex.trim() : fallback, scheme === "dark");
}

/**
 * Paint the appearance onto the document root. Returns what was applied, which
 * the caller logs and the About screen mirrors.
 */
export function applyTheme(root, { preference, systemDark, accent } = {}) {
  const scheme = resolveScheme(preference, systemDark);
  const vars = themeAccent(accent, scheme);
  root.dataset.theme = scheme;
  root.style.colorScheme = scheme;
  root.style.setProperty("--accent", vars.accent);
  root.style.setProperty("--accent-soft", vars.accentSoft);
  root.style.setProperty("--accent-line", vars.accent);
  root.style.setProperty("--accent-strong", vars.accentHover);
  root.style.setProperty("--accent-hover", vars.accentHover);
  root.style.setProperty("--accent-contrast", vars.accentContrast);
  return { scheme, vars };
}
