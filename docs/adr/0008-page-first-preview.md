# ADR-0008 — The Pages view shows the report's first page while the PDF prints

- **Status:** accepted, 2026-11-26
- **Owners:** Maintainer, UI/UX
- **Applies to:** `ui/js/screens/report.js`, `ui/js/pdf-math.js`, the report's Pages view

## Context

The Pages view is the printed PDF; it is the only view that shows what *Save as
PDF* writes. A print takes about 380 ms for a one-page plan and about 1 s for a
nine-page plan. Until it returns there is no document, so the pane showed a page shape
reading "Preparing page 1…" with nothing of the report in it.

The print cannot be streamed: WebKitGTK's print operation paginates the whole
document before it emits a page, and the GTK print backend writes the file at
the end. Printing page 1 first would pay the layout cost twice.

The live view already renders the export HTML in an iframe and typesets it, so
the exact document that will be printed is on screen in the Live tab before the
print starts.

## Decision

- While the first PDF prints, the live frame is shown through a page-shaped
  window positioned exactly where the PDF's first page will appear.
- The window is an A4 box at the viewer's fit width, with the chosen print
  margins as padding; the report body is laid out in CSS pixels and scaled by
  the 72/96 ratio the page points need (`pageFirstBox` in `pdf-math.js`, which
  has a unit test).
- The PDF replaces it as soon as it arrives, and remains the authoritative page.
  The preview is a window onto the report, not a PDF.
- A re-render with pages already on screen keeps those pages; the preview is
  only for the first render.
- The bar says "Page 1" while the preview is shown instead of "No pages".

## Consequences

- The wait has the real report in it, at the printed page's size, with no second
  typeset and no second print.
- The preview has no stamped footer (the export writes it onto the PDF) and no
  page count.
- Only the visible part of page 1 is shown: the page is clipped at the same edge
  as the PDF pane, so the bottom margin does not appear while the preview
  stands in.
- A block that the print pushes whole onto page 2 can appear at the bottom of
  page 1 in the preview; the printed page is the one that counts.
- The preview's geometry is tied to the viewer's fit formula; both now come from
  `pdf-math.js`.

## Alternatives rejected

- **Print page 1 alone, then the rest.** GTK paginates the whole document for
  either range, so this doubles the work and pushes the full preview past a
  second without producing page 1 much sooner.
- **A second iframe with the export HTML.** It works but pays a second parse and
  KaTeX pass while the print is already competing for the same thread.
- **Incremental PDF output.** WebKitGTK does not expose it; the file is written
  at the end.
