// The appearance rules: which scheme wins, and what accent comes out of it.

import test from "node:test";
import assert from "node:assert/strict";

import {
  THEME_PREFERENCES,
  desktopIsDark,
  normalisePreference,
  resolveScheme,
  themeAccent,
  tintedPalette,
  paletteTokens,
} from "./theme.js";

function relativeLuminance(hex) {
  const [r, g, b] = [1, 3, 5].map((start) => Number.parseInt(hex.slice(start, start + 2), 16) / 255)
    .map((value) => (value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function hue(hex) {
  const [r, g, b] = [1, 3, 5].map((start) => Number.parseInt(hex.slice(start, start + 2), 16) / 255);
  const max = Math.max(r, g, b);
  const delta = max - Math.min(r, g, b);
  if (!delta) return 0;
  const value = max === r ? ((g - b) / delta) % 6 : max === g ? (b - r) / delta + 2 : (r - g) / delta + 4;
  return (value * 60 + 360) % 360;
}

function contrast(a, b) {
  const [light, dark] = [relativeLuminance(a), relativeLuminance(b)].sort((x, y) => y - x);
  return (light + 0.05) / (dark + 0.05);
}

test("an unknown or missing preference is 'system'", () => {
  assert.equal(normalisePreference(""), "system");
  assert.equal(normalisePreference("midnight"), "system");
  assert.equal(normalisePreference(undefined), "system");
  for (const preference of THEME_PREFERENCES) {
    assert.equal(normalisePreference(preference), preference);
  }
});

test("system follows the desktop, an explicit choice does not", () => {
  assert.equal(resolveScheme("system", true), "dark");
  assert.equal(resolveScheme("system", false), "light");
  assert.equal(resolveScheme("light", true), "light");
  assert.equal(resolveScheme("dark", false), "dark");
  assert.equal(resolveScheme(undefined, true), "dark");
});

test("the desktop accent is used when it is a colour", () => {
  assert.equal(themeAccent("#ff8800", "dark").accent, "#ff8800");
  assert.equal(themeAccent("#FF8800", "light").accent, "#ff8800");
});

test("each palette has its own fallback accent", () => {
  assert.equal(themeAccent(null, "dark").accent, "#5cb3ff");
  assert.equal(themeAccent(null, "light").accent, "#2b6cb8");
  assert.equal(themeAccent("not-a-colour", "light").accent, "#2b6cb8");
});

test("the accent tokens keep a readable contrast pair", () => {
  assert.equal(themeAccent("#2b6cb8", "light").accentContrast, "#ffffff");
  assert.match(themeAccent("#5cb3ff", "dark").accentSoft, /^rgba\(/);
});

test("a wallpaper palette tints the surfaces with its own hue", () => {
  for (const scheme of ["dark", "light"]) {
    const palette = tintedPalette("#7dcfff", scheme);
    // 191° is the cyan of the sample. The low-chroma surfaces round their
    // channels, so the hue is checked where the tint is strongest.
    assert.ok(Math.abs(hue(palette["--text"]) - 191) < 25, `${scheme} text hue`);
    assert.ok(Math.abs(hue(palette["--surface-hi"]) - 191) < 25, `${scheme} surface hue`);
    assert.notEqual(palette["--surface"], palette["--bg"]);
  }
});

test("the tinted palette keeps text readable on its own surfaces", () => {
  for (const scheme of ["dark", "light"]) {
    for (const accent of ["#7dcfff", "#f7768e", "#9ece6a", "#ff5500", "#808080"]) {
      const palette = tintedPalette(accent, scheme);
      assert.ok(
        contrast(palette["--text"], palette["--bg"]) >= 7,
        `${scheme} ${accent}: text on background`,
      );
      assert.ok(
        contrast(palette["--text"], palette["--surface"]) >= 7,
        `${scheme} ${accent}: text on surface`,
      );
    }
  }
});

test("a hue-less accent still produces a usable palette", () => {
  const palette = tintedPalette("#808080", "dark");
  assert.match(palette["--bg"], /^#[0-9a-f]{6}$/);
  assert.ok(contrast(palette["--text"], palette["--bg"]) >= 7);
});

test("the desktop's own scheme wins over the webview's guess", () => {
  assert.equal(desktopIsDark("dark", false), true);
  assert.equal(desktopIsDark("light", true), false);
  assert.equal(desktopIsDark(null, true), true);
  assert.equal(desktopIsDark(undefined, false), false);
});

test("the desktop palette supplies the exact background and text", () => {
  const tokens = paletteTokens({ background: "#1a1111", surface: "#271d1d", text: "#f0dede" });
  assert.equal(tokens["--bg"], "#1a1111");
  assert.equal(tokens["--surface"], "#271d1d");
  assert.equal(tokens["--text"], "#f0dede");
  // Secondary text sits between the two, leaning towards the background.
  assert.ok(contrast(tokens["--muted"], "#1a1111") < contrast("#f0dede", "#1a1111"));
  assert.ok(contrast(tokens["--muted"], "#1a1111") > contrast(tokens["--faint"], "#1a1111"));
});

test("a missing or malformed desktop palette is ignored", () => {
  assert.equal(paletteTokens(null), null);
  assert.equal(paletteTokens({ background: "not-a-colour", text: "#ffffff" }), null);
  assert.equal(paletteTokens({ background: "#101010" }), null);
  const noSurface = paletteTokens({ background: "#101010", text: "#eeeeee" });
  assert.equal(noSurface["--text"], "#eeeeee");
  assert.match(noSurface["--surface"], /^#[0-9a-f]{6}$/);
});
