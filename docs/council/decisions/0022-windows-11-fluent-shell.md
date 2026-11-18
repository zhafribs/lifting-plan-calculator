# 0022 — Windows 11 Fluent shell

**Status:** accepted
**Date:** 2026-11-18
**Release:** v2.15.0
**Change:** `ui/css/windows.css`, `ui/js/windows-theme.js`, `ui/js/app.js`,
`ui/js/dom.js`, `src-tauri/src/theme_windows.rs`, `src-tauri/src/commands.rs`,
`src-tauri/tauri.windows.conf.json`, tests and docs
**Seats:** Release Manager, Product Manager, UI/UX Designer, Windows Platform
Engineer, QA & Test Engineer, Accessibility Reviewer, Technical Writer,
Maintainer / Steward. The Release Manager chaired; the implementer did not
score.

## Proposal

Give the Windows build a Windows 11 Fluent shell without touching the lift
mathematics, the printed report or the PDF:

- a native Windows 11 title bar over Mica, with a solid Fluent fallback where
  Mica is unavailable;
- a Fluent NavigationView rail that starts expanded at 240 px and folds to a
  56 px icon strip from a hamburger in its header;
- the Windows system light/dark setting and the system accent colour;
- Fluent cards, controls, status badges, list rows and focus rings;
- Linux and macOS keep the existing theme, and the Windows layout can be
  previewed on Linux with `--platform-preview=windows`.

## What the council saw

- **Product Manager:** the change is confined to the shell. The calculation
  engine, chart, report, PDF and page-margin control are untouched, and the
  scope-of-use text still applies. A minor version is right: new appearance,
  same figures.
- **UI/UX Designer:** the light and dark token sets follow the approved design,
  the rail is 240/56 px with a 3 px accent bar and a subtle selection fill, and
  the two widths were inspected at 1440×950 and 720×540. The compact rail keeps
  its tooltips and shortcuts, and the expanded rail keeps labels even below
  1100 px, where the base stylesheet used to collapse it.
- **Windows Platform Engineer:** the accent comes from
  `DwmGetColorizationColor` (`#rrggbb`), with `#0067c0` and `#60cdff` fallbacks;
  `tauri.windows.conf.json` sets `transparent: true`, `windowEffects: mica` and
  keeps native decorations. Windows 10 therefore gets the same layout on solid
  surfaces. The preview flag changes the CSS marker only; the updater still uses
  the real platform.
- **QA & Test Engineer:** `cargo test` → **246 passed**; `node --test
  ui/js/windows-theme.test.mjs` → **5 passed**; `cargo clippy --all-targets -- -D
  warnings` is clean; `git diff --check` is clean. The base Linux layout was
  inspected without the preview flag and is unchanged. Windows preview screens
  were inspected in light, dark, expanded and compact.
  **Pre-publication checklist:** the physical Windows run (Mica on 11, solid
  fallback on 10, 125 %/150 % scaling, live accent change, high contrast, and a
  report/PDF comparison) is still to be signed off before the tag is pushed.
- **Accessibility Reviewer:** the toggle has `aria-expanded` and a changing
  `aria-label`, the active tab carries `aria-current="page"`, focus is a 2 px
  accent ring with a 2 px offset, compact icons keep their names through the
  existing `title` attributes, and `forced-colors` and reduced-motion rules are
  present.
- **Technical Writer:** the README and tutorial explain the Windows 11 shell,
  the Windows 10 fallback and the unchanged report; the release notes carry the
  same points and the Linux preview caveat.
- **Maintainer / Steward:** no Fluent runtime, npm build step, new dependency or
  CSP change. The only native addition is one DWM call behind the existing
  `windows` crate, and the new frontend test runs in CI.
- **Release Manager:** the notes, version and this record name v2.15.0. The tag
  waits on the Windows checklist above; a failure there reopens the change.

## Decision

**Accepted.** The Windows build ships the Fluent shell, Mica on Windows 11 and a
solid Fluent fallback on Windows 10, following the system theme and accent,
while the report and the calculations stay as they are.

## Consequences

- Windows users see a different shell; Linux and macOS users see no visual
  change.
- Windows 10 needs no Mica support, because the token layer stands on its own.
- The report and PDF output remain the contract they were before this release.
- `--platform-preview=windows` is a QA hook, not a shipped Windows behaviour.
