# 0028 — Operator page breaks, and the Windows scheme switch

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.9
**Change:** `src-tauri/src/report.rs`, `src-tauri/src/report_style.rs`,
`src-tauri/src/commands.rs`, `src-tauri/src/update.rs`,
`src-tauri/src/appearance.rs`, `ui/js/screens/report.js`, `ui/js/api.js`,
`ui/css/windows.css`, `ui/css/app.css`
**Seats:** Frontend Engineer, Build & Platform Engineer, QA & Test Engineer,
Product Manager. The Release Manager chaired; the implementer did not score.

## Proposal

- **Page breaks the operator controls.** A "New page" tick on every report
  section sends that section to a fresh page. The request carries the section
  keys to `build_report`; the builder wraps the section in a `new-page` div, and
  the stylesheet makes that a page break. Nothing is ticked by default, so the
  report's own layout is unchanged for anyone who does not ask.
- **Windows appearance.** The Fluent stylesheet keyed its dark palette off
  `prefers-color-scheme`, so the Settings switch stored a preference that
  nothing applied. It now keys off `data-theme`, the same switch the rest of the
  app uses; "Match the desktop" still resolves through the OS setting.
- **Windows persistence.** The preference file was placed under `$HOME`, which a
  normal Windows session lacks, so saving returned an error and the choice did
  not survive. It now uses `%APPDATA%`.
- **Installer errors name the file.** A spawn failure tells the operator where
  the downloaded installer is, so the update can be finished by hand.

## What the council saw

- The page-break path is tested at the builder level (`section-card new-page`,
  and the wrapping div); the UI was checked in the running app, where each row
  shows the include tick and the New page tick, disabled until the section is
  included.
- The Windows stylesheet change was reviewed against the selectors it replaces;
  no media query remains in that file that could override `data-theme`.
- The `%APPDATA%` lookup is the platform's documented location for roaming
  per-user settings; `XDG_CONFIG_HOME` remains first, so Linux is unchanged.
- 262 Rust tests and 29 frontend tests pass; `clippy -D warnings` is clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 8/10 | the break is one class and one checkbox; no second layout engine |
| Build & Platform Engineer | 8/10 | the Windows path uses the platform's own data folder rather than inventing a fallback |
| QA & Test Engineer | 7/10 | the builder and the stylesheet are covered, but the Windows switch was not exercised on a Windows desktop in this session |
| Product Manager | 8/10 | the operator can lay the plan out as the site expects, and the update no longer dead-ends in silence |

**Mean 7.75/10** — at the bar.

## Known limits

- The Windows fixes were compile-checked only; the CI Windows job proves the
  build and the PDF probe, not the scheme switch or the installer hand-off.
- The reported "update from 2.15.0 fails" could not be reproduced from the
  release assets: the signed manifest verifies, and the setup and MSI are listed
  in it. The error text from that machine is needed to pin it.
- A section's break applies to its first card only; a section built from several
  cards (uniform load) continues on the following page normally.
