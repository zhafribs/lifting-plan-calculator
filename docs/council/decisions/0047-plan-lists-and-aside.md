# 0047 — The plan's line lists, and the chart note beside its label

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `src-tauri/src/report.rs`, `src-tauri/src/report_style.rs`,
`src-tauri/src/commands.rs`, `src-tauri/examples/dump_fixtures.rs`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The plan's figures said what the lift weighed but not what it was made of. List
the load and tackle lines inside section 1, after the working; if the working
range diagram is printed, put them below it. Also move the chart-capacity note
from under the figure to the label line, where it fits at the printed size.

## What changed

- `ReportInputs` carries the load and tackle lines from the Overall Weight
  state (`build_report` clones them from `AppState`).
- `plan_head` prints two compact tables after the diagram (or after the crane
  working when there is no diagram):
  - **Load items:** description, measured by, quantity, measurement (the
    per-metre/per-volume pair as `value × value`), weight.
  - **Lifting tackle:** description, quantity, weight each, subtotal.
- A line with no name is an unused blank row and is left out. If both lists are
  empty, nothing is printed.
- `Chart capacity` uses a new `metric_cell_aside`: the note sits on the label
  line and wraps only if the cell is too narrow.

## Evidence

- The Summary preview at 1440×950 with a 5,000 kg "Steel plate" line and the
  crane example shows the table under the working pair, with the header row and
  the figure right-aligned.
- New tests: `the_plan_lists_the_load_and_tackle_lines`,
  `the_line_lists_follow_the_working_diagram_when_one_is_printed`,
  `the_chart_capacity_note_sits_beside_the_label`.
- `cargo test --all-targets` 271 passed; clippy clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the figures come from the same store values the tabs use; no new arithmetic |
| Security & Build | 8/10 | the lists are read-only report content; names are escaped |
| Design & Steward | 8/10 | the lists sit with the plan details, or under the diagram, and the note is on the label line |

**Mean 8.0/10** — at the bar.

## Known limits

- Long lists flow with the section; the operator still decides page breaks.
- A line with no name is omitted from the plan, even if it has a weight.
- The aside wraps under the label on a very narrow printed cell rather than
  shrinking.
