# 0063 — Report render cache, lazy screens, and two advisories

**Status:** accepted
**Date:** 2026-11-26
**Release:** v2.17.16
**Change:** `ui/js/report-build.js`, `ui/js/graph/capture-cache.js`,
`ui/js/graph/snapshot.js`, `ui/js/screens/graph.js`, `ui/js/screens/report.js`,
`ui/js/screen-registry.js`, `ui/js/app.js`, CI/docs, `src-tauri/Cargo.lock`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Two performance changes and two lockfile fixes:

- Cache the working-range diagram by crane state and quality; skip the report
  build when its key is unchanged; let the latest edit win when builds overlap;
  capture the live preview at 2× and print at 3×.
- Build screens on first visit behind a small registry. Summary asks for the
  Graph screen first because Graph registers the capture, so the diagram still
  works when Summary is opened before Graph.
- Move `cookie` to 0.18.3 for RUSTSEC-2026-0594 and `url` to 2.5.9 for
  RUSTSEC-2026-0592. Neither is a code change.

## What the council saw

- **Safety & QA (9.0):** the first export capture fell from a repeated cost to a
  cache hit; edits in the default Pages view now reach the page and Save/Print;
  the export-quality race is pinned by a test; lazy screens preserve state,
  sections and the Graph-before-Report order. No arithmetic or report content
  changed; `cargo test --all-targets` 282 pass.
- **Security & Build (9.0, then 9.5 for the advisory patch):** no new
  dependency, network call, file write, permission or CSP change; the diagram
  cache holds at most two images and the screen registry is in memory and
  bounded by the ten literal keys. `cargo audit --deny warnings` exits 0 after
  the bump; the old lock fails on the same advisory.
- **Design & Steward (8.5):** the registry and build-key modules have one job,
  the Report→Graph exception is documented in `docs/architecture.md`, inherited
  keys (`toString`, `__proto__`) are tested, and the measurement is recorded.
- **Release Manager:** the notes name 2.17.16 and this record; the report and
  arithmetic are unchanged.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 9.0/10 | latest edit wins; Pages/Save current; export diagram guarded |
| Security & Build | 9.5/10 | bounded caches; additive CI; advisory genuinely closed |
| Design & Steward | 8.5/10 | clear boundaries; architecture recipe; measured record |

**Mean 9.0/10** — at the bar.

## Decision

**Accepted.** The report caches its diagram and builds, screens are created on
first visit, and the two lockfile advisories are closed.

## Consequences

- Text-only report edits no longer rasterise the diagram or call `build_report`.
- Startup builds one screen; the first Summary visit builds Graph headlessly.
- Live preview uses 2×; Page view, Save and Print keep 3×.
- The report content, footer, margins and arithmetic are unchanged.

## Known limits

- The first “Show report” or Summary visit pays Graph construction on that
  click (about 2 ms on the development machine).
- A failed screen factory is retried, but still surfaces only through the global
  error path; it has no toast of its own.
- Vector diagram and the design/copy pass remain follow-ups.
