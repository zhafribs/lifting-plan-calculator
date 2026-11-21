# 0056 — Tackle belongs to a lift, and every line control is boxed

**Status:** accepted
**Date:** 2026-11-21
**Release:** v2.17.9
**Change:** `src-tauri/src/model.rs`, `src-tauri/src/report.rs`,
`ui/js/store.js`, `ui/js/screens/overall.js`, `ui/js/screens/report.js`,
`ui/css/app.css`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Load lines already name their lift, but the tackle list did not: the rigging was
always added to whichever lift was heaviest. A 2-leg sling used only for the
concrete pour should not be carried by the steel lift. Give tackle lines the
same Lift field, and make the line's controls look editable in both cards.

## What changed

- `TackleItem` gains `lift: String` (serde default, `"Lift 1"` for a new line).
- `LiftGroup` gains `tackle: f64`. `LiftTotals::of` sums each lift's load and
  its own tackle; the governing lift is the one with the greatest **total at
  the hook**, and `load` / `tackle` / `gross` come from that lift.
- Named tackle that matches no lift, and tackle with a blank lift, is shared:
  it is counted on every lift. A typo therefore overstates rather than
  understates the rigging.
- The tackle card has a Lift field; Add tackle item copies the previous line's
  lift. The Tackle example line is `Lift 1`, matching the Load example.
- The report's lift note names the governing lift by its total.
- The description fields use the normal input border (they had a fainter one)
  and the measured-by picker is boxed like the other controls.
- A new plan starts with Plan details, Crane, Working Range Diagram, Load Items
  and Lifting Tackle, and Outrigger Pad Size ticked.

## Evidence

- Model tests `tackle_rides_with_its_own_lift` (950 + 100 tackle beats a 1,000
  load) and `shared_tackle_counts_on_the_lift_it_rides_with` (blank tackle on
  both lifts, governing total 1,100).
- In the app at 1440×950: Steel Plate 1,500 (Lift 1) and 2,000 (Lift 2) with an
  80 kg sling on Lift 1 gives Overall 2,000 and tackle 0; moving the sling to
  Lift 2 gives Overall 2,080 and "Heaviest lift: Lift 2 — 2,080.00 kg".
- 277 Rust tests, frontend tests, clippy clean.
- At 720×540 with the rail expanded both cards fit without overflow: lift,
  description, total on the first line, then the boxed measured-by, quantity and
  weight.
- The report sections card starts with all five ticked, with the two labels in
  title case; the plan preview shows the same order.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the governing lift is chosen by total at the hook, and unmatched tackle is shared rather than dropped |
| Security & Build | 8/10 | a state field and grouping; no new call, dependency or write |
| Design & Steward | 8/10 | lift names read the same in both cards, and the border makes the measured-by picker legible as a control |

**Mean 8.0/10** — at the bar.

## Known limits

- Lift names are matched exactly; `Concrete` and `concrete` are different lifts.
- Tackle shared across lifts is counted once per lift, which is correct for
  mutually exclusive lifts but overstates if two lifts happen at once.
- The printed tackle list does not prefix its rows with the lift name; only the
  load rows do.
