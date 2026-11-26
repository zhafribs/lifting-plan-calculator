import test from "node:test";
import assert from "node:assert/strict";

import { captureKey, createCaptureCache } from "./capture-cache.js";

test("the key is stable across object insertion order", () => {
  const a = { boom_length: 30, angle: 45, crane_x: 0 };
  const b = { crane_x: 0, angle: 45, boom_length: 30 };
  assert.equal(captureKey(a, 900, 700, 3), captureKey(b, 900, 700, 3));
});

test("the key changes with state, size and scale", () => {
  const state = { boom_length: 30, angle: 45 };
  assert.notEqual(
    captureKey(state, 900, 700, 3),
    captureKey({ ...state, angle: 46 }, 900, 700, 3),
  );
  assert.notEqual(
    captureKey(state, 900, 700, 3),
    captureKey(state, 900, 700, 2),
  );
  assert.notEqual(
    captureKey(state, 900, 700, 3),
    captureKey(state, 800, 700, 3),
  );
  assert.notEqual(
    captureKey(state, 900, 700, 3, "svg"),
    captureKey(state, 900, 700, 3, "png"),
  );
});

test("the cache hits, and evicts the least recently used entry", () => {
  const cache = createCaptureCache(2);
  assert.equal(cache.get("a"), null);
  cache.set("a", "A");
  cache.set("b", "B");
  assert.equal(cache.get("a"), "A");
  cache.set("c", "C");
  assert.equal(cache.get("b"), null);
  assert.equal(cache.get("a"), "A");
  assert.equal(cache.get("c"), "C");
  assert.equal(cache.size, 2);
  cache.clear();
  assert.equal(cache.size, 0);
});
