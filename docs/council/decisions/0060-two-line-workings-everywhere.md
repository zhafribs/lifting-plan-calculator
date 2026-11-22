# 0060 — Two-line workings in every sling arrangement

**Status:** accepted
**Date:** 2026-11-22
**Release:** v2.17.13
**Change:** `src-tauri/src/report.rs`, `src-tauri/src/report_uniform.rs`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The two-line split was fixed for the uniform geometry but the nonuniform,
tandem, factor and capacity workings still broke into one step per line or ran
wide. All of them are the same kind of working and should read the same way.

## What changed

- `split_long_equation` moves into `report.rs` as a shared function, with
  `split_long_equations` applying it to a whole HTML fragment.
- The nonuniform, tandem, rigging-factor and capacity sections now pass through
  it, so every arrangement gets the same two-line grouping.
- The split is balanced and the continuation aligns under the first equals.
- A block stays in its half-column unless a line is genuinely wide; then it
  takes the full row. No formula was shortened and no number changed.

## Evidence

- The full fixture report exports as 9 pages at 100% scale; no page shrink.
- Uniform, nonuniform and tandem pages show two-line workings with the tail
  under the first equals.
- `cargo test --all-targets` 281 passed; clippy clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the same arithmetic and literals print; only line breaks and alignment changed |
| Security & Build | 8/10 | a shared formatter and a section pass; no new call, write or dependency |
| Design & Steward | 8/10 | every arrangement reads the same, with wide blocks only where needed |

**Mean 8.0/10** — at the bar.

## Known limits

- A formula too wide for the full row keeps one step per line rather than
  shrinking the page.
