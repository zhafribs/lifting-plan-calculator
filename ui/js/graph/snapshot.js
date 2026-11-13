// The Graph tab registers an offscreen renderer here. The Summary screen asks
// for a fresh diagram on every report build, so it reflects the current crane
// and jib configuration without needing the Graph tab to be open.

let capture = null;

export function registerGraphCapture(fn) {
  capture = fn;
}

export function captureWorkingDiagram() {
  return capture ? capture() : null;
}
