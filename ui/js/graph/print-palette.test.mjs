// The print palette must match what the CSS filter chain produced, or the
// report's diagram changes colour. The expected values are what Chromium yields
// for `invert(1) hue-rotate(180deg) brightness(1.08)` on the same colours.

import test from "node:test";
import assert from "node:assert/strict";

import { printColour, printify } from "./print-palette.js";

const CASES = [
  [[20, 25, 34], [244, 249, 255]], // background
  [[223, 230, 240], [19, 28, 37]], // label text
  [[255, 183, 3], [150, 72, 0]], // boom
  [[92, 179, 255], [16, 110, 192]], // jib
  [[255, 0, 0], [255, 157, 157]],
  [[0, 0, 255], [235, 235, 255]],
];

test("the print palette is the CSS filter chain's result", () => {
  const data = new Uint8ClampedArray(CASES.flatMap(([rgb]) => [...rgb, 255]));
  printify(data);
  CASES.forEach(([input, expected], index) => {
    const got = [...data.slice(index * 4, index * 4 + 3)];
    got.forEach((value, channel) => {
      assert.ok(
        Math.abs(value - expected[channel]) <= 2,
        `${input} channel ${channel}: ${value} vs ${expected[channel]}`,
      );
    });
  });
});

test("alpha is left alone", () => {
  const data = new Uint8ClampedArray([20, 25, 34, 128]);
  printify(data);
  assert.equal(data[3], 128);
});

function channels(value) {
  return [...value.matchAll(/[\d.]+/g)].slice(0, 3).map((match) => Number(match[0]));
}

test("printColour matches the pixel filter within rounding", () => {
  for (const [input, expected] of [
    ["#141922", [244, 249, 255]],
    ["rgb(255, 183, 3)", [150, 72, 0]],
  ]) {
    const actual = channels(printColour(input));
    actual.forEach((value, index) => {
      assert.ok(Math.abs(value - expected[index]) <= 2, `${input} channel ${index}`);
    });
  }
});

test("printColour preserves alpha", () => {
  assert.match(printColour("rgba(92, 179, 255, 0.5)"), /^rgba\(.*0\.5\)$/);
});
