# 0071 — The plan is saved as an explicit, versioned file

**Status:** accepted
**Date:** 2026-11-28
**Release:** v2.17.24
**Change:** `src-tauri/src/plan.rs`, `src-tauri/src/commands.rs`,
`src-tauri/src/lib.rs`, `src-tauri/src/outrigger.rs`, `ui/js/graph/snapshot.js`,
`ui/js/screens/graph.js`, `ui/js/screens/report.js`, `ui/js/screens/crane.js`,
`ui/js/screens/overall.js`, `ui/js/components.js`, `ui/js/api.js`, tests,
ADR-0010, spec `2026-11-28-plan-file.md`
**Seats:** Safety & QA, Security & Build, Design & Steward. Bar 8/10 per seat.
The implementer did not score.

## Proposal

- Explicit **Save plan…** and **Open plan…** on the Summary, to a path the
  operator chooses. No auto-save, no recent-files list.
- One JSON file, `format: lifting-plan-calculator/plan`, `version: 1`, carrying
  the wire `AppState` and the report settings, including the Graph tab's drawing
  state and the issue date. Results are recomputed on open, never stored.
- Rust parses and validates the untrusted file: 4 MiB cap, format/version,
  required top-level keys, every string and number, catalog keys, unique ids,
  sections/page breaks, margins, scale, date, and the diagram state.
- A refusal returns a message and changes nothing; the save path runs the same
  validation, so the app cannot write a file it would refuse to open.
- The privacy contract changes in one sentence: no plan is stored between
  launches unless the operator saves one. The appearance setting and `ui.log`
  are named as the other writes; the update check only reads.

## What the council saw

- **Safety & QA (8.0):** the printed figures and verdicts are untouched; a saved
  plan carries inputs, not results; the diagram's sweep labels reproduce from
  the saved drawing state; the jib offset uses its control's `-30..60` range;
  the size cap is enforced on write. It caught three real defects over the
  rounds: a page break the UI could leave on a hidden section, a diagram that
  redrew defaults, and a workbook re-sync that discarded the saved drawing.
- **Security & Build (9.0):** the file is untrusted and fully validated; nothing
  is read or written outside the dialog's choice; the sign-off/footer strings
  stay escaped; the staged write cannot truncate an existing plan; no new
  dependency, network call, permission or CSP change; the 4 MiB write cap cannot
  be bypassed. Its notes led to a `lpc-part-` staging prefix so the sweep cannot
  delete a user's similar-named file.
- **Design & Steward (8.5):** the buttons and copy are clear; the issued date is
  explained and kept with the plan; Open-replaces and the workbook-table limit
  are stated where they bite; the privacy claim is scoped to the plan and the
  appearance file is named. It caught the broadened "nothing is stored" claim,
  which was false because the theme is kept.

## Evidence

- **Format and validation:** 23 unit tests — round trip, every refusal (format,
  version, missing keys, negative weight, absurd capacity, duplicate ids, unknown
  catalog keys/sections, page break on a disabled section, margins, scale, date,
  strings, oversize, malformed JSON), the diagram state, the outrigger flags, and
  the write-side size cap with four 50 000-element arrays.
- **Diagram:** a runtime probe applied a plan with a 10°–60° sweep and captured
  the report SVG: labels 10° and 60°, no 80°. The saved drawing survives the
  workbook sync by construction (writer settles the link, applies the plan, marks
  a deviation; the next sync sees no change).
- **Frontend:** 93 tests, including `snapshot.js`'s four for the diagram state
  and its pending path; the ES module syntax loop is clean.
- **Rust:** 305 tests at the final gate, clippy `-D warnings`, `check-vendored.sh`
  and the audit jobs green.
- **Plan size:** a plan with a full workbook chart stays far below the 4 MiB cap;
  the write-side cap test uses the largest arrays the schema allows.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8.0/10 | figures untouched; diagram and page-break states reproducible; size cap on write |
| Security & Build | 9.0/10 | untrusted file contained; staged write; no new surface |
| Design & Steward | 8.5/10 | copy, date, Open consequences and the privacy claim corrected |

**Mean 8.5/10** — above the bar.

## Decision

**Accepted.** A plan is saved and reopened explicitly, with its inputs, report
settings and diagram state; the privacy claim is scoped to the plan and the
appearance file is named.

## Known limits

- The native save/open dialogs and the overwrite/read-only paths have no
  automated test; they rest on `rfd` and the shared staged-write code.
- The workbook writer/capture interaction is verified by reading and unit tests,
  not by a webview open-and-print run; the no-workbook case was probed.
- The workbook's full table view is not saved; the chart points, boom/radius
  lists and label are, so the check and report are complete.
- A plan keeps its issue date; reopening next month prints that date.
- Missing inner fields fall back to the wire defaults; the top-level shape is
  required.
- A saved plan is recomputed with the app that opens it; the same version prints
  the same figures.
