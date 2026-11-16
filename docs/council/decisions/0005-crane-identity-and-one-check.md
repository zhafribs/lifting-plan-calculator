# 0005 — The workbook names the crane, and the chart is the only check

**Status:** accepted
**Date:** 2026-11-16
**Release:** v2.9.0
**Change:** `src-tauri/src/excel.rs`, `src-tauri/src/crane.rs`,
`src-tauri/src/report.rs`, `ui/js/screens/crane.js`, `README.md`,
`docs/adr/0005-chart-is-the-only-check.md`
**Seats:** Domain Safety Engineer, QA & Test Engineer, Technical Writer,
Product Manager. The Release Manager chaired; the implementer did not score.

## Proposal

Two changes to the Crane tab, both from the operator's seat:

1. **The workbook names the machine.** The example charts now open with a title
   line (`Tadano TL200M (20 ton)`). Importing a chart fills **Crane name** and
   **Crane rated capacity** from it, so the operator types neither and cannot
   mistype a name that then prints on the plan.
2. **One check.** The booked-rating check added in ADR-0002 is withdrawn: the
   load chart is the rating that applies at the boom and radius in use, and a
   second test against the plate rating answered a different question. The tab's
   "Booked rating check" row and the report's booked note go with it.

## What the council saw

- `cargo test --all-targets` → **202 passed**; the three booked-check tests are
  replaced by a guard that the card no longer claims a check.
- `cargo clippy --all-targets -- -D warnings` clean.
- The title-line reader accepts `(20 ton)`, `(30 TON)`, `(20000 kg)` and
  refuses headings that carry no rating — `TOTAL RATED LOADS … (360°), Unit:
  ton` and `Crane location on axis (X, Y)` are both tested as non-matches.
- A workbook without a title line leaves what the operator typed alone (test).
- Both bundled workbooks are asserted: `Tadano TL200M` / 20 t and
  `Tadano TR350M-1` / 35 t.
- The locally built 2.9.0 AppImage shows the "Crane name" field and no booked
  row; the workbook picker could not be driven headlessly (the XDG portal has
  no backend under Xvfb), so the import is proven by tests, not by a click.
- The page-margin control was driven in the running app under Xvfb: the popover
  opens from **Margins**, the 20 mm preset fills the four fields and reports
  "Text area: 170 × 257 mm on A4", and Apply re-renders the page view at the new
  margin. Values outside 5–50 mm or leaving under 60 mm of text block Apply with
  a spoken error.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Domain Safety Engineer | 8/10 | the plan now states one capacity verdict with one meaning; the chart range refusal from ADR-0001 still guards the radius |
| QA & Test Engineer | 8/10 | parsing, refusal of look-alike headings, absence-of-line fallback and both bundled workbooks are covered; the removed check's tests were replaced, not deleted |
| Technical Writer | 8/10 | the tab, the report and the README say the same thing: the chart is the check, the rating is on the plan for the record |
| Product Manager | 8/10 | fewer fields to type, and no second verdict to explain on site |

**Mean 8.0/10** — at the bar.

## Required changes before merge

1. **SAFE-1 — withdraw, don't erase.** ADR-0002 is marked withdrawn and links to
   ADR-0005, so the earlier reasoning stays auditable.
2. **TW-1 — say what is not checked.** The README states plainly that the rated
   capacity is printed for the record and not tested.
3. **QA-1 — keep the guard.** A test asserts the crane-rating card makes no
   claim ("checked against 75%" / "below the 75% rule" must not appear).

## Known limits

- A crane too small for the lift is caught by the chart (or by the radius being
  off the chart), not by a plate-rating comparison. That is the accepted
  trade-off of ADR-0005.
- The title-line format is the workbook's convention: a name, then a bracketed
  rating. A workbook that names the machine differently simply leaves the
  fields for the operator to fill.
