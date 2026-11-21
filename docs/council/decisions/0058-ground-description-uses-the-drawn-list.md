# 0058 — The ground description list uses the drawn dropdown

**Status:** accepted
**Date:** 2026-11-21
**Release:** v2.17.11
**Change:** `ui/js/components.js`, `ui/js/screens/outrigger.js`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Every picker in the app draws its own list except the Outrigger's soil / rock
description. That one still opened the platform's popup, which renders as a
large detached box under XWayland and does not carry the app's highlight or
keyboard handling. It should behave like the ground category above it.

## What changed

- The dropdown wiring in `selectInput` moves to an exported
  `attachSelectMenu(select)`. `selectInput` calls it, so its behaviour is
  unchanged.
- The Outrigger description `<select>` calls `attachSelectMenu` after it is
  created, so the list is drawn by the app even though its options are rebuilt
  when the category changes.
- No option text, value or formula changed.

## Evidence

- 1920×1080: with Cohesive Soils chosen, the description list opens under the
  field with all five descriptions, the selected row highlighted, and the same
  border and scroll behaviour as every other picker. Escape and the wheel close
  it as before.
- `node --check` on both modules; 277 Rust tests and clippy clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the option list and the selected value are unchanged; only the drawing path moved |
| Security & Build | 8/10 | a wiring extraction in one module; no new call, write or dependency |
| Design & Steward | 8/10 | the last inconsistent picker now matches the rest of the app |

**Mean 8.0/10** — at the bar.

## Known limits

- The list is rebuilt each time the category changes; an open menu closes with
  the old category, which is the same behaviour the control had before.
