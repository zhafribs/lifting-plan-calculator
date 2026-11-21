# 0057 — The working range diagram starts at half width

**Status:** accepted
**Date:** 2026-11-21
**Release:** v2.17.10
**Change:** `ui/js/screens/report.js`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The diagram was printed at full width by default. At that size it pushes the
plan details down and takes a whole page on many plans. Half width still shows
the boom, radius and hook, and leaves room on the page.

## What changed

- The Report sections card opens with the diagram ticked and the scale at 50.
- The scale field is unchanged: 25 to 100, normalised when focus leaves.
- No formula, figure or verdict changed.

## Evidence

- `ui/js/screens/report.js`: the initial `diagramScale`, the field value and the
  blur fallback all read 50.
- Live at 1440×950 and 720×540 with the rail expanded: the diagram previews at
  half width and the field reads `50 %`; typing 100 restores full width.
- `cargo test --all-targets` 277 passed, frontend tests passed, clippy clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | presentation only; no figure or verdict is touched |
| Security & Build | 8/10 | three literals in one module; no new call or write |
| Design & Steward | 8/10 | a smaller default that fits the page, with the existing control still in reach |

**Mean 8.0/10** — at the bar.

## Known limits

- Other callers of the report builder still default to 100%; only the app's
  Report sections card opens at 50.
