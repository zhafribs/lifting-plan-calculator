import test from "node:test";
import assert from "node:assert/strict";

import { createScreenRegistry } from "./screen-registry.js";

test("a factory is not called until its screen is requested", () => {
  const calls = [];
  const screens = createScreenRegistry({
    overall: () => (calls.push("overall"), { root: "overall" }),
    crane: () => (calls.push("crane"), { root: "crane" }),
  });
  assert.deepEqual(calls, []);
  assert.equal(screens.has("crane"), true);
  assert.equal(screens.get("crane").root, "crane");
  assert.deepEqual(calls, ["crane"]);
});

test("the first instance is cached", () => {
  let calls = 0;
  const screens = createScreenRegistry({
    overall: () => {
      calls += 1;
      return { root: "overall" };
    },
  });
  assert.equal(screens.get("overall"), screens.get("overall"));
  assert.equal(calls, 1);
});

test("an unknown or inherited key is null and calls no factory", () => {
  let calls = 0;
  const screens = createScreenRegistry({
    overall: () => {
      calls += 1;
      return {};
    },
  });
  for (const key of ["missing", "toString", "constructor", "__proto__"]) {
    assert.equal(screens.get(key), null, key);
    assert.equal(screens.has(key), false, key);
  }
  assert.equal(calls, 0);
});

test("a failed factory is not cached and can be retried", () => {
  let calls = 0;
  const screens = createScreenRegistry({
    overall: () => {
      calls += 1;
      if (calls === 1) throw new Error("boom");
      return { root: "overall" };
    },
  });
  assert.throws(() => screens.get("overall"), /boom/);
  assert.equal(screens.get("overall").root, "overall");
  assert.equal(calls, 2);
});
