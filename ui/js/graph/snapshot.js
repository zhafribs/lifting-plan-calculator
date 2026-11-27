// The Graph tab registers an offscreen renderer and its drawing state here. The
// Summary screen asks for the diagram when it rebuilds, and a saved plan carries
// the state the diagram was drawn from, so opening it reproduces the picture.

import { DEFAULT_STATE } from "./geometry.js";

let capture = null;
let readState = null;
let writeState = null;
// A plan can be opened before the Graph tab has ever been built; hold its state
// until the screen registers.
let pending = null;

export function registerGraphCapture(fn) {
  capture = fn;
}

export function registerDiagramState(read, write) {
  readState = read;
  writeState = write;
  if (pending) {
    write(pending);
    pending = null;
  }
}

export function captureDiagramState() {
  if (readState) return readState();
  return pending ? { ...pending } : { ...DEFAULT_STATE };
}

export function applyDiagramState(next) {
  if (!next) return;
  if (writeState) writeState(next);
  else pending = { ...next };
}

export function captureWorkingDiagram(options) {
  return capture ? capture(options) : null;
}
