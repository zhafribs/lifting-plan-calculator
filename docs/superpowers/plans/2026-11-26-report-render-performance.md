# Report Render Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop rebuilding the report and re-rasterising the working-range diagram when nothing relevant changed, keep the latest edit when builds overlap, and capture the live preview at 2× while Page view and Save/Print stay at 3×.

**Architecture:** Put a small LRU cache in front of the diagram renderer, keyed by the graph's own state and the requested size/scale. Extract a pure report-build key and a single-flight queue, then wire the Summary screen to them: it skips identical builds, reruns once when an edit arrives during a build, and requests `preview` or `export` quality depending on the consumer. Timing goes to the existing `window.uilog`.

**Tech Stack:** Plain ES modules, Node's built-in test runner, Tauri IPC, Rust report builder (unchanged).

**Spec:** `docs/superpowers/specs/2026-11-26-report-render-performance.md`

## Global Constraints

- No new dependency, npm build step or CSP change.
- The exported diagram remains 2700×2100 (900×700 at 3×).
- The report content, CSS, footer, margins and arithmetic do not change.
- The live preview frame may use a 2× diagram; Page view, Save as PDF, Save as HTML and Print use 3×.
- Cache keys include every input that changes the report: state, include, issuedAt, signoff, footer, margins, page breaks, diagram scale, diagram image and quality.
- Keep the existing 250 ms preview and 900 ms page-view debounces.
- Cache memory is bounded: at most two diagram images (preview + export) and one report HTML.

## Review Focus

- An edit arrives while `build_report` is running: the final HTML must reflect the latest state, not the build that started first.
- Diagram inputs change but the rest of the report does not: the cache must miss and the new diagram must appear.
- The report text changes but the diagram does not: the diagram cache must hit, and no canvas allocation or PNG encode may occur.
- Switching Live → Pages → Save: all printed paths must use the 3× diagram, even if the live frame was built at 2×.
- Short, repeated edits: the single-flight queue must not build every keystroke or leave a stale HTML.

---

### Task 1: Diagram capture cache

**Files:**
- Create: `ui/js/graph/capture-cache.js`
- Create: `ui/js/graph/capture-cache.test.mjs`
- Create: `ui/js/graph/snapshot.test.mjs`
- Modify: `ui/js/graph/snapshot.js`
- Modify: `ui/js/screens/graph.js` (capture registration and `renderDiagram`)

**Interfaces:**
- Produces: `captureKey(state: object, width: number, height: number, scale: number) -> string`
- Produces: `createCaptureCache(limit?: number) -> { get(key): string|null, set(key, value): void, clear(): void, size: number }`
- Produces: `captureWorkingDiagram(options?: { width?: number, height?: number, scale?: number }) -> string|null`
- `registerGraphCapture(fn: (options?) => string|null)` stays the registration entry point.

- [ ] **Step 1: Write the failing capture-cache tests**

Create `ui/js/graph/capture-cache.test.mjs`:

```js
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
  assert.notEqual(captureKey(state, 900, 700, 3), captureKey({ ...state, angle: 46 }, 900, 700, 3));
  assert.notEqual(captureKey(state, 900, 700, 3), captureKey(state, 900, 700, 2));
  assert.notEqual(captureKey(state, 900, 700, 3), captureKey(state, 800, 700, 3));
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
```

- [ ] **Step 2: Run the test and confirm it fails**

Run: `node --test ui/js/graph/capture-cache.test.mjs`  
Expected: FAIL because `./capture-cache.js` does not exist.

- [ ] **Step 3: Implement `captureKey` and `createCaptureCache`**

Create `ui/js/graph/capture-cache.js`:

- `captureKey` sorts `Object.keys(state)`, joins `key=value` with `|`, and prefixes
  `${width}x${height}@${scale}:`.
- `createCaptureCache(limit = 2)` keeps a `Map`; `get` re-inserts a hit so
  iteration order is LRU; `set` evicts the first key while `size > limit`.
- `size` is a getter; `clear` empties the map.

- [ ] **Step 4: Run the cache tests**

Run: `node --test ui/js/graph/capture-cache.test.mjs`  
Expected: PASS (3 tests).

- [ ] **Step 5: Forward capture options and add the cache to the graph**

In `ui/js/graph/snapshot.js`:

```js
export function captureWorkingDiagram(options) {
  return capture ? capture(options) : null;
}
```

Add `ui/js/graph/snapshot.test.mjs`:

```js
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
```

In `ui/js/screens/graph.js`:

- Import `captureKey, createCaptureCache` from `../graph/capture-cache.js`.
- Create one cache: `const captureCache = createCaptureCache(2);`.
- Change `renderDiagram(width, height)` to `renderDiagram(width, height, scale = 3)`.
- After `syncFromWorkbook(false)`, compute `const key = captureKey(state, width, height, scale);`
  and `const cached = captureCache.get(key); if (cached) return cached;`.
