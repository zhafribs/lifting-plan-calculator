# 0030 — Name the arrangement; draw every input section the same way

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.11
**Change:** `src-tauri/src/report_nonuniform.rs`, `src-tauri/src/report.rs`
**Seats:** Technical Writer, QA & Test Engineer, Product Manager. The Release
Manager chaired; the implementer did not score.

## Proposal

- The uniform inputs were titled `LIFT INPUT PARAMETERS - UNIFORM LOAD`, but the
  nonuniform and tandem inputs were a bare `Lift Input Parameters` sub-heading
  under a section called just "Nonuniform Load" / "Tandem Lift". All three now
  read `LIFT INPUT PARAMETERS - <ARRANGEMENT>`, including the early-return paths
  (not set up, no load, refused), so the heading is the same whether or not the
  section has data.
- The nonuniform and tandem rows were a two-column key/value table; the uniform
  ones are three-column figure cards (`card_grid`). All three now use
  `card_grid`, so the plan reads the same way whichever arrangement it carries.
- The tandem title passed by the report is "Tandem" rather than "Tandem Lift",
  matching the tab and the new heading.

## What the council saw

- The heading is asserted for both arrangements, including the empty case
  (`4. LIFT INPUT PARAMETERS - TANDEM`), and the uniform case already had one.
- The card markup is asserted by the asymmetric reference test, which now checks
  the `metric-k` / `metric-v` pair instead of `<tr><th>`.
- 263 Rust tests pass; `clippy -D warnings` is clean. The report's own
  renumbering still decides the number, so the title and the number cannot
  disagree.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Technical Writer | 8/10 | the heading says which arrangement the figures belong to, in every state of the section |
| QA & Test Engineer | 8/10 | both the populated and the empty paths are asserted, and the card markup is pinned by the reference test |
| Product Manager | 8/10 | the three arrangements read as one document shape |

**Mean 8.0/10** — at the bar.

## Known limits

- The cards are three across, like the uniform ones; a very long label wraps
  inside its card rather than widening the column.
- The change is presentational only: the rows, their order and their values are
  unchanged.
