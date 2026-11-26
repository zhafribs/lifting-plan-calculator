# 0069 — The working-range diagram follows the colour scheme

**Status:** accepted
**Date:** 2026-11-26
**Release:** v2.17.22
**Change:** `ui/js/graph/diagram-palette.js`, `ui/js/screens/graph.js`,
`ui/js/app.js`, `ui/css/app.css`, test, ADR-0009
**Seats:** Safety & QA, Security & Build, Design & Steward. Bar 8/10 per seat.
The implementer did not score.

## Proposal

- Keep the chart's original palette as `DIAGRAM_DARK`.
- Derive `DIAGRAM_LIGHT` from it through the existing print filter, so a light
  window shows the chart the report prints.
- Pick the palette from the document's colour scheme and repaint on a change.
- Keep the print capture on the dark palette, where the filter is applied.

## What the council saw

- **Safety & QA (9.0):** both print paths use `PRINT_PALETTE`, `printify` runs
  once, and the 21 dark values are identical to `HEAD` — the printed page is the
  same chart. No colour literal is left in `graph.js`.
- **Security & Build (8.5):** no new dependency, network call, write, permission
  or CSP change; the palette is static data that only reaches canvas styles and
  the escaping SVG surface; the event carries a scheme string with one listener
  on a cached screen; CI copies and runs the new test with its dependency.
- **Design & Steward (9.0):** the light chart sits in the light window (chart
  field 1.06:1 against the card, grid 1.13:1 as before); text tokens clear 4.5:1
  and the warning red, the one screen-only value, is 6.65:1 on its pill; the
  theme switch repaints only on a real change and cannot leak.

## Evidence

- **Print invariant:** `graph.js:1159` (SVG) and `:1176` (PNG) pass
  `PRINT_PALETTE`; one `printify` at `:1181`; `DIAGRAM_DARK` reproduces the old
  24 references / 21 unique values exactly.
- **Light set:** every key equals `printColour(DIAGRAM_DARK[key])` except
  `danger`, which is `#a33333` (6.44:1 on the chart field, 6.65:1 on the pill).
- **Tests:** 78 frontend tests, 283 Rust tests, clippy `-D warnings`, the ES
  module syntax loop and `check-vendored.sh` pass.
- **Screenshots:** `/tmp/opencode/graph-light-final.png` and
  `graph-dark-final.png` (20:54:19/24) postdate the source (20:54:00).

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 9.0/10 | print palette pinned and unchanged; printify once; colours equal to HEAD |
| Security & Build | 8.5/10 | no new surface; static palette; bounded event; CI covers the test |
| Design & Steward | 9.0/10 | light chart fits the window; text passes; one documented screen-only red |

**Mean 8.8/10** — above the bar.

## Decision

**Accepted.** A light scheme draws the chart in the report's colours; a dark
scheme keeps the dark chart; the printed diagram is unchanged.

## Known limits

- The axis tick grey is 3.77:1, below the text floor: it is the printed page's
  own faint value, kept so the screen matches the page.
- The chart paper is the page's `rgb(244,249,255)`, so it reads a shade cooler
  than a green- or clay-tinted card.
- The print call sites cannot be unit-tested without a DOM; `PRINT_PALETTE` pins
  the alias, not the arguments.
- The screen composites filtered colours while the page filters composited
  pixels, so a translucent pill over a line can differ by a shade.