- Replace the fixed `const scale = 3;` with the parameter.
- After `const url = off.toDataURL("image/png");` store `captureCache.set(key, url);`
  and return `url`.
- Register with options:
  `registerGraphCapture(({ width = 900, height = 700, scale = 3 } = {}) => renderDiagram(width, height, scale));`

- [ ] **Step 6: Run the frontend tests and syntax checks**

Run: `node --test ui/js/graph/capture-cache.test.mjs ui/js/graph/snapshot.test.mjs`  
Run: `node --check ui/js/screens/graph.js`  
Expected: PASS; no syntax error.

- [ ] **Step 7: Commit**

```bash
git add ui/js/graph/capture-cache.js ui/js/graph/capture-cache.test.mjs \
  ui/js/graph/snapshot.js ui/js/graph/snapshot.test.mjs ui/js/screens/graph.js
git commit -m "perf(report): cache the working-range diagram capture"
```

---

### Task 2: Report build key and single-flight queue

**Files:**
- Create: `ui/js/report-build.js`
- Create: `ui/js/report-build.test.mjs`

**Interfaces:**
- Produces: `buildReportKey(inputs: object) -> string` — canonical JSON with object keys sorted recursively.
- Produces: `createSingleFlight(task: (args) => Promise<T>) -> (args) => Promise<T>` — one task at a time; a call during a run replaces the pending args and the task reruns once with the latest args. The returned promise resolves to the final run's result.

- [ ] **Step 1: Write the failing tests**

Create `ui/js/report-build.test.mjs`:

```js
import test from "node:test";
import assert from "node:assert/strict";
import { buildReportKey, createSingleFlight } from "./report-build.js";

test("the build key ignores object key order but not value changes", () => {
  const a = { state: { angle: 45, boom: 30 }, include: ["crane"], quality: "preview" };
  const b = { quality: "preview", include: ["crane"], state: { boom: 30, angle: 45 } };
  assert.equal(buildReportKey(a), buildReportKey(b));
  assert.notEqual(buildReportKey(a), buildReportKey({ ...a, quality: "export" }));
  assert.notEqual(buildReportKey(a), buildReportKey({ ...a, include: ["crane", "uniform"] }));
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
```

- [ ] **Step 2: Run the tests and confirm they fail**

Run: `node --test ui/js/report-build.test.mjs`  
Expected: FAIL because `./report-build.js` does not exist.

- [ ] **Step 3: Implement the pure module**

Create `ui/js/report-build.js`:

- `canonical(value)` recursively sorts object keys; arrays keep their order;
  primitives pass through.
- `buildReportKey(inputs)` returns `JSON.stringify(canonical(inputs))`.
- `createSingleFlight(task)` keeps `running` and `pending`; while running, a new
  call stores its args in `pending` and returns the running promise; the loop
  calls `task` with `pending` until no pending args remain; `finally` clears
  `running`. On rejection, clear `pending` so the next call starts clean.

- [ ] **Step 4: Run the tests**

