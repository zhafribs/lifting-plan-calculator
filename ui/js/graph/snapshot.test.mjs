import test from "node:test";
import assert from "node:assert/strict";

import {
  applyDiagramState,
  captureDiagramState,
  captureWorkingDiagram,
  registerDiagramState,
  registerGraphCapture,
} from "./snapshot.js";
import { DEFAULT_STATE } from "./geometry.js";

test("no registered renderer returns null", () => {
  registerGraphCapture(null);
  assert.equal(captureWorkingDiagram({ scale: 2 }), null);
});

test("capture options reach the registered renderer", () => {
  registerGraphCapture((options) => `capture:${options.scale}`);
  assert.equal(captureWorkingDiagram({ scale: 3 }), "capture:3");
  registerGraphCapture(null);
});

test("the diagram state defaults before the graph is built", () => {
  assert.deepEqual(captureDiagramState(), { ...DEFAULT_STATE });
});

test("a plan opened before the graph is built is held", () => {
  applyDiagramState({ ...DEFAULT_STATE, config: "boom_jib", angle: 30 });
  assert.equal(captureDiagramState().angle, 30);
  assert.equal(captureDiagramState().config, "boom_jib");
});

test("registering the graph consumes the held state", () => {
  let written = null;
  let state = { ...DEFAULT_STATE };
  registerDiagramState(
    () => ({ ...state }),
    (next) => {
      written = next;
      state = { ...DEFAULT_STATE, ...next };
    },
  );
  assert.equal(written.angle, 30);
  assert.equal(captureDiagramState().angle, 30);
});

test("later state changes reach the graph", () => {
  let state = { ...DEFAULT_STATE };
  registerDiagramState(
    () => ({ ...state }),
    (next) => {
      state = { ...state, ...next };
    },
  );
  applyDiagramState({ angle: 60 });
  assert.equal(captureDiagramState().angle, 60);
});
