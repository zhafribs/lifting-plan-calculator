# Vector diagram output

**Date:** 2026-11-26

## Problem

The printed working-range diagram is a 2700×2100 PNG embedded as a data URL.
Every report export carries a raster image, re-decoded by the print pipeline.
The line work and labels are sharp enough at 3×, but the PDF is larger than it
needs to be and the print path pays a raster decode.

## Goal

Emit the diagram as SVG for the report, so the PDF carries vectors:

- same layout and label placement as the current print capture;
- no raster image for the diagram in the exported PDF;
- sharper at any zoom; PDF size is measured, not assumed;
- PNG remains the fallback when SVG generation fails;
- no change to the screen canvas, report layout, maths or page.

## Non-goals

- Replacing the interactive Graph canvas.
- Changing the report's diagram size or scale control.
- New dependencies or a bundler.
- Reworking the report's HTML/CSS.

## Constraints

- No new dependency; the SVG surface is authored in `ui/js/graph/`.
- The diagram image stays a `data:` URL; CSP already allows `img-src data:`.
- The printed colours must match `printify`'s filter chain, not the dark screen
  palette.
- Text must remain real text in the SVG; no glyph outlines unless a font is
  unavailable.
- The capture cache key includes the output format.
- A failed SVG build falls back to the current PNG without failing the report.

## Acceptance

- Export a plan and run `pdfimages -list`: the diagram contributes no image (or
  only a tiny fallback), while `pdffonts`/text extraction still sees the labels.
- The diagram is visually identical to the 2.17.18 export at 100% and when
  zoomed in the PDF.
- The same plan exported with the SVG diagram is 689,927 B against 895,745 B
  for the PNG diagram, 205,818 B (23.0%) smaller.
- The live frame and Page view both show the SVG diagram.
- A forced SVG failure still produces a report with the PNG diagram.
- Capture cache hits by crane state + format; a report text edit does not
  rebuild either format.
- Node tests cover the SVG surface and colour mapping; cargo tests unchanged.

## Risks

- WebKitGTK/WebView2 may rasterise an SVG `<img>` during print, or substitute
  fonts; the spike must prove vector output before the full surface is written.
- `measureText` is needed for pill widths; the surface must use a real 2D
  measure context, not an approximation, or labels shift.
- SVG text depends on the report's fonts; a missing family must fall back
  predictably. A standalone SVG `<img>` cannot use the app's document webfont,
  so a small Inter subset is embedded and measured with the document's metrics.
- High-alpha colours are flattened over the diagram background before the
  filter; low-alpha fills stay translucent so the grid remains visible.
