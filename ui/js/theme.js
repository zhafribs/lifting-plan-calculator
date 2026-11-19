// The shell's appearance: which palette to paint, and the accent that goes with
// it. The scheme follows the desktop unless the operator chose one in About; the
// accent is the desktop's own when it publishes one, ours otherwise.
//
// Pure, so the rules can be tested without a webview.

import { accentVars } from "./windows-theme.js";

export const THEME_PREFERENCES = ["system", "light", "dark"];

// The accent each palette falls back to when the desktop does not answer.
const FALLBACK_ACCENT = { dark: "#5cb3ff", light: "#2b6cb8" };

// The surfaces a wallpaper palette is allowed to tint. The lightness ladder is
// ours, so contrast is guaranteed; only the hue and a little chroma come from
// the wallpaper.
const TINT_SURFACES = [
  "--bg",
  "--bg-rail",
  "--surface",
  "--surface-2",
  "--surface-3",
  "--surface-hi",
  "--border",
  "--border-soft",
  "--border-strong",
  "--text",
  "--muted",
  "--faint",
];

function channels(hex) {
  const clean = hex.replace("#", "");
  return [0, 2, 4].map((index) => Number.parseInt(clean.slice(index, index + 2), 16));
}

function hueOf(hex) {
  const [r, g, b] = channels(hex).map((value) => value / 255);
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const delta = max - min;
  if (delta === 0) return 0;
  let hue;
  if (max === r) hue = ((g - b) / delta) % 6;
  else if (max === g) hue = (b - r) / delta + 2;
  else hue = (r - g) / delta + 4;
  return (hue * 60 + 360) % 360;
}

function hsl(hue, saturation, lightness) {
  const a = saturation * Math.min(lightness, 1 - lightness);
  const channel = (n) => {
    const k = (n + hue / 30) % 12;
    const value = lightness - a * Math.max(-1, Math.min(k - 3, 9 - k, 1));
    return Math.round(value * 255).toString(16).padStart(2, "0");
  };
  return `#${channel(0)}${channel(8)}${channel(4)}`;
}

/** A tinted version of our palette: the accent's hue, our own lightness ladder. */
export function tintedPalette(accent, scheme) {
  const h = hueOf(accent);
  if (scheme === "light") {
    return {
      "--bg": hsl(h, 0.22, 0.97),
      "--bg-rail": hsl(h, 0.2, 0.93),
      "--surface": "#ffffff",
      "--surface-2": hsl(h, 0.2, 0.985),
      "--surface-3": hsl(h, 0.21, 0.94),
      "--surface-hi": hsl(h, 0.23, 0.9),
      "--border": hsl(h, 0.18, 0.87),
      "--border-soft": hsl(h, 0.16, 0.91),
      "--border-strong": hsl(h, 0.2, 0.8),
      "--text": hsl(h, 0.26, 0.12),
      "--muted": hsl(h, 0.15, 0.4),
      "--faint": hsl(h, 0.12, 0.52),
    };
  }
  return {
    "--bg": hsl(h, 0.14, 0.08),
    "--bg-rail": hsl(h, 0.16, 0.06),
    "--surface": hsl(h, 0.13, 0.11),
    "--surface-2": hsl(h, 0.14, 0.14),
    "--surface-3": hsl(h, 0.15, 0.18),
    "--surface-hi": hsl(h, 0.16, 0.23),
    "--border": hsl(h, 0.15, 0.2),
    "--border-soft": hsl(h, 0.14, 0.16),
    "--border-strong": hsl(h, 0.17, 0.28),
    "--text": hsl(h, 0.22, 0.93),
    "--muted": hsl(h, 0.15, 0.65),
    "--faint": hsl(h, 0.13, 0.48),
  };
}

export function normalisePreference(value) {
  return THEME_PREFERENCES.includes(value) ? value : "system";
}

/** The desktop's own scheme when it publishes one, the webview's otherwise. */
export function desktopIsDark(hint, mediaDark) {
  if (hint === "dark") return true;
  if (hint === "light") return false;
  return Boolean(mediaDark);
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
export function applyTheme(
  root,
  { preference, systemDark, accent, accentSource } = {},
) {
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

  // A wallpaper palette tints the surfaces too; every other source keeps the
  // neutral palette the stylesheet ships with.
  const tint = accentSource === "wallpaper" ? tintedPalette(vars.accent, scheme) : null;
  for (const name of TINT_SURFACES) {
    root.style.setProperty(name, tint ? tint[name] : "");
  }
  return { scheme, vars, tinted: Boolean(tint) };
}
