# Lazy Screen Creation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build each tab the first time it is opened, cache it afterwards, and keep Summary's working-range diagram working even when the Graph tab was never opened.

**Architecture:** A pure registry maps screen keys to factories and caches the first successful instance. `app.js` supplies the existing `create*Screen(ctx)` calls as factories; `go()` and `showReport()` ask the registry instead of indexing a prebuilt object. The Report factory first asks for the Graph screen, which registers the offscreen capture, but does not show it.

**Tech Stack:** Plain ES modules, Node's built-in test runner, existing Tauri IPC.

**Spec:** `docs/superpowers/specs/2026-11-26-lazy-screen-creation.md`

## Global Constraints

- No new dependency, npm build step or CSP change.
- No screen's DOM, layout, copy or behaviour changes.
- A factory runs at most once per launch; a failed factory is not cached and may be retried.
- Summary must capture the current diagram even if Graph has never been shown.
- Start-screen flags, `Ctrl+1..8`, the rail and every "Show report" button keep working.
- No persisted screen state between launches.

## Review Focus

- Summary opened before Graph: the diagram must still be present.
- A screen opened later must see the current state and solved result, not the state at launch.
- A factory failure must not poison the registry; the next navigation retries.
- Repeated navigation must reuse the instance and must not rebuild its DOM.
- `showReport()` from another tab must set the report sections before the first report build.

---

### Task 1: Pure screen registry

**Files:**
- Create: `ui/js/screen-registry.js`
- Create: `ui/js/screen-registry.test.mjs`
- Modify: `.github/workflows/ci.yml` (frontend test list)
- Modify: `AGENTS.md` (local test command)

**Interfaces:**
- Produces: `createScreenRegistry(factories: Record<string, () => object>) -> { get(key: string): object|null, has(key: string): boolean, created(): string[] }`
- A factory returns the existing screen object shape: `{ root, sync?, render?, show?, setSections? }`.

- [ ] **Step 1: Write the failing tests**

Create `ui/js/screen-registry.test.mjs`:

```js
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
  assert.deepEqual(screens.created(), ["crane"]);
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

test("an unknown key is null and calls no factory", () => {
  let calls = 0;
  const screens = createScreenRegistry({
    overall: () => {
      calls += 1;
      return {};
    },
  });
  assert.equal(screens.get("missing"), null);
  assert.equal(screens.has("missing"), false);
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
  assert.deepEqual(screens.created(), ["overall"]);
});
```

- [ ] **Step 2: Run the tests and confirm they fail**

Run: `node --test ui/js/screen-registry.test.mjs`  
Expected: FAIL because `./screen-registry.js` does not exist.

- [ ] **Step 3: Implement the registry**

Create `ui/js/screen-registry.js`:

- `has(key)` uses `Object.hasOwn(factories, key)`.
- `get(key)` returns null when absent, otherwise creates once and caches the
  successful result. A throw propagates and leaves the cache untouched.
- `created()` returns the cached keys in creation order.

- [ ] **Step 4: Run the tests**

Run: `node --test ui/js/screen-registry.test.mjs`  
Expected: PASS (4 tests).

- [ ] **Step 5: Add the test to CI and the local command**

In `.github/workflows/ci.yml`, copy `ui/js/screen-registry.js` and
`screen-registry.test.mjs` into `$work` and add `$work/screen-registry.test.mjs`
to the `node --test` list. Add the same file to the frontend test command in
`AGENTS.md`.

- [ ] **Step 6: Run the CI-shaped test set**

Run the `node --test` command from `.github/workflows/ci.yml` in a clean temp
directory. Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add ui/js/screen-registry.js ui/js/screen-registry.test.mjs \
  .github/workflows/ci.yml AGENTS.md
git commit -m "perf(ui): add a lazy screen registry"
```

---

### Task 2: Wire `app.js` to the registry

**Files:**
- Modify: `ui/js/app.js` (imports, screen construction, `go`, `showReport`, start-screen selection)

**Interfaces:**
- Consumes: `createScreenRegistry` from Task 1.
- Produces: internal `screen(key, create)` logging wrapper; `screens.get(key)` in `go` and `showReport`.

- [ ] **Step 1: Replace the eager object with factories**

In `ui/js/app.js`, import `createScreenRegistry`. Replace the
`screens = { overall: createOverallScreen(ctx), ... }` object with:

```js
function screen(key, create) {
  return () => {
    const started = performance.now();
    const instance = create();
    window.uilog &&
      window.uilog(`screen ${key}: ${Math.round(performance.now() - started)}ms`);
    return instance;
  };
}

