# 0053 — The report scale across sling arrangements

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `src-tauri/src/report_uniform.rs`, `src-tauri/src/report.rs`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Two PDFs of the same plan, one from a direct hook and one from a 2-leg round
choke, had different font sizes. The text and the text box were identical; only
the scale differed. The cause was not the arrangement but a formula wider than
its half-column: WebKit shrank the whole printed page to fit it.

## What changed

- `split_long_equation` breaks a display equation at its top-level equals signs
  into an `aligned` block. The math is unchanged; only line breaks are added.
- `geometry_grid` marks a block whose (split) formula is still long, and
  `working_grid` prints it across both columns instead of half a page.
- The geometry tests compare against the reference one-line strings through an
  `unaligned` helper, so the expected literals stay exactly as they were.

## Evidence

The same plan (1,580 kg, crane example, outrigger example) was rendered for five
uniform arrangements through the app's deterministic PDF path, and the
plan-details title measured:

| Arrangement | Before | After |
| --- | --- | --- |
| Direct hook | 14 | 14 |
| 2-leg round choke | 12 | 14 |
| Nested 3-leg | 9 | 14 |
| Four-leg direct | 13 | 14 |
| Round choke | 13 | 14 |

The nested geometry page was rendered and checked: split formulas are readable,
over-wide blocks span the row, and no formula is clipped.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | every formula is byte-identical once the alignment markup is removed; the tests still compare the reference strings |
| Security & Build | 8/10 | report layout only |
| Design & Steward | 8/10 | all uniform arrangements now share one scale, and the geometry stays two columns where it fits |

**Mean 8.0/10** — at the bar.

## Known limits

- The full fixture report (which also carries the non-uniform and tandem
  sections) measures 13, not 14: those sections have not been through the same
  formula check. A plan that ticks them can still shrink slightly; the same
  treatment should be applied there next.
- The system print dialog's own scale still applies when the operator changes
  it.
