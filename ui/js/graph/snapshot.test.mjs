import test from "node:test";
import assert from "node:assert/strict";

import { captureWorkingDiagram, registerGraphCapture } from "./snapshot.js";

test("no registered renderer returns null", () => {
  registerGraphCapture(null);
  assert.equal(captureWorkingDiagram({ scale: 2 }), null);
});

test("capture options reach the registered renderer", () => {
  registerGraphCapture((options) => `capture:${options.scale}`);
  assert.equal(captureWorkingDiagram({ scale: 3 }), "capture:3");
  registerGraphCapture(null);
});
