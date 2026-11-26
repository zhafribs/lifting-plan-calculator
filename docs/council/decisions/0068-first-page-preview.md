# 0068 — First-page preview, clean diagram grid, reachable zoomed page

**Status:** accepted
**Date:** 2026-11-26
**Release:** v2.17.21
**Change:** `ui/js/screens/report.js`, `ui/js/pdf-math.js`, `ui/js/graph/svg-surface.js`,
`ui/css/app.css`, tests, ADR-0008
**Seats:** Safety & QA, Security & Build, Design & Steward. Bar 8/10 per seat.
The implementer did not score.

## Proposal

- Show the report's first page in the Pages view while the PDF prints, then
  replace it with the printed page.
- Fix the printed diagram's grid, which joined every line to the previous one
  because `moveTo` continued the open path.
- Stop a page wider than the pane from centring itself out of reach.

## What the council saw

- **Safety & QA (8.5):** the grid path is 16 M / 16 L and the hatch 82 M / 82 L,
  so no subpath is joined; the whole-image difference against the old canvas
  capture is 2.05/255 mean with no shifted lines or fills. The preview shows the
  same export HTML the PDF is printed from, is clipped to the pane, and cannot
  outlive the print (`finally`, mode switch, error path). Report content and
  arithmetic untouched.
- **Security & Build (9.0):** no new dependency, network call, file write,
  permission or CSP change; the overlay sets only computed geometry; the
  version is in all three places and the notes name the tag. Its 8.0 blocker was
  this record's absence.
- **Design & Steward (8.5):** containment measured inside the pane at two window
  sizes and flush with the scrollport (`gap=0`); the mode guard, the resize
  re-placement and the label guard hold; the copy says the preview is not the
  PDF.

## Evidence

- **Grid:** `/tmp/opencode/diagram.svg` dumped from the app's report HTML; the
  fixed render matches the old canvas capture's grid positions within 1.4 px at
  1400 px, and only text antialiasing differs.
- **Zoom:** runtime probe with a 911 px page in a 626 px pane:
  `scrollLeft=0 frameLeft=18`; a narrower page still centres.
- **Preview box:** `pageFirstBox(626, 25.4 mm)` = zoom 0.9710, 578.0 × 817.5,
  padding 69.9; the PDF page is 578 × 818 at the same x. Body 438 × 678 in a
  438 × 678 text area. The clipped sheet ends flush with the pane (`gap=0`).
- **Gates:** 72 frontend tests, 283 Rust tests, clippy `-D warnings`, the ES
  module syntax loop, `check-vendored.sh` and `cargo audit` pass.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8.5/10 | grid fixed without moving content; preview lifecycle and clipping verified |
| Security & Build | 9.0/10 | no new surface; only computed geometry; release gates green |
| Design & Steward | 8.5/10 | containment, transitions and copy verified at two sizes |

**Mean 8.7/10** — above the bar.

## Decision

**Accepted.** The Pages view shows page 1 while the PDF prints, the printed grid
is clean, and a zoomed page scrolls to its left edge.

## Known limits

- The preview is the report, not the PDF: no stamped footer until the print
  replaces it, and a block the print pushes whole onto page 2 can appear at the
  bottom of the preview briefly.
- Only the visible part of page 1 is shown; the bottom margin appears when the
  PDF lands.
- A hung pdf.js parse would leave the preview on screen; there is no timeout.
- The Windows scrollbar gutter is reserved but unverified on a Windows host.
