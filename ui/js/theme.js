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
const HEX = /^#[0-9a-f]{6}$/i;

function mix(from, to, amount) {
  const parse = (hex) => [1, 3, 5].map((start) => Number.parseInt(hex.slice(start, start + 2), 16));
  const [r1, g1, b1] = parse(from);
  const [r2, g2, b2] = parse(to);
  const channel = (a, b) => Math.round(a + (b - a) * amount).toString(16).padStart(2, "0");
  return `#${channel(r1, r2)}${channel(g1, g2)}${channel(b1, b2)}`;
}

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
  "--scroll",
  "--placeholder",
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

/**
 * The desktop's own palette, when it publishes one: surfaces are mixed from its
 * background towards its text colour, so the shell is the same warmth as the
 * rest of the desktop instead of an approximation of it.
 */
export function paletteTokens(palette) {
  if (!palette) return null;
  const background = typeof palette.background === "string" ? palette.background.trim() : "";
  const text = typeof palette.text === "string" ? palette.text.trim() : "";
  if (!HEX.test(background) || !HEX.test(text)) return null;
  const surface =
    typeof palette.surface === "string" && HEX.test(palette.surface.trim())
      ? palette.surface.trim()
      : mix(background, text, 0.06);
  return {
    "--bg": background,
    "--bg-rail": mix(background, text, 0.03),
    "--surface": surface,
    "--surface-2": mix(background, text, 0.09),
    "--surface-3": mix(background, text, 0.13),
    "--surface-hi": mix(background, text, 0.18),
    "--border": mix(background, text, 0.16),
    "--border-soft": mix(background, text, 0.1),
    "--border-strong": mix(background, text, 0.26),
    "--text": text,
    "--muted": mix(text, background, 0.35),
    "--faint": mix(text, background, 0.55),
    "--scroll": mix(background, text, 0.22),
    "--placeholder": mix(text, background, 0.5),
  };
}

/** A tinted version of our palette: the accent's hue, our own lightness ladder. */
export function tintedPalette(accent, scheme) {
  const h = hueOf(accent);
  if (scheme === "light") {
    return {
      "--bg": hsl(h, 0.32, 0.93),
      "--bg-rail": hsl(h, 0.3, 0.89),
      "--surface": hsl(h, 0.3, 0.98),
      "--surface-2": hsl(h, 0.3, 0.96),
      "--surface-3": hsl(h, 0.32, 0.91),
      "--surface-hi": hsl(h, 0.34, 0.86),
      "--border": hsl(h, 0.28, 0.83),
      "--border-soft": hsl(h, 0.25, 0.88),
      "--border-strong": hsl(h, 0.3, 0.75),
      "--text": hsl(h, 0.28, 0.12),
      "--muted": hsl(h, 0.16, 0.38),
      "--faint": hsl(h, 0.13, 0.5),
      "--scroll": hsl(h, 0.2, 0.78),
      "--placeholder": hsl(h, 0.12, 0.58),
    };
  }
  return {
    "--bg": hsl(h, 0.25, 0.1),
    "--bg-rail": hsl(h, 0.27, 0.08),
    "--surface": hsl(h, 0.23, 0.13),
    "--surface-2": hsl(h, 0.25, 0.16),
    "--surface-3": hsl(h, 0.27, 0.2),
    "--surface-hi": hsl(h, 0.28, 0.25),
    "--border": hsl(h, 0.25, 0.22),
    "--border-soft": hsl(h, 0.22, 0.18),
    "--border-strong": hsl(h, 0.27, 0.31),
    "--text": hsl(h, 0.22, 0.92),
    "--muted": hsl(h, 0.16, 0.64),
    "--faint": hsl(h, 0.14, 0.47),
    "--scroll": hsl(h, 0.22, 0.23),
    "--placeholder": hsl(h, 0.12, 0.36),
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
  { preference, systemDark, accent, accentSource, palette } = {},
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

  // The desktop's own surfaces win when it publishes them *and* the operator
  // asked to follow the desktop: that is the same background and text the
  // terminals and the GTK applications use. An explicit Light/Dark always uses
  // our own ladder, tinted from the palette accent when there is a wallpaper
  // accent, so the chosen scheme still carries the desktop's colour.
  const desktopScheme = systemDark ? "dark" : "light";
  const followingDesktop = normalisePreference(preference) === "system";
  const exact =
    followingDesktop && scheme === desktopScheme ? paletteTokens(palette) : null;
  const surfaces =
    exact ?? (accentSource === "wallpaper" ? tintedPalette(vars.accent, scheme) : null);
  for (const name of TINT_SURFACES) {
    root.style.setProperty(name, surfaces ? surfaces[name] : "");
  }
  return { scheme, vars, tinted: Boolean(surfaces), exact: Boolean(exact) };
}
