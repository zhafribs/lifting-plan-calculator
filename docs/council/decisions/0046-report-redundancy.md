# 0046 — Removing two repeated lines from the report

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `src-tauri/src/report_outrigger.rs`, `src-tauri/src/report.rs`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Two printed lines repeated what the line above them already said:

1. The square pad printed `Side = √A = 1.396 m` and then
   `Minimum pad: 1.40 × 1.40 m, rounded up from 1.396 m.`
2. The plan-details usage cell printed the figure and band, then
   `of chart capacity` underneath.

## What changed

- The square-pad equation now carries both figures on one line with its
  label: `Minimum square pad  Side = √A = 1.396 m ≈ 1.40 m`. The `Minimum pad`
  sentence is gone. The formula keeps `√A`: the side of a square is the square
  root of its area.
- The plan-details usage sub-line drops `of chart capacity`; the caution case
  keeps `past the 75% target`, and an empty sub-line no longer renders a blank
  row element.

## Evidence

- `the_pad_section_prints_the_formula_the_minimum_square_and_the_thickness`
  now asserts the `≈ 1.40` line and the absence of `Minimum pad:`.
- `the_ground_text_comes_after_the_pad_result` anchors on the
  `Minimum square pad` subheading.
- `the_usage_card_bands_follow_the_chart_usage` still passes for caution
  (`past the 75% target`) and over (`over chart capacity`).
- `cargo test --all-targets` 268 passed; clippy clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the figures are unchanged and both are printed; the correct `√A` form is kept |
| Security & Build | 8/10 | report strings only; no engine path touched |
| Design & Steward | 8/10 | two duplicate lines removed, the working above each keeps the meaning |

**Mean 8.0/10** — at the bar.

## Known limits

- The over-limit state still prints `over chart capacity`; it was not part of
  the request and carries a different meaning from the removed phrase.
- Older printed plans keep the removed lines.
