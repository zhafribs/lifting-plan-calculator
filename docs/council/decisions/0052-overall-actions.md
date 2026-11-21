# 0052 — An Actions card on the Overall Weight tab

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `ui/js/screens/overall.js`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The Crane tab had Load example / Show report / Clear; the Overall tab did not,
so the first screen had no sample and no way to reset the lines.

## What changed

A fourth card in the right column, using the same three components as the Crane
tab:

- **Load example** — one load line (Steel Plate, Fixed, qty 1, 1,500 kg) and one
  tackle line (2-Leg Chain Sling, qty 1, 80 kg). Gross becomes 1,580 kg.
- **Show report** — opens the plan with the crane section selected.
- **Clear** — returns both lists to one blank line, like a fresh plan.

## Evidence

- 1440×950: the card sits under Next step; Load example fills both cards and the
  summary (1,500 + 80 = 1,580 kg, required 2,106.67 kg); Show report opens the
  plan with the load/tackle lists; Clear restores the blank rows.
- The buttons are the shared `exampleButton` / `reportButton` / `clearButton`.
- `node --check` and `node --test` 38 passed; no engine change.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | sample values are the requested ones; the totals come from the engine |
| Security & Build | 8/10 | state updates only; no new call or dependency |
| Design & Steward | 8/10 | the first tab now matches the Crane tab's actions and reset behaviour |

**Mean 8.0/10** — at the bar.

## Known limits

- The sample is not persisted; it is a session example like the Crane tab's.
