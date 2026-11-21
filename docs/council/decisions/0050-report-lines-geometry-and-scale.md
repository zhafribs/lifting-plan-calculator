# 0050 — The report gains its lines, a two-column geometry, and a diagram scale

**Status:** accepted
**Date:** 2026-11-21
**Release:** v2.17.8
**Change:** `src-tauri/src/report.rs`, `src-tauri/src/report_style.rs`,
`src-tauri/src/report_outrigger.rs`, `src-tauri/src/report_uniform.rs`,
`src-tauri/src/commands.rs`, `ui/js/screens/report.js`, `ui/js/api.js`,
`ui/js/components.js`, `ui/css/app.css`, `CHANGELOG.md`,
`docs/releases/v2.17.8.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The report carried the plan's figures but not the lines they came from; its
geometry used a full page width for six short formulas; the square pad repeated
its result in a second sentence; and the working range diagram had one size.
The operator asked for the lines in section 1, a two-column geometry for every
arrangement, a single centred square rule, and a scale for the diagram.

## What changed

- **Plan lines.** `ReportInputs` carries the load and tackle items from
  `AppState`; section 1 prints two side-by-side lists of description and
  subtotal, loads left and tackle right, after the working or under the diagram.
- **Diagram scale.** `diagram_scale` (25–100%, default 100) reaches the report
  as an inline image width. The Report sections card has a 50/75/100% control
  beside the diagram tick, disabled while the tick is off.
- **Geometry.** `geometry_grid` groups the section's label/formula parts into
  the table-based working grid, two blocks per row, for every arrangement. CSS
  columns were tried first and ignored by the printed PDF, so the table grid is
  what ships.
- **Square pad.** One centred line, bold label, hairlines above and below.
- **Chart capacity note** beside its label; the usage cell drops “of chart
  capacity”.
- **Dropdown scroll.** The custom list no longer closes on its own wheel.

## Evidence

- The fixture report renders to PDF: section 4 shows two blocks per row and the
  document is five pages, down from seven. The square rule renders centred with
  its hairlines.
- `the_diagram_scale_reaches_the_image` asserts `width: 50%`;
  `the_plan_lists_the_load_and_tackle_lines` and
  `the_line_lists_follow_the_working_diagram_when_one_is_printed` cover the
  lists; the round-choke equation tests are unchanged.
- `cargo test --all-targets` 272 passed; clippy clean; `node --test` 38 passed.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | every formula and figure is unchanged; removed sentences duplicated a row above them |
| Security & Build | 8/10 | report strings and one image width; no engine path touched |
| Design & Steward | 8/10 | the report is shorter, the lines sit with their figures, and the diagram size is the operator's choice |

**Mean 8.0/10** — at the bar.

## Known limits

- The diagram scale only shrinks (25–100%); larger would overflow the printed
  width.
- Very long formulas could still exceed a half-column; none do in the fixture.
- The report layout is checked by PDF render, not by an automated visual test.
