import test from "node:test";
import assert from "node:assert/strict";

import { buildReportKey, createSingleFlight, ensureQuality } from "./report-build.js";

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

test("an export request re-asserts after a preview overwrites it", async () => {
  let state = { html: "", quality: "" };
  let writes = 0;
  const rebuild = async (quality) => {
    writes += 1;
    // The first run is replaced by a preview, as the queue can do.
    state = writes === 1
      ? { html: "preview", quality: "preview" }
      : { html: quality, quality };
  };
  const html = await ensureQuality("export", () => state, rebuild);
  assert.equal(html, "export");
  assert.equal(writes, 2);
});

test("a matching quality is returned without another build", async () => {
  let writes = 0;
  const state = { html: "preview", quality: "preview" };
  const html = await ensureQuality("preview", () => state, async () => {
    writes += 1;
  });
  assert.equal(html, "preview");
  assert.equal(writes, 0);
});

test("a quality that never sticks throws instead of returning the wrong build", async () => {
  let writes = 0;
  const state = { html: "preview", quality: "preview" };
  const rebuild = async () => {
    writes += 1;
    state.html = "preview";
    state.quality = "preview";
  };
  await assert.rejects(
    ensureQuality("export", () => ({ ...state }), rebuild),
    /still rebuilding/,
  );
  assert.equal(writes, 3);
});