const screens = createScreenRegistry({
  overall: screen("overall", () => createOverallScreen(ctx)),
  crane: screen("crane", () => createCraneScreen(ctx)),
  outrigger: screen("outrigger", () => createOutriggerScreen(ctx)),
  graph: screen("graph", () => createGraphScreen(ctx)),
  uniform: screen("uniform", () => createUniformScreen(ctx)),
  nonuniform: screen("nonuniform", () => createNonuniformScreen(ctx)),
  tandem: screen("tandem", () => createTandemScreen(ctx)),
  report: screen("report", () => {
    // The Graph screen registers the offscreen renderer Summary captures.
    // Build it without showing it, so the diagram exists on a first visit.
    screens.get("graph");
    return createReportScreen(ctx);
  }),
  settings: screen("settings", () => createSettingsScreen(ctx)),
  about: screen("about", () => createAboutScreen(ctx)),
});
```

- [ ] **Step 2: Use `get`/`has` at every call site**

- `go(key)`: `const next = screens.get(key); if (!next) return;` then use `next`
  for `current`.
- `showReport(sections)`: `const report = screens.get("report");` call
  `report.setSections` when present, then `go("report")`.
- Start-screen selection: `screens.has(info.start_screen) ? info.start_screen : "overall"`.
- No other `screens[...]` or `screens.<key>` references remain.

- [ ] **Step 3: Check syntax and the registry tests**

Run: `node --check ui/js/app.js`  
Run: `node --test ui/js/screen-registry.test.mjs`  
Expected: both pass.

- [ ] **Step 4: Commit**

```bash
git add ui/js/app.js
git commit -m "perf(ui): create screens on first visit"
```

---

### Task 3: Verify every entry path

**Files:**
- No production changes unless a test exposes a defect.

- [ ] **Step 1: Confirm the default launch is lazy**

Run the app normally, then read `~/.cache/lifting-plan-calculator/ui.log`.

Expected: one `screen overall: Nms` line; no crane, graph, report or settings
construction.

- [ ] **Step 2: Visit every tab once**

Open Overall, Crane, Outrigger, Graph, Uniform, Nonuniform, Tandem, Summary,
Settings and About. Return to two of them.

Expected: one `screen <key>` line per tab, no second line on return. No tab is
blank and each shows current data.

- [ ] **Step 3: Open Summary before Graph**

Use `cargo run -- --screen=report`.

Expected: `screen graph` is created before `screen report`; the page view
contains the working-range diagram when the diagram box is ticked.

- [ ] **Step 4: Check the cross-tab report path**

From Crane, Outrigger, Uniform, Nonuniform and Tandem, press **Show report**.

Expected: Summary opens with the requested sections; the diagram is present;
no blank or duplicate screen.

- [ ] **Step 5: Check keyboard and start-screen flags**

Use `Ctrl+1`…`Ctrl+8`, the rail, and two `--screen=` values (for example
`--screen=uniform` and `--screen=about`). Expected: the named screen appears;
unknown values fall back to Overall.

- [ ] **Step 6: Run the full gates**

Run: `node --test` with the CI list from `.github/workflows/ci.yml`  
Run: `cd src-tauri && cargo test --all-targets && cargo clippy --all-targets -- -D warnings`  
Run: the `node --check` loop from `AGENTS.md`.  
Expected: all pass.

- [ ] **Step 7: Measure startup**

Compare the first `app main start` to first-tab timing before and after, using
the `screen <key>: Nms` line. Record the result in the PR/commit body; do not
claim a win without a log line.

- [ ] **Step 8: Three-seat review**

Run the gate from `AGENTS.md`:

- **Safety & QA:** the first-visit state, Summary-before-Graph, Show report and
  factory-failure retry; no report content change.
- **Security & Build:** no new dependency/network/file write; the registry
  caches only in memory; CI test list is additive.
- **Design & Steward:** the registry boundary, names/comments, no dead eager
  code, honesty about the Graph dependency.

Each seat scores out of 10 with cited evidence; every seat must reach 8.0.

---

## Self-Review

- **Spec coverage:** lazy construction (Tasks 1–2), Summary capture dependency
  (Task 2), every entry path and measurement (Task 3).
- **Step scan:** the registry has exact tests; the wiring names each call site;
  verification names the command and expected log.
- **Type consistency:** `createScreenRegistry`, `get`, `has`, `created`, `screen`;
  the existing screen object shape is unchanged.
- **Review Focus:** covered by Task 3 steps 2–5 and the three seat prompts.
- **Proportion:** one subsystem, three tasks; no screen internals are touched.
