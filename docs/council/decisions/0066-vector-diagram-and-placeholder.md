# 0066 — Vector working-range diagram and page placeholder

**Status:** accepted
**Date:** 2026-11-26
**Release:** v2.17.19
**Change:** `ui/js/graph/svg-surface.js`, `ui/js/graph/print-palette.js`,
`ui/js/screens/graph.js`, `ui/js/screens/report.js`, `src-tauri/src/report.rs`,
tests, ADR-0007
**Seats:** Safety & QA, Security & Build, Design & Steward. Bar 8/10 per seat.
The implementer did not score.

## Proposal

- Build the printed working-range diagram as SVG from the existing `paint()`
  code, embed it as an SVG data URL, and keep the PNG capture as a fallback.
- Flatten high-alpha print colours over the diagram background before the old
  filter; keep low-alpha fills translucent so the grid survives.
- Use a measured system-sans family inside the standalone SVG.
- Show a page-shaped "Preparing page 1…" placeholder on the first Pages render.

## What the council saw

- **Safety & QA (8.5):** the placeholder appears on the first render and clears
  on every exit; the radius dimension colour is exactly `rgb(181,131,4)`; the
  PNG fallback was forced and printed a diagram; no arithmetic, verdict or
  report text changed.
- **Security & Build (8.5):** the Rust filter accepts only the two app data-URL
  prefixes, untrusted `https` and `data:text/html` are refused, the URL is
  HTML-escaped, SVG in `<img>` is script-free, and the cache is bounded.
- **Design & Steward (8.5):** the grid shows through the 10% envelope, the
  radius line matches the old pixel, the system-sans trade-off is recorded in
  ADR-0007, and the plan/spec/CHANGELOG no longer overclaim size or parity.
- **Release Manager:** notes name 2.17.19; no size claim is made.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8.5/10 | placeholder, fallback, colour and report integrity verified |
| Security & Build | 8.5/10 | exact data-URL prefixes; escaped sink; bounded cache |
| Design & Steward | 8.5/10 | grid/colour fidelity; records honest |

**Mean 8.5/10** — at the bar.

## Decision

**Accepted.** The diagram prints as vectors with a PNG fallback, the first Pages
render has a page-shaped placeholder, and the colour/typeface trade-offs are
recorded.

## Known limits

- Printed diagram labels use the system sans, not the app's Inter.
- Low-alpha overlapping shapes are a documented approximation against the old
  flattened pixel.
- PDF size versus 2.17.18 was not measured; no size claim is made.
- The forced fallback was verified by a temporary throw; it has no automated
  test.
