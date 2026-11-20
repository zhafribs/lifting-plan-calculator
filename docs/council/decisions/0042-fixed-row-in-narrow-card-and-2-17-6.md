# 0042 — The fixed row in a narrow card, and 2.17.6

**Status:** accepted
**Date:** 2026-11-20
**Release:** v2.17.6
**Change:** `ui/css/app.css`, `CHANGELOG.md`, `docs/releases/v2.17.6.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

With the rail expanded in a 728px window, the fixed-weight card is about 450px
wide. The two-line row's columns were `1fr 156px 122px 96px` with a 62px
quantity in the first column; the flexible column fell to about 53px, so the
quantity overflowed under the dropdown and "Fixed" read as "ed". Keep the row
on two lines and let the dropdown sit between the quantity and the weight box.

## What changed

Below a 485px card, the fixed-weight row's columns become
`62px minmax(0, 1fr) 122px 96px`. The quantity keeps its full 62px, the
dropdown takes whatever is left between the quantity and the weight box, and
the description still spans the first line. Per-metre and per-volume rows are
excluded by `:not(.two-measure)` and keep their own arrangement; above 485px
nothing changes.

## Evidence

- Built 2.17.6 and reproduced the reported window: 728×699 with the navigation
  expanded (card ≈450px). The row now reads `1 | Fixed | 0 kg | 0.00 kg` with
  "Fixed" whole and no overlap.
- The same window with a per-metre line: `Weight by Metre | kg/m`, then
  `1 | m | 0.00 kg`, unchanged from 2.17.4.
- 1440×950 with the rail expanded: the wide one-line row is unchanged.
- `node --check` on every frontend module, `node --test` 38 passed,
  `cargo test --all-targets` 268 passed (no Rust changed).

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | field order, units and values unchanged; no calculation path touched |
| Security & Build | 8/10 | one container query; no new call, write or dependency |
| Design & Steward | 8/10 | the row stays two lines as asked, the quantity keeps its width and the dropdown gets the remainder |

**Mean 8.0/10** — at the bar.

## Known limits

- Below roughly 360px of card width the dropdown would have too little room;
  the window minimum (720px) with the expanded rail keeps the card above 440px,
  so that case is out of reach.
- No automated visual test covers the grid; the two windows were checked by hand.
