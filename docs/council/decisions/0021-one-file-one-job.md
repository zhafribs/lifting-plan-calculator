# 0021 — One file, one job: the first cut

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.2
**Change:** `docs/architecture.md`, `src-tauri/src/report_style.rs`,
`ui/css/tokens.css`, `ui/index.html`
**Seats:** Build & Platform Engineer, QA & Test Engineer, Technical Writer.
The Release Manager chaired; the implementer did not score.

## Proposal

The operator asked for a codebase where each part can be understood on its own,
not one that is small for its own sake. This release starts with the two moves
that are pure extraction, and records the map and the remaining seams instead of
merging anything.

- `report_style.rs` holds the report stylesheet. It is one string because three
  printers read it; keeping it beside the HTML assembly meant editing a 1 500
  line file to change a type size.
- `tokens.css` holds the type face, the measurements and both palettes.
  `app.css` keeps layout and components, and reads only token names.
- `docs/architecture.md` states what each file is for, where a new screen,
  report section, command or update path goes, and where the tests live.

## What the council saw

- Both moves are extraction: the same CSS text, the same const, reachable from
  the same places. The 253 Rust tests and 25 frontend tests pass unchanged.
- The light and dark palettes were photographed in the running app after the
  split, so the cascade order of the two stylesheets is proven, not assumed.
- The remaining seams are named with their order: the diagram screen's drawing
  layer, then the export probe out of `commands.rs`. The engine files are
  deliberately left alone — they are cohesive and heavily tested, and splitting
  them would add indirection for no reader.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | no build or packaging change; the split is import-level only |
| QA & Test Engineer | 8/10 | tests unchanged and green, and the visual check covers the one thing tests cannot: cascade order |
| Technical Writer | 8/10 | the map explains the rule ("one file, one job") and where to add each kind of thing, not just the file list |

**Mean 8.0/10** — at the bar.

## Known limits

- This is the first cut, not the whole restructure: `screens/graph.js` and the
  export probe in `commands.rs` are still to move, in that order.
- A document does not keep a codebase tidy by itself; the rule only holds if new
  code follows the "where to add something" section.
