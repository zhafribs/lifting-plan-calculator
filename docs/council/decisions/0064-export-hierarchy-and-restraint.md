# 0064 — Report export hierarchy and surface restraint

**Status:** accepted
**Date:** 2026-11-26
**Release:** v2.17.17
**Change:** `ui/js/screens/report.js`, `ui/css/app.css`, `ui/css/windows.css`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (QA & Test
Engineer, Technical Writer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The Summary Export card had two equally prominent buttons and four equal-width
actions. Give it one hierarchy, remove decoration that did not serve it, and fix
the copy:

- Save as PDF primary, Print secondary, Save as HTML / Open in browser tertiary.
- Open in browser writes the current report first; it reuses the saved file only
  when the bytes match, so an edit cannot reopen an older plan.
- Delete the false "Use in report" note; describe the margins as the values in
  the preview bar; cover both print paths.
- Remove the active-nav and card-head gradients and the dead base nav rule; use
  a 1px readable status border on toasts.

## What the council saw

- **Safety & QA (9.0):** every export path calls `ensureHtml("export")`; PDF
  keeps the stamped A4 path; Open now reuses only identical HTML; no report
  content, footer, margins or arithmetic changed. The toast status borders clear
  3:1 in all four palettes after the text-token fix.
- **Security & Build (9.5):** CSS and one screen's class/order only; no
  dependency, network, CSP, capability, path or file-write change. The Windows
  toast override is load-bearing at its specificity and documented.
- **Design & Steward (8.5):** three real levels tied to product truth; the
  gradients were decoration; the copy is true and shorter; Inter stays because
  the app's fixed value boxes depend on its metrics.
- **Release Manager:** notes name 2.17.17 and this record; the report is
  unchanged.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 9.0/10 | current export on every path; readable status; no report change |
| Security & Build | 9.5/10 | no new surface; specificity fix documented |
| Design & Steward | 8.5/10 | clear hierarchy; decoration removed; honest copy |

**Mean 9.0/10** — at the bar.

## Decision

**Accepted.** The export card has one primary action, the shell is quieter, and
Open cannot serve a stale plan.

## Consequences

- Save as PDF is the obvious output; Print and HTML/browser actions are still
  present at lower emphasis.
- An edit followed by Open in browser can ask for a filename.
- The report and its PDF layout are unchanged.

## Known limits

- PDF and HTML still share the same file icon.
- Toasts still have no `role`/`aria-live`; pre-existing, unchanged by this
  record.
- `InterVariable-Italic.woff2` ships without a matching `@font-face`; upkeep
  only.
- Inter stays for the fixed value boxes; the rationale lives in the CSS comment
  and this record, not a DESIGN.md.
