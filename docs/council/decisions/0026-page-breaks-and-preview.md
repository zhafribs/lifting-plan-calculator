# 0026 — Page breaks: what was measured, and what was chosen

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.7
**Change:** `src-tauri/src/report.rs`, `src-tauri/src/report_style.rs`,
`ui/js/screens/report.js`, `ui/css/app.css`
**Seats:** Build & Platform Engineer, QA & Test Engineer, Product Manager. The
Release Manager chaired; the implementer did not score.

## Proposal

The operator reported a section title landing at the foot of one page with its
content on the next, and asked which methods exist. The candidates were built
and measured with the real stylesheet in headless Chromium, on a six-section
document of half-page sections:

| Method | Pages | Orphan titles |
| --- | --- | --- |
| current (heading `break-after: avoid`) | 4 | 0 in the synthetic document |
| `break-inside: avoid` on the whole section card | 6 | 0 |
| each section starts on a new page | 9 | 0 |
| heading wrapped with its first block | equals card-avoid when the first block is the table (13 → 14) | 0 |
| **keep-with-next stated on both sides** | **4** | **0** |

Choices:

- Keep the compact version: the heading may not end a page *and* the block after
  it may not start one. Both declarations are cheap and pair up, so a renderer
  that drops one still has the other.
- The three alternatives that guarantee the result by moving whole sections cost
  50 % to 125 % more pages for a six-section plan. For a document that is handed
  to a crane operator, blank paper is not a neutral cost.
- Measured pagination (render, measure section offsets, then insert explicit
  breaks) is the escalation if a real plan still shows the fault. It is the only
  method that is both exact and paper-efficient, but it needs a measuring pass
  and a reprint after every content change.

The same release fixes the preview: both modes now share one frame, and the live
view is laid out at the print width and zoomed to the pane, so it reads at the
same scale as the page view instead of at 7 pt in a column.

## What the council saw

- The page counts above come from `chromium --print-to-pdf` with
  `report_style.rs` verbatim; orphan detection reads each page's last text line.
- The synthetic corpus could not reproduce an orphan with the current CSS, so
  the reported case is likely a specific section shape. The both-sides rule is
  the cheap hardening; the exact fix is the measured pass, kept in reserve.
- Both preview modes were photographed in the running app, before and after.
- 261 Rust tests and 27 frontend tests pass; `clippy -D warnings` is clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | the chosen rule is CSS only, no extra render pass and no page cost |
| QA & Test Engineer | 8/10 | every candidate was measured on the same corpus, with page counts and orphan counts, not asserted |
| Product Manager | 8/10 | the plan keeps its page count, and the preview finally reads at print size |

**Mean 8.0/10** — at the bar.

## Known limits

- The reported fault was not reproduced exactly; the hardening covers the
  heading case but a different shape (for example a lead-in paragraph above a
  table) may still split.
- Measured pagination is designed but not built. It is the next step if the
  fault survives with real data.
- The live view is a continuous flow, not paginated; that is what "live" means
  here, and the page view remains the place to check breaks.
