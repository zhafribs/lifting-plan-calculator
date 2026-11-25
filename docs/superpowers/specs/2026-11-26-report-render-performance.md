# Report render performance

**Date:** 2026-11-26

## Problem

The Summary screen rebuilds the whole report — including a 2700×2100 raster of the
working-range diagram — on every typing pause, even when the diagram inputs have
not changed. The build path also drops edits made while a build is running, and
the live preview and the printed page share one image quality.

Evidence (2.17.15, `bbf26ac`):

- `ui/js/screens/report.js` calls `captureWorkingDiagram()` inside every
  `buildNow()` (about line 655) after a 250 ms debounce.
- `ui/js/screens/graph.js:1127` renders a 900×700 canvas at 3×, runs
  `getImageData`/`putImageData` over ~22.7 MB, PNG-encodes it and returns a data
  URL every time.
- `rebuild()` (report.js:682) deduplicates concurrent builds but discards edits
  that arrive during a build; the report can remain one edit stale.
- The same HTML quality is used for the live frame, the page view and Save/Print.

## Goal

Make report preview and PDF export responsive without changing the report's
content, layout, arithmetic, footer or page margins:

- no diagram re-raster and no report build when the report inputs are unchanged;
- edits during a build always produce a report from the latest state;
- live preview captures the diagram at 2×; Page view and Save/Print capture at
  3×, so the exported PDF keeps today's diagram resolution;
- timing is visible in `ui.log` so the next optimisation is measured, not guessed.

## Non-goals

- Vector/SVG diagram output (follow-up).
- Lazy screen creation and startup work (separate plan).
- Visual/copy restraint pass (separate plan).
- Any change to the printed report's content or CSS.

## Constraints

- No new dependency, npm build step or CSP change.
- The exported diagram stays 2700×2100 (3× of 900×700).
- The report and PDF must not change except that the live preview's diagram is
  captured at 2× (it is not printed).
- The cache key must include every input that changes the report: state,
  sections, sign-off, footer, margins, page breaks, diagram scale, diagram image
  and preview/export quality.
- One diagram cache entry per quality is enough; memory stays bounded.
- Existing debounces stay: 250 ms live, 900 ms page view.

## Acceptance

- With the Summary open and the Graph inputs unchanged, typing a report field
  reuses the cached diagram: no new canvas allocation, PNG encode or
  `getImageData`.
- Rebuilding with identical inputs makes no `build_report` IPC call.
- Editing while a build is in flight ends with the latest edit rendered.
- Switching to Pages after an edit prints at 3×; Save/Print also uses 3×.
- `ui.log` records capture, build and preview timings.
- `node --test` and `cargo test` stay green; no report output changes.
