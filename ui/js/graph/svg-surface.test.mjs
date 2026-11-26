import test from "node:test";
import assert from "node:assert/strict";

import { createSvgSurface } from "./svg-surface.js";

function surface() {
  return createSvgSurface({
    width: 100,
    height: 80,
    scale: 2,
    measure: { font: "", measureText: (text) => ({ width: text.length * 8 }) },
  });
}

test("a filled path is emitted with the print colour", () => {
  const s = surface();
  s.fillStyle = "#141922";
  s.beginPath();
  s.moveTo(0, 0);
  s.lineTo(10, 10);
  s.fill();
  const svg = s.toString();
  assert.match(svg, /<svg[^>]*width="200"[^>]*height="160"/);
  assert.match(svg, /viewBox="0 0 100 80"/);
  assert.match(svg, /<path d="M 0 0 L 10 10"/);
  assert.match(svg, /fill="rgb\(244, 249, 255\)"/);
});

test("fillRect becomes a rect in print colours", () => {
  const s = surface();
  s.fillStyle = "rgb(255, 183, 3)";
  s.fillRect(2, 3, 20, 10);
  const svg = s.toString();
  assert.match(svg, /<rect x="2" y="3" width="20" height="10"/);
  assert.match(svg, /fill="rgb\(1[45][0-9], 7[0-9], 0\)"/);
});

test("text keeps its anchor, baseline and escapes", () => {
  const s = surface();
  s.fillStyle = "#dfe6f0";
  s.font = "700 12px Inter, sans-serif";
  s.textAlign = "center";
  s.textBaseline = "middle";
  s.fillText("a & <b>", 20, 30);
  const svg = s.toString();
  assert.match(svg, /<text[^>]*text-anchor="middle"/);
  assert.match(svg, /dominant-baseline="central"/);
  assert.match(svg, /font-weight="700"/);
  assert.match(svg, /font-size="12"/);
  assert.match(svg, />a &amp; &lt;b&gt;<\/text>/);
});

test("measureText delegates to the supplied context", () => {
  const s = surface();
  assert.equal(s.measureText("abc").width, 24);
});

test("a dashed stroke carries stroke-dasharray", () => {
  const s = surface();
  s.strokeStyle = "#5f7191";
  s.lineWidth = 2;
  s.beginPath();
  s.moveTo(0, 0);
  s.lineTo(40, 0);
  s.setLineDash([7, 5]);
  s.stroke();
  const svg = s.toString();
  assert.match(svg, /stroke="rgb\(/);
  assert.match(svg, /stroke-width="2"/);
  assert.match(svg, /stroke-dasharray="7 5"/);
});

test("rgba fill keeps its alpha", () => {
  const s = surface();
  s.fillStyle = "rgba(92, 179, 255, 0.5)";
  s.fillRect(0, 0, 10, 10);
  const svg = s.toString();
  assert.match(svg, /fill="rgb\(/);
  assert.match(svg, /fill-opacity="0.5"/);
});