Run: `node --test ui/js/report-build.test.mjs`  
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add ui/js/report-build.js ui/js/report-build.test.mjs
git commit -m "perf(report): add a build key and single-flight queue"
```

---

### Task 3: Wire the Summary screen to the cache and quality split

**Files:**
- Modify: `ui/js/screens/report.js`
- Modify: `ui/js/screens/graph.js` (already returns the requested scale in Task 1)

**Interfaces:**
- Consumes: `captureWorkingDiagram({ scale, width, height })`, `buildReportKey(inputs)`, `createSingleFlight(task)`.
- Produces: internal `buildNow(quality)`, `rebuild(quality)`, `ensureHtml(quality)`; `quality` is `"preview" | "export"`.

- [ ] **Step 1: Import the new helpers and add the timing log**

At the top of `ui/js/screens/report.js`:

```js
import { buildReportKey, createSingleFlight } from "../report-build.js";
```

Add a small helper near the report state:

```js
function timed(label, started, detail = "") {
  const ms = Math.round(performance.now() - started);
  window.uilog && window.uilog(`report ${label}: ${ms}ms${detail ? ` ${detail}` : ""}`);
  return ms;
}
```

- [ ] **Step 2: Make `buildNow` quality-aware and memoised**

Replace `buildNow()` so it:

- accepts `quality` (`"preview"` default, `"export"` for print);
- captures the diagram with `scale: quality === "export" ? 3 : 2`;
- builds `inputs = { state: store.state, include, issuedAt, signoff, footerText, margins, pageBreaks: [...pageBreaks], diagramScale, diagram, quality }`;
- computes `const key = buildReportKey(inputs);`
- returns `lastHtml` immediately when `key === lastKey`;
- otherwise calls `api.buildReport(...)` as today, stores `lastKey = key`,
  `lastHtml = html`, `lastQuality = quality`, and logs `capture`/`build` timings.
- declare `let lastQuality = ""` beside `lastHtml`.
- set `frame.srcdoc` for both qualities: the report opens on Pages, so an export
  build must also leave the Live frame populated.

Keep the existing `srcdoc` substitution and error message.

- [ ] **Step 3: Route rebuilds through the single-flight queue**

Replace the `building` promise logic:

```js
const build = createSingleFlight(buildNow);
function rebuild(quality = "preview") {
  return build(quality);
}
async function ensureHtml(quality = "preview") {
  if (!lastHtml || lastQuality !== quality) await rebuild(quality);
  return lastHtml;
}
```

Set `lastQuality` when a build completes. `schedulePreview` calls `rebuild("preview")`.
`refreshPdf`, Save as HTML, Save as PDF and Print call `ensureHtml("export")`.
Opening in a browser uses `ensureHtml("export")` so the saved file has the print diagram.

- [ ] **Step 4: Keep the live frame and page view independent**

- `schedulePreview` builds the preview only while Live is the visible mode; in
  Pages it schedules the export build instead, so a hidden frame is not rebuilt.
- `refreshPdf` uses `ensureHtml("export")` and the PDF bytes.
- Switching to Pages without an edit builds the export quality once; switching back
  to Live rebuilds the preview if the last build was export quality.

- [ ] **Step 5: Verify the wiring**

Run: `node --check ui/js/screens/report.js`  
Run: `node --test ui/js/graph/capture-cache.test.mjs ui/js/graph/snapshot.test.mjs ui/js/report-build.test.mjs`  
Expected: all PASS.

- [ ] **Step 6: Commit**

```bash
git add ui/js/screens/report.js
git commit -m "perf(report): memoise builds and split preview/export quality"
```

---

### Task 4: Measurement and manual verification

**Files:**
- No production files; verification only.

- [ ] **Step 1: Confirm the test suites**

Run: `node --test ui/js/graph/capture-cache.test.mjs ui/js/graph/snapshot.test.mjs ui/js/report-build.test.mjs`  
Run: `cargo test --all-targets`  
Run: `cargo clippy --all-targets -- -D warnings`  
Expected: all pass.

- [ ] **Step 1a: Add the new tests to CI**

In `.github/workflows/ci.yml`, extend the “Test the frontend's pure modules”
step: copy `ui/js/graph/capture-cache.js`, `capture-cache.test.mjs`,
`snapshot.js`, `snapshot.test.mjs`, `ui/js/report-build.js` and
`report-build.test.mjs` into the temporary work directory, and add the three
test files to the `node --test` command. Keep the existing Windows theme
`js/` + `css/` layout intact.

Run the same `node --test` command locally in a clean temp directory.
Expected: PASS.

- [ ] **Step 2: Measure unchanged typing**

Run the app, open Summary, rename a sign-off field repeatedly.  
Read `~/.cache/lifting-plan-calculator/ui.log`.

Expected: the first edit logs a diagram capture and a build; subsequent edits
with the same diagram log a cache hit (capture well under 5 ms) and no
`build_report` IPC until a report input actually changes.

- [ ] **Step 3: Verify the latest edit wins**

Type quickly while the first build is running (simulate with a large plan or a
slow machine). Expected: the final frame shows the last text; no stale field.

- [ ] **Step 4: Verify 2×/3× split**

Open Pages after a live preview. Expected: the page view is sharp, the PDF has
the same 3× diagram as before, and Save as PDF does not re-raster at 2×.
Compare a saved PDF's size against the prior version; it should be unchanged or
smaller, never larger.

- [ ] **Step 5: Report the measurements**

Record the before/after capture, build and preview times in the PR/commit
message. Do not claim a speed-up without a log line.

- [ ] **Step 6: Three-seat review**

Run the repository's review gate from `AGENTS.md`:

- **Safety & QA** — the `cargo test --all-targets` and Node test output, plus the
  report/PDF comparison; no figure, verdict or report content changed.
- **Security & Build** — the CI test-list change, no new dependency/network/file
  write, and the cache is bounded.
- **Design & Steward** — the new module boundaries, names and comments, and the
  plan's honesty about what is deferred.

Each seat scores out of 10 with a cited file, line or command; every seat must
reach 8.0 before the branch lands.

---

## Self-Review

- **Spec coverage:** diagram cache (Task 1), build key/queue (Task 2), quality
  split/memoisation (Task 3), measurement and acceptance (Task 4).
- **Step scan:** each code step names the signature and the test; no “handle edge
  cases” step.
- **Type consistency:** `captureWorkingDiagram({ scale })`, `captureKey`,
  `createCaptureCache`, `buildReportKey`, `createSingleFlight`, `ensureHtml(quality)`.
- **Review Focus:** stale builds, diagram invalidation, cache hit on text-only
  edits, Live→Pages→Save quality, rapid edits.
- **Proportion:** the plan covers one subsystem (report rendering); vector
  diagram, lazy screens and the design pass are explicitly follow-ups.
