# 0018 — The diagram keeps its old size, and the print palette is baked in

**Status:** accepted
**Date:** 2026-11-18
**Release:** v2.14.0
**Change:** `ui/js/screens/graph.js`, `ui/js/graph/print-palette.js`,
`src-tauri/src/report.rs`, `.github/workflows/ci.yml`
**Seats:** Build & Platform Engineer, QA & Test Engineer, Product Manager. The
Release Manager chaired; the implementer did not score.

## Proposal

Decision 0016 gave the diagram the full text width because, at the old 78 mm
height cap, its labels were unreadable. On paper the result was a picture that
took most of the page and still looked soft, so 0016's sizing is withdrawn:

- the report gives the diagram back its `max-height: 78mm` (about 100 mm across
  for the 900 × 700 capture), and
- the blur is fixed at its cause rather than by enlarging the picture.

The cause was the report's `filter: invert(1) hue-rotate(180deg)
brightness(1.08)`. A filtered image is not embedded in the PDF as captured;
Chromium rasterises the filtered layer again at 300 dpi, which is what produced
the soft edges. Chromium will embed the untouched bitmap at its own resolution —
2700 px across 100 mm, about 684 ppi — so the capture now paints its colours
already turned over, using the filter chain's own matrix, and the image element
carries no filter.

The 3× capture and the 25 % larger labels stay: they are what makes the smaller
diagram readable.

## What the council saw

- `printify` was checked against Chromium's rendering of the same filter chain
  on the palette's colours and the primaries: every channel is within 2 of the
  filter's output, so the printed colours do not change.
- A capture-sized canvas put through the new path, printed with the same
  headless Chromium the export uses: the PDF embeds a 2700 × 2100 image at
  684 ppi. The same page with the old CSS filter embedded 2011 × 1573 at
  300 ppi, and before 0016 the capture itself was 382 px at 90 ppi.
- WebKitGTK 4.1 has no `CanvasRenderingContext2D.filter`, so the transform is a
  byte loop over the pixels, not a second canvas; it exists as its own module
  with a `node --test` case, which the CI test job now runs.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | the transform is one pure function with no platform dependency; WebKitGTK's missing canvas filter is worked around, not worked against |
| QA & Test Engineer | 8/10 | the colour maths is asserted against Chromium, and the print resolution was measured in a real exported PDF |
| Product Manager | 8/10 | the plan keeps its page for the calculation, and the diagram is sharper than it was at full width |

**Mean 8.0/10** — at the bar.

## Known limits

- The blur was reproduced and measured through Chromium's print path, which is
  what Linux uses. Windows prints through headless Edge first (also Chromium);
  its own WebView2 fallback was not re-measured.
- The diagram's labels are back to about 4 pt on paper. Sharp at 684 ppi, but a
  reader who wants them bigger still has the 0016 trade: full width, less room
  for the calculations.
