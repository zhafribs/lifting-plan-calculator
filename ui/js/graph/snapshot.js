// The Graph tab registers an offscreen renderer here. The Summary screen asks
// for the diagram when it rebuilds, so it reflects the current crane and jib
// configuration without needing the Graph tab to be open. Options are forwarded
// to the renderer: the report asks for a smaller preview or a print-quality
// capture.

let capture = null;

export function registerGraphCapture(fn) {
  capture = fn;
}

export function captureWorkingDiagram(options) {
  return capture ? capture(options) : null;
}
