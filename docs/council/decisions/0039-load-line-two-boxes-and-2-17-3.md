# 0039 — The load line's two boxes, and 2.17.3

**Status:** accepted
**Date:** 2026-11-20
**Release:** v2.17.3
**Change:** `ui/js/screens/overall.js`, `ui/css/app.css`, the rail toggle from
0038, `CHANGELOG.md`, `docs/releases/v2.17.3.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

At a narrow window, a load line measured by the metre or the volume put two
value boxes, the "Measured by" dropdown and the quantity on one 122px line. The
unit took the width the number needed, and "Weight by Volume" had almost no room
left. Stack the two value boxes at every width, and in a narrow card give the
dropdown a line of its own with the quantity beneath it.

## What changed

- A load line whose measurement reads two values (per metre: `kg/m` and `m`;
  per volume: `m3` and `kg/m3`) carries a `two-measure` marker.
- Its two value boxes stack at every window size, so each keeps room for its
  unit instead of splitting 122px in half.
- Below a 620px card, the measured-by dropdown spans the card's width and the
  quantity drops to the line below. A fixed-weight line keeps its existing
  two-line layout.
- 2.17.3 also carries the Linux rail toggle from 0038, which was accepted
  separately.

## Evidence

Checked on the built binary at 720×540 and 1440×950:

- per metre: `kg/m` and `m` boxes each show their unit and a value;
- per volume: `m3` and `kg/m3`, with "Weight by Volume" fully readable;
- fixed: unchanged two-line layout at 720 and one-line at 1440;
- switching between the three types repaints the row and adds or removes the
  extra line.

Automated: `node --check` on every frontend module, `node --test` 38 passed,
`cargo test --all-targets` 268 passed, `cargo clippy --all-targets -- -D
warnings` clean (no Rust changed).

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the field order and units are unchanged; the same values feed the same calculation, and the total still reads from the same store |
| Security & Build | 8/10 | no new input, call or dependency; one class toggled from the existing type |
| Design & Steward | 8/10 | the narrow and wide cases both have room for their units, fixed lines are untouched, and the rule is one class instead of a second layout |

**Mean 8.0/10** — at the bar.

## Known limits

- The narrow two-measure row is three lines tall; that is the cost of readable
  units at 720px.
- No automated visual test covers the grid; the states were checked by hand at
  the two widths.
- The `two-measure` marker is a class on the row, so a future field type that
  reads two values must set it in `measurementFields`.
