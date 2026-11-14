// The page view's arithmetic, without a webview: zoom, fitting and the pages
// that keep a canvas.

import test from "node:test";
import assert from "node:assert/strict";

import {
  ZOOM_MAX,
  ZOOM_MIN,
  backingScale,
  clampZoom,
  fitPage,
  fitWidth,
  keptPages,
  pageLabel,
  stepZoom,
} from "./pdf-math.js";

test("zoom stays within the range the controls offer", () => {
  assert.equal(clampZoom(0.1), ZOOM_MIN);
  assert.equal(clampZoom(9), ZOOM_MAX);
  assert.equal(clampZoom(Number.NaN), 1);
  assert.equal(clampZoom(1.25), 1.25);
});

test("fit width leaves a gutter and never divides by nothing", () => {
  // A4 is 595 pt wide; a 900 px pane at 48 px of padding is 1.43x.
  assert.equal(Math.round(fitWidth(900, 595) * 100), 143);
  assert.equal(fitWidth(0, 595), 1);
  assert.equal(fitWidth(900, 0), 1);
  // A very wide page in a narrow pane stops at the minimum, not at nothing.
  assert.equal(fitWidth(200, 595), ZOOM_MIN);
});

test("fit page uses the tighter of the two sides", () => {
  const page = { width: 595, height: 842 };
  // 900x1200: width gives 1.43, height gives 1.37 — the height wins.
  assert.equal(Math.round(fitPage(900, 1200, page) * 100), 137);
  // Narrow and short: the width wins.
  assert.equal(Math.round(fitPage(600, 2000, page) * 100), 93);
  assert.equal(fitPage(0, 1200, page), 1);
  assert.equal(fitPage(900, 1200, null), 1);
});

test("the backing store keeps text crisp without a canvas nobody can see", () => {
  assert.equal(backingScale(1, 1), 1);
  assert.equal(backingScale(2, 1), 2);
  // A 3x screen is capped at 2x: 200% zoom at 3x would be a 6x canvas.
  assert.equal(backingScale(1, 3), 2);
  assert.equal(backingScale(2, 2), 4);
  // Zero or missing density still renders.
  assert.equal(backingScale(1, 0), 1);
});

test("a zoom step lands on whole percentages and stops at the ends", () => {
  assert.equal(stepZoom(1, 1), 1.15);
  assert.equal(stepZoom(1, -1), 0.85);
  assert.equal(stepZoom(ZOOM_MAX, 1), ZOOM_MAX);
  assert.equal(stepZoom(ZOOM_MIN, -1), ZOOM_MIN);
});

test("only the pages around the reader keep a canvas", () => {
  assert.deepEqual([...keptPages(0, 7, 2)], [0, 1, 2]);
  assert.deepEqual([...keptPages(3, 7, 2)], [1, 2, 3, 4, 5]);
  assert.deepEqual([...keptPages(6, 7, 2)], [4, 5, 6]);
  // A single-page plan keeps exactly one page, not a negative index.
  assert.deepEqual([...keptPages(0, 1, 2)], [0]);
});

test("the page label is one-based and clamped", () => {
  assert.equal(pageLabel(1, 7), "Page 1 of 7");
  assert.equal(pageLabel(0, 7), "Page 1 of 7");
  assert.equal(pageLabel(99, 7), "Page 7 of 7");
  assert.equal(pageLabel(1, 0), "No pages");
});
