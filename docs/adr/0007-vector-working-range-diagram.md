# ADR-0007 — The working-range diagram prints as SVG with a PNG fallback

- **Status:** accepted, 2026-11-26
- **Owners:** Maintainer, UI/UX, QA
- **Applies to:** `ui/js/graph/svg-surface.js`, `ui/js/screens/graph.js`, the report's diagram image

## Context

The report embedded the diagram as a 2700×2100 PNG. The lines and labels were
sharp at 3×, but every export carried a raster image and the print path decoded
and re-encoded it. A spike proved the app's WebKit print keeps an SVG `<img>` as
vectors: `pdfimages -list` reported no images and the labels extracted as text.

Two constraints surfaced in the spike:

1. A standalone SVG image cannot use the app's document `@font-face`; the labels
   fell back to DejaVu Sans. Embedding the 344 KB Inter face was possible but
   would put the whole font in every capture.
2. The old `printify` filtered the **composited** pixel, while mapping a
   semi-transparent colour before compositing shifted the radius line by up to
   80 blue levels.

## Decision

- `paint()` writes into a small Canvas2D-compatible SVG surface. The screen
  canvas is unchanged.
- Print colours are flattened over the diagram background (`#141922`) and then
  filtered, so semi-transparent accents keep their old printed hue. Overlapping
  translucent shapes are an approximation.
- Diagram labels use `sans-serif` both in the SVG and in the measure context, so
  pill widths match what prints. The screen and the rest of the app keep Inter;
  the printed diagram face can differ from the previous PNG.
- The PNG path remains the fallback and is used if the surface, the measure
  context or an unsupported transform fails.
- The capture cache keys the format, so an SVG and a PNG cannot be confused.

## Consequences

- The exported diagram is vector; labels remain text.
- The printed diagram's typeface is the system sans, not the app's Inter.
- The PDF size is measured against 2.17.18 when an export is available; no size
  claim is recorded yet.
- The SVG is larger as an HTML data URL than the PNG for simple plans, but the
  PDF does not carry the raster image.

## Alternatives rejected

- **Embed the full Inter woff2 in every SVG.** It worked in the spike and
  produced a small PDF, but adds 344 KB to every capture and the data URL; the
  asynchronous font load would complicate a synchronous capture.
- **CSS filter on the SVG.** The print pipeline rasterises filtered content,
  which is exactly what this change avoids.
- **Text-to-path conversion.** It needs a font parser and loses extractable
  labels; the spike already showed text works.
