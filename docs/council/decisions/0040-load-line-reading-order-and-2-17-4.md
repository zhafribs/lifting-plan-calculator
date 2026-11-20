# 0040 — The load line's reading order, and 2.17.4

**Status:** accepted
**Date:** 2026-11-20
**Release:** v2.17.4
**Change:** `ui/css/app.css`, `CHANGELOG.md`, `docs/releases/v2.17.4.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

2.17.3 gave the "Measured by" dropdown a full-width line and put both value
boxes under it. The operator reads the line as weight, length, quantity — so
the dropdown belongs beside the first value box, and the quantity beside the
second. Correct 2.17.3 in its own release; published bytes are not overwritten.

## What changed

Below a 620px card, a per-metre or per-volume line is now:

```
description ........................ remove
[ Weight by Metre ▾ ] [ kg/m ]
[ Qty ]               [ m ]       0.00 kg
```

Per volume the same shape with `Weight by Volume`, `m3` and `kg/m3`. The two
value boxes are placed as separate grid items (`display: contents` on their
cell), the quantity is 62px, and the subtotal closes the row. A fixed-weight
line is untouched, and above 620px the two boxes still stack in the measurement
column.

## Evidence

- Built 2.17.4 and checked at 720×540: per metre and per volume both show the
  dropdown beside the first box with the full label, the quantity beside the
  second, and the subtotal at the end. A fixed line keeps its two-line layout.
- At 1440×950 the wide row is unchanged from 2.17.3: the two boxes stack in the
  122px column.
- `node --check` on every frontend module, `node --test` 38 passed,
  `cargo test --all-targets` 268 passed (no Rust changed).

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | field order and units unchanged; the same store values feed the same calculation |
| Security & Build | 8/10 | CSS only; no call, write or dependency |
| Design & Steward | 8/10 | the row now follows the reading order, the label has room, and the previous arrangement stays documented in 2.17.3 |

**Mean 8.0/10** — at the bar.

## Known limits

- The narrow row is three lines tall (description, weight, quantity and length);
  that is the space the readable units need at 720px.
- 2.17.3 remains published with the earlier arrangement; this release corrects
  it rather than replacing the tag.
- No automated visual test covers the grid; the states were checked by hand.
