# 0059 — The plan block, diagram and workings share one box

**Status:** accepted
**Date:** 2026-11-22
**Release:** v2.17.12
**Change:** `src-tauri/src/report.rs`, `src-tauri/src/report_style.rs`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The plan details, the working range diagram, the two crane workings and the
line lists printed as separate cards. They are one plan header, so they belong
inside one border, with the widest content taking the room it needs.

## What changed

- The diagram, the two workings and the line lists now sit inside the Plan
  details section.
- The diagram takes the left of the row. When it leaves room on its right, the
  workings and the lists stack there; each working is at most two lines.
- When the scale leaves no room (100%, or a narrow page), they drop under the
  diagram: Required capacity and Usage side by side, the lists below.
- The lists heading uses the same bold 7 pt label as the two workings.
- The workings keep the report's normal equation size; no formula was shortened
  beyond the Usage line's two-line split at its equals sign.
- A page break ticked for the line lists still gives them their own section.

## Evidence

- 1440×950 in the Pages preview at 50%: diagram left, Required capacity, Usage
  and the lists right, no clipping.
- At 100%: the workings wrap below, side by side, and the lists follow.
- `cargo test --all-targets` 279 passed; clippy clean. The Usage reference
  string is compared with the alignment markup removed, so its literals did
  not change.
- 1920×1080 print preview checked for the refreshed screenshot.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | no figure, verdict or limit changed; the working strings keep their literals |
| Security & Build | 8/10 | report markup and CSS only; no new call, write or dependency |
| Design & Steward | 8/10 | one box for one plan, with a readable fallback when the diagram takes the page |

**Mean 8.0/10** — at the bar.

## Known limits

- The right column is tight at 50%; the line lists sit close together but fit.
- The two workings only pair side by side in the wrapped fallback, not beside
  the diagram.
