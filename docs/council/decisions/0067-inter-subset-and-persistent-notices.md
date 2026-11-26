# 0067 — Embedded Inter subset and persistent notices

**Status:** accepted
**Date:** 2026-11-26
**Release:** v2.17.20
**Change:** `ui/js/graph/diagram-font.js`, `ui/js/graph/svg-surface.js`,
`ui/js/screens/graph.js`, `ui/js/app.js`, `ui/index.html`, `ui/css/app.css`,
tests, ADR-0007
**Seats:** Safety & QA, Security & Build, Design & Steward. Bar 8/10 per seat.
The implementer did not score.

## Proposal

- Embed a Basic-Latin/degree subset of the bundled Inter in the printed SVG, so
  diagram labels keep the app face instead of falling back to the system sans.
- Keep the PNG capture as the fallback when the SVG path fails.
- Keep a failed action on screen until it is dismissed; add a close button to
  every notice and announce through live regions that exist before the message.
- Cap retained notices at four, a transient one giving way before an error.

## Evidence

- **Subset provenance.** `pyftsubset ui/fonts/InterVariable.woff2
  --unicodes="U+0020-007E,U+00B0" --flavor=woff2 --layout-features='*'`
  reproduces the embedded bytes: 72,292 B,
  `sha256 4a5713d1cbf18b7acdfbe07279a589766976e446fac1ee6faa58fac055440f9f`.
  It covers 96 code points, which is every character the diagram prints.
- **Size.** The same plan exported through the app: **689,927 B** with the SVG
  diagram against **895,745 B** with the PNG diagram 2.17.18 used — 205,818 B
  (23.0%) smaller. `pdfimages -list` lists only the PNG run's 2700×2100 image;
  `pdffonts` shows `InterVariable` and `InterVariable_700wght` in the SVG run.
- **Metrics.** Labels are measured with the document Inter; the subset's advance
  widths match it for every printed character at both weights, so pill widths
  and placement are unchanged.
- **Notices.** A runtime probe fired four errors, a success, a fifth error and a
  second success; after each step the retained list was four errors, and the
  live regions settled at `status="Another saved notice."` and
  `alert="Error five"`.
- **Gates.** 69 frontend tests, 283 Rust tests, `clippy -D warnings`, the ES
  module syntax loop and `check-vendored.sh` pass.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 9.0/10 | one announcement per notice; subset covers every glyph; pills unchanged; size pair verified |
| Security & Build | 9.0/10 | no new dependency, network, write, permission or CSP; bounded data URL; escaped font CSS |
| Design & Steward | 8.5/10 | affordance, focus handoff and copy verified; subset reproducible and metrically exact |

**Mean 8.8/10** — above the bar.

## Decision

**Accepted.** The printed diagram keeps Inter through a 71 KB subset, and a
failure stays on screen until dismissed. Colour flattening and the PNG fallback
carry over from record 0066.

## Known limits

- In the PDF text layer, a few punctuation glyphs inside diagram labels (such as
  `=`) may not extract; the printed page is unaffected.
- Low-alpha overlapping shapes remain the approximation recorded in ADR-0007.
- Two messages to the same live region in one frame collapse to the last one.
- A fifth error replaces the oldest error; the changelog and release notes say so.
- Dismiss, focus and announcement paths have no automated test; the runtime
  probe above is the evidence.
