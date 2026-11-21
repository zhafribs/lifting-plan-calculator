# 0049 — The choke geometry in two columns

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `src-tauri/src/report_uniform.rs`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Section 4 printed one label and one display equation per line, using the full
page width for formulas that are mostly short. Lay the round-choke workings out
two per row, in the same grid the rest of the report already uses.

## What changed

- The round-choke geometry builds label/formula blocks and prints them with
  `working_grid`: two per row.
- The choke-to-load height keeps its `Y` and `Height = Y − Radius` equations in
  one block, so the derivation is not separated across columns.
- The prose "Operational Rule Declared" is a block of its own rather than a
  full-width paragraph between formulas.

## Evidence

- `cargo test --all-targets` 271 passed, including
  `the_round_choke_height_states_its_derivation`, which asserts the three
  equations unchanged.
- `cargo clippy --all-targets -- -D warnings` clean.
- The formulas are unchanged; only their arrangement moved.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | every equation string is the same; no figure or limit touched |
| Security & Build | 8/10 | presentation only |
| Design & Steward | 8/10 | the section uses the report's existing two-column working grid |

**Mean 8.0/10** — at the bar.

## Known limits

- Only the round-choke branch is compacted. The rectangular choke and the
  direct/nested branches keep their one-column flow; they have their own long
  formulas and should be converted with the same care when needed.
