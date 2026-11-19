import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  platformScope,
  initialNavMode,
  nextNavMode,
  accentVars,
  navToggleLabel,
} from "./windows-theme.js";

test("windows platform is scoped; linux is default", () => {
  assert.equal(platformScope({ platform: "windows" }), "windows");
  assert.equal(platformScope({ platform: "linux" }), "default");
  assert.equal(platformScope({ platform: "linux", platform_preview: "windows" }), "windows");
});

test("the rail starts expanded on a wide window and compact on a narrow one", () => {
  assert.equal(initialNavMode(1440), "expanded");
  assert.equal(initialNavMode(900), "compact");
});

test("the rail toggles both ways and labels its action", () => {
  assert.equal(nextNavMode("expanded"), "compact");
  assert.equal(nextNavMode("compact"), "expanded");
  assert.equal(navToggleLabel("expanded"), "Collapse navigation");
  assert.equal(navToggleLabel("compact"), "Expand navigation");
});

test("accent contrast follows the accent luminance", () => {
  assert.equal(accentVars("#0078D4", false).accentContrast, "#ffffff");
  assert.equal(accentVars("#FCE100", false).accentContrast, "#000000");
  assert.match(accentVars("#0078D4", true).accentSoft, /^rgba?\(/);
});

test("the windows stylesheet is scoped and follows the app's scheme", () => {
  const css = readFileSync(new URL("../css/windows.css", import.meta.url), "utf8");
  assert.match(css, /\[data-platform="windows"\]/);
  // The Settings switch is the authority: a media query here would override it.
  assert.match(css, /\[data-platform="windows"\]\[data-theme="dark"\]/);
  assert.doesNotMatch(css, /prefers-color-scheme/);
  assert.match(css, /--font-ui/);
});
