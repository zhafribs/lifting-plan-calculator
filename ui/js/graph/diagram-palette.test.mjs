import test from "node:test";
import assert from "node:assert/strict";

import {
  DIAGRAM_DARK,
  DIAGRAM_LIGHT,
  PRINT_PALETTE,
  diagramPalette,
} from "./diagram-palette.js";
import { printColour } from "./print-palette.js";

function channels(colour) {
  const rgb = /^rgba?\(([^)]+)\)$/i.exec(colour);
  if (rgb) return rgb[1].split(",").slice(0, 3).map((part) => Number.parseFloat(part));
  const hex = colour.replace("#", "");
  return [0, 2, 4].map((index) => Number.parseInt(hex.slice(index, index + 2), 16));
}

function luminance(colour) {
  const [r, g, b] = channels(colour).map((value) => {
    const channel = value / 255;
    return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a, b) {
  const [bright, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (bright + 0.05) / (dark + 0.05);
}

test("the light palette is the dark one through the print filter", () => {
  for (const [key, value] of Object.entries(DIAGRAM_DARK)) {
    if (key === "danger") continue;
    assert.equal(DIAGRAM_LIGHT[key], printColour(value), key);
  }
});

test("the light warning red is chosen for the screen, not printed", () => {
  assert.notEqual(DIAGRAM_LIGHT.danger, printColour(DIAGRAM_DARK.danger));
  assert.ok(contrast(DIAGRAM_LIGHT.danger, DIAGRAM_LIGHT.bg) >= 4.5);
});

test("the light chart is light and the dark chart is dark", () => {
  assert.ok(luminance(DIAGRAM_LIGHT.bg) > 0.8, DIAGRAM_LIGHT.bg);
  assert.ok(luminance(DIAGRAM_DARK.bg) < 0.05, DIAGRAM_DARK.bg);
});

test("the light chart's text stays legible", () => {
  for (const key of ["text", "jib", "boom", "danger"]) {
    assert.ok(contrast(DIAGRAM_LIGHT[key], DIAGRAM_LIGHT.bg) >= 4.5, key);
  }
  // The tick grey is the printed page's own faint value, kept so the screen
  // matches the page; it is deliberately below the text floor.
  assert.ok(contrast(DIAGRAM_LIGHT.axisText, DIAGRAM_LIGHT.bg) >= 3.5);
});

test("the print starts from the dark palette", () => {
  assert.equal(PRINT_PALETTE, DIAGRAM_DARK);
  assert.notEqual(PRINT_PALETTE, DIAGRAM_LIGHT);
});

test("diagramPalette follows the scheme and defaults to dark", () => {
  assert.equal(diagramPalette("light"), DIAGRAM_LIGHT);
  assert.equal(diagramPalette("dark"), DIAGRAM_DARK);
  assert.equal(diagramPalette(undefined), DIAGRAM_DARK);
});
