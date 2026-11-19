// The appearance rules: which scheme wins, and what accent comes out of it.

import test from "node:test";
import assert from "node:assert/strict";

import {
  THEME_PREFERENCES,
  normalisePreference,
  resolveScheme,
  themeAccent,
} from "./theme.js";

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
