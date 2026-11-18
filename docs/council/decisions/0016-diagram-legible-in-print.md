# 0016 — The diagram has to be readable on the plan

**Status:** accepted
**Date:** 2026-11-18
**Release:** v2.13.5
**Change:** `ui/js/screens/graph.js`, `src-tauri/src/report.rs`,
`CHANGELOG.md`
**Seats:** Technical Writer, QA & Test Engineer, Product Manager. The Release
Manager chaired; the implementer did not score.

## Proposal

The optional working-range diagram was printed at most 78 mm tall — a third of
the sheet — because a height cap kept it small. Its labels are drawn at 9–10 px
in a 900 px-wide canvas, so at that size they landed around 3 pt on paper: too
small to read, and the soft edge of a downscaled snapshot made the lines look
blurred as well.

Three changes, all in how the diagram is captured and placed:

- the report gives it the full text width (about 159 mm at one-inch margins)
  instead of a height cap;
- the offscreen capture renders at 3× (2700 px across, against the 1878 px a
  300 dpi print needs), so the lines stay sharp;
- the capture draws its labels 25 % larger, while the Graph tab keeps its
  screen sizes — one `labelScale` in the drawing code, set only for the capture.

## What the council saw

- Rendered in the running app: the page view now shows the diagram across the
  text width with the radius, height, tip and angle figures legible; the Graph
  tab is unchanged at its screen size.
- The capture size maths: 159 mm at 300 dpi is 1878 px, so a 2700 px snapshot
  has headroom rather than being stretched.
- The rest of the suite is unchanged: 246 tests, `clippy -D warnings` clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Technical Writer | 8/10 | the figure now reads at the report's own label size, which was the point of including it |
| QA & Test Engineer | 8/10 | the change is visual and was checked in the running app at page width; the drawing code keeps one type-scale switch instead of a second renderer |
| Product Manager | 8/10 | an unreadable diagram on a signed plan is worse than no diagram |

**Mean 8.0/10** — at the bar.

## Known limits

- The enlarged labels are sized for the standard crane/jib layout; a very long
  dimension string could crowd the arc on an unusual configuration. The report
  width leaves about 25 % more room than the old cap did.
- A taller diagram pushes the sections below it to the next page, which is the
  intended trade for a readable figure.
