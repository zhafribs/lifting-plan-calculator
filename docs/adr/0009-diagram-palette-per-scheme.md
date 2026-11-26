# ADR-0009 — The working-range diagram takes the light scheme's colours

- **Status:** accepted, 2026-11-26
- **Owners:** Maintainer, UI/UX
- **Applies to:** `ui/js/graph/diagram-palette.js`, the Graph tab's canvas

## Context

The chart canvas had one palette: a dark field (`#141922`) with blue, amber and
pale grey. That matched the original app, where the whole window was dark. In a
light scheme the app's surfaces are white and pale grey, so the chart stood out
as a dark rectangle; the report, meanwhile, prints the same chart light, because
the print path filters the dark pixels.

The screen and the print therefore already disagreed. The chart needed a light
rendition, and there were two ways to get one: design a second palette, or use
the one the report already prints.

## Decision

- `DIAGRAM_DARK` keeps the original palette, value for value.
- `DIAGRAM_LIGHT` is every dark value through the existing print filter
  (`printColour`), so a light window shows the chart the page shows.
- The screen picks a palette from `document.documentElement.dataset.theme`; a
  `lpc:theme` event repaints an open chart when the scheme changes.
- The print capture always starts from `PRINT_PALETTE` (the dark set): the filter
  runs there, and filtering the filtered colours would wash the page out.
- One value leaves the print palette: `danger`, the below-ground warning red.
  The printed red is 3:1 on the light page; the screen uses the app's light
  warning red (`#a33333`) at about 6.8:1.

## Consequences

- What the screen shows in a light scheme is what the page prints, so a reader
  can trust that the chart they arranged is the one they get.
- Some greys are faint by design (grid 1.13:1, axis ticks 3.77:1 against the
  chart field). They are the page's own values and are kept for that parity; the
  tick grey sits inside the app's own "faint" tier.
- A future palette change moves both the screen and the page; there is no second
  set to keep in step.
- `paint()` sets a module-level active palette. Painting is synchronous, so the
  screen's choice cannot leak into the print canvas.

## Alternatives rejected

- **A separately designed light palette.** It would look better than the print
  chart, but the screen would stop predicting the page, and two palettes would
  need maintaining.
- **A CSS filter on the canvas.** It rasterises the filtered content on print and
  softens the diagram, which the vector work removed.
- **Keeping the dark chart in both schemes.** It is what prompted this change.
