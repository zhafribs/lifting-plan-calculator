// A one-image report for the Windows CI job: headless Edge is asked to print an
// SVG diagram that carries the embedded Inter subset, which is what the app's
// report does. The app's own export probe has no diagram, so this checks the
// vector and font behaviour of the Windows document engine directly.
//
// Usage: node packaging/windows-diagram-probe.mjs <out.html>

import { readFileSync, writeFileSync } from "node:fs";

const source = readFileSync(
  new URL("../ui/js/graph/diagram-font.js", import.meta.url),
  "utf8",
);
const match = /export const DIAGRAM_FONT = "([^"]+)";/.exec(source);
if (!match) throw new Error("DIAGRAM_FONT was not found in diagram-font.js");
const font = match[1];

const svg =
  '<svg xmlns="http://www.w3.org/2000/svg" width="800" height="500" viewBox="0 0 800 500">' +
  `<style>@font-face{font-family:"InterDiagram";src:url("${font}") format("woff2");` +
  "font-weight:100 900;font-style:normal;}</style>" +
  '<rect width="800" height="500" fill="rgb(244,249,255)"/>' +
  '<path d="M 40 430 L 760 430" fill="none" stroke="rgb(137,157,191)" stroke-width="2"/>' +
  '<path d="M 60 80 L 520 400" fill="none" stroke="rgb(151,73,0)" stroke-width="8"/>' +
  '<path d="M 60 80 Q 620 120 740 410" fill="none" stroke="rgb(87,153,213)" stroke-width="4"/>' +
  '<text x="60" y="470" font-family="InterDiagram" font-size="18" fill="rgb(21,29,39)">' +
  "R = 21.21 m \u00b7 45\u00b0</text>" +
  "</svg>";

const url = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
const html =
  "<!doctype html><html><head><meta charset=\"utf-8\">" +
  "<style>@page{size:A4;margin:20mm}body{margin:0}img{width:100%}</style>" +
  `</head><body><img src="${url}" alt="diagram probe"></body></html>`;

const out = process.argv[2];
if (!out) throw new Error("usage: node windows-diagram-probe.mjs <out.html>");
writeFileSync(out, html);
console.log(`wrote ${out}`);
