# 0054 — Load lines are grouped into lifts

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `src-tauri/src/model.rs`, `src-tauri/src/report.rs`,
`src-tauri/src/report_style.rs`, `ui/js/store.js`,
`ui/js/screens/overall.js`, `ui/css/app.css`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

A sheet of load lines is not one lift. Steel plate, GI, scaffold and concrete
may be lifted at different times, so summing every line overstates the crane
requirement. A concrete pour and its bucket, however, are lifted together and
must add up. Give each line a lift name; lines under one name add up, and the
plan is sized by the heaviest lift.

## What changed

- `LoadItem` gains `lift: String` (serde default; `"Lift 1"` for a new line).
- `LiftTotals::of` groups lines by lift, sums each group, and sets `load` to the
  **heaviest group**. `LiftTotals` carries `lifts: [{name, weight}]` for the
  summary. With one group — every existing state and every ported test — the
  result is the same sum as before.
- The Overall Weight row has a `Lift` field. "Add load item" copies the last
  line's lift, so a second item for the same pour is one click away.
- The summary says "Heaviest lift: <name> — <weight>" when there is more than
  one lift.
- The report prefixes each item with its lift when there are several, and adds
  "Overall weight uses <lift> (<weight>)."

## Evidence

- Model test `the_overall_weight_is_the_heaviest_lift`: Steel 800, Concrete
  1,500 + bucket 80 → load 1,580, two groups.
- Report test `the_plan_names_the_lift_that_sets_the_overall_weight`.
- 1440×950 in the app: Steel Plate (Lift 1, 1,500) plus a blank line changed to
  Lift 2 with 800 gives Total load 1,500 (not 2,300), Overall 1,580 with the
  80 kg tackle, and "Heaviest lift: Lift 1 — 1,500.00 kg".
- `cargo test --all-targets` 274 passed; clippy clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the arithmetic per lift is unchanged; the plan is sized by the heaviest lift, and the old one-group states still sum |
| Security & Build | 8/10 | a state field and grouping; no new call or dependency |
| Design & Steward | 8/10 | a lift field per line, a named governing lift in the summary, and the report says which lift the overall uses |

**Mean 8.0/10** — at the bar.

## Known limits

- The tackle list is shared: its total is added to the heaviest lift. Tackle
  that belongs to one lift only is not modelled yet.
- Lines in the same lift are grouped by exact name (case-sensitive).
- The non-uniform and tandem tabs still use their own single gross.
