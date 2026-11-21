# 0055 — Plan details and the line lists are optional sections

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `src-tauri/src/report.rs`, `ui/js/screens/report.js`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Unticking every Report section still left the plan header and the load/tackle
lists on the page, with no way to remove them. They are content, so they should
be sections like the rest.

## What changed

- **Plan details** is a section above Crane. It carries the title, the plan
  meta table, the crane figures and the setup note.
- The **working range diagram** moves out of the plan header into its own block.
- **Load items and lifting tackle** is a section after the diagram and before
  the outrigger, in its own card.
- The report screen adds `no-plan` / `no-lines` to the include set when those
  boxes are unticked. Every other caller (the tabs' Show report, the tests)
  keeps the header and the lists, so their reports do not change.
- With everything unticked the plan is empty apart from its page. An empty
  body made the print backend fail with "Invalid page range", so a blank line
  is emitted when nothing else is, keeping the page.

## Evidence

- 1440×950: the Report sections card reads Plan details, Crane, Working range
  diagram, Load items and lifting tackle, Outrigger Pad Size, then the sling
  sections. Unticking all leaves a blank plan.
- 274 Rust tests pass; `node --check` and the frontend tests pass.
- The section order and the blank state were checked in the running app.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | no figure changed; the setup note is gated with the header |
| Security & Build | 8/10 | include markers only; no new call or write |
| Design & Steward | 8/10 | the leftover content becomes named, removable sections in the requested order |

**Mean 8.0/10** — at the bar.

## Known limits

- The `no-plan` / `no-lines` markers are internal to the report screen; another
  caller that wants the plan without a header would need to pass them.
- Section numbering is unchanged; the new line-list card is unnumbered, like the
  diagram.
