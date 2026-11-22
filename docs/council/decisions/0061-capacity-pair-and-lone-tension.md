# 0061 — The capacity workings pair; the lone tension stays on one line

**Status:** accepted
**Date:** 2026-11-22
**Release:** v2.17.14
**Change:** `src-tauri/src/report_capacity.rs`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Two workings were laid out against their content. The tension working stands
alone, so splitting it wastes a line. Minimum Required Sling WLL and WLL
Utilisation are a pair, so stacking them wastes the second column. Put each
where the row can hold it.

## What changed

- In Rigging Factors & Tension, the lone tension equations are no longer split;
  they take the full row on one line.
- In Final Capacity, each required-WLL block is paired with its utilisation
  block in the working grid. Both are split for the half column, so each reads
  two lines.
- Nested lifts pair the first and second sling the same way (four blocks, two
  rows).
- The tag-rating PASS/FAIL sentence moves into the utilisation cell; the
  configured SWL stays below the grid.
- No formula, number or verdict changed.

## Evidence

- 100% scale in the Pages view with the direct-hook example: section 5 prints
  `Tension = … = 0.0 kg` on one line; section 6 prints Minimum WLL and WLL
  Utilisation side by side, each two lines.
- `cargo test --all-targets` 281 passed; clippy clean. The capacity tests
  compare through the `unaligned` helper, so the reference literals are
  unchanged.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the same arithmetic and literals; only placement changed |
| Security & Build | 8/10 | section layout and a helper signature; no new call or write |
| Design & Steward | 8/10 | each working uses the row it has, and the pair reads as a pair |

**Mean 8.0/10** — at the bar.

## Known limits

- The utilisation cell also carries its PASS/FAIL sentence, so that cell can be
  taller than the required-WLL cell.
