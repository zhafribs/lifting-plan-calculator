# 0051 — A typed diagram scale

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `ui/js/screens/report.js`, `ui/css/app.css`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The scale was a 50/75/100% picker. The operator asked to type the figure and
keep the diagram's ratio.

## What changed

- The picker is a number field with a `%` suffix, default 100, disabled while
  the diagram is unticked.
- Input is clamped to 25–100 while typing and normalised on blur, so the field
  shows the value the report used.
- The report sets only the image's width; its height stays auto, so the diagram
  keeps its own aspect ratio at every scale. The old `max-height: 78mm` on the
  image pinned the height and made the scale horizontal-only; it is gone.

## Evidence

- The Summary screen at 1440×950 shows `Scale [100 %]` beside the diagram tick,
  dimmed until the diagram is enabled.
- `the_diagram_scale_reaches_the_image` covers the width reaching the report.
- `node --check` on the screen; 272 Rust tests pass.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | no figure or calculation touched |
| Security & Build | 8/10 | one input; the report clamps the value again |
| Design & Steward | 8/10 | a typed scale reads like the rest of the plan-detail fields and normalises its own value |

**Mean 8.0/10** — at the bar.

## Known limits

- The field accepts only 25–100; larger would exceed the printed column.
- The ratio is preserved by width-only sizing, not by a visible lock control.
