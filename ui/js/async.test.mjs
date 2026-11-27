import test from "node:test";
import assert from "node:assert/strict";

import { withTimeout } from "./async.js";

test("resolves a promise that beats the deadline", async () => {
  assert.equal(await withTimeout(Promise.resolve(7), 100, "late"), 7);
});

test("passes a rejection through", async () => {
  await assert.rejects(withTimeout(Promise.reject(new Error("boom")), 100, "late"), /boom/);
});

test("rejects a promise that outlasts the deadline", async () => {
  await assert.rejects(withTimeout(new Promise(() => {}), 5, "late"), /late/);
});

test("a settled promise leaves no timer holding the process", async () => {
  await withTimeout(Promise.resolve(1), 5000, "late");
  // No assertion beyond returning: a live timer would keep the test process up.
});
