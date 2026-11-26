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
   fell back to DejaVu Sans in the first spike. A 71 KB Basic-Latin/degree
   subset of Inter, embedded in the SVG, keeps the label face without carrying
   the full 344 KB variable font.
2. The old `printify` filtered the **composited** pixel, while mapping a
   semi-transparent colour before compositing shifted the radius line by up to
   80 blue levels.

## Decision

- `paint()` writes into a small Canvas2D-compatible SVG surface. The screen
  canvas is unchanged.
- High-alpha print colours are flattened over the diagram background
  (`#141922`) and then filtered, so the radius accent keeps its old printed hue.
  Low-alpha fills stay translucent so the grid is not erased; their hue is a few
  levels away from the old composited pixel.
- The SVG embeds the Inter subset as a data URL; labels use `InterDiagram` and
  the measure context uses the document's Inter, whose metrics match. The
  printed diagram keeps the screen face, and the PDF embeds the small subset.
- The PNG path remains the fallback and is used if the surface, the measure
  context or an unsupported transform fails.
- The capture cache keys the format, so an SVG and a PNG cannot be confused.

## Consequences

- The exported diagram is vector; labels remain text.
- The printed diagram keeps Inter; the SVG carries a 71 KB subset.
- The PDF size is measured against 2.17.18 when an export is available; no size
  claim is recorded yet.
- The SVG is larger as an HTML data URL than the PNG for simple plans, but the
  PDF does not carry the raster image.

## Alternatives rejected

- **Embed the full 344 KB Inter variable font.** It worked in the spike but is
  five times the subset; Basic Latin and the degree sign are all the diagram
  prints.
- **Use the system sans.** It removed the label face the screen and the report
  had always shown; the subset is small enough to avoid that.
- **CSS filter on the SVG.** The print pipeline rasterises filtered content,
  which is exactly what this change avoids.
- **Text-to-path conversion.** It needs a font parser and loses extractable
  labels; the spike already showed text works.
