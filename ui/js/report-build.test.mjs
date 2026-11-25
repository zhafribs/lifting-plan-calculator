import test from "node:test";
import assert from "node:assert/strict";

import { buildReportKey, createSingleFlight } from "./report-build.js";

test("the build key ignores object key order but not value changes", () => {
  const a = {
    state: { angle: 45, boom: 30 },
    include: ["crane"],
    quality: "preview",
  };
  const b = {
    quality: "preview",
    include: ["crane"],
    state: { boom: 30, angle: 45 },
  };
  assert.equal(buildReportKey(a), buildReportKey(b));
  assert.notEqual(buildReportKey(a), buildReportKey({ ...a, quality: "export" }));
  assert.notEqual(
    buildReportKey(a),
    buildReportKey({ ...a, include: ["crane", "uniform"] }),
  );
});

test("array order is part of the key", () => {
  assert.notEqual(
    buildReportKey({ pageBreaks: ["crane", "uniform"] }),
    buildReportKey({ pageBreaks: ["uniform", "crane"] }),
  );
});

test("a call during a run reruns once with the latest args", async () => {
  const seen = [];
  let release;
  const task = createSingleFlight(async (value) => {
    seen.push(value);
    if (seen.length === 1) await new Promise((resolve) => (release = resolve));
    return value;
  });
  const first = task("a");
  const second = task("b");
  const third = task("c");
  release();
  assert.equal(await first, "c");
  assert.equal(await second, "c");
  assert.equal(await third, "c");
  assert.deepEqual(seen, ["a", "c"]);
});

test("a rejected run clears the queue", async () => {
  let calls = 0;
  const task = createSingleFlight(async () => {
    calls += 1;
    if (calls === 1) throw new Error("boom");
    return "ok";
  });
  await assert.rejects(task("a"), /boom/);
  assert.equal(await task("b"), "ok");
});
