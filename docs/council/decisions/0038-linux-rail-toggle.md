# 0038 — The rail toggle on Linux

**Status:** accepted
**Date:** 2026-11-20
**Release:** next tag
**Change:** `ui/css/app.css`, `ui/css/windows.css`, `ui/js/app.js`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The Windows build had a header button that collapses the rail to icons. Linux
kept the rail expanded above 1100px because the shell was driven by a media
query alone. Give Linux the same button without changing the Windows look or
the calculation.

## What changed

- The header (toggle + app name) is now base shell markup and styling,
  not a Windows-only block. The Windows layer still adds its Fluent metrics on
  top.
- The compact rail layout (64px strip, labels and shortcut chips hidden, name
  and version kept in the footer) is driven by `data-nav`, so a click works at
  any width. An expanded rail overrides the 1099px breakpoint, so choosing
  labels in a narrow window works.
- Until the toggle is used, the rail still follows the window: the resize
  handler re-evaluates the breakpoint, and the first click pins the choice for
  the rest of the session. Nothing is written to disk.
- Windows keeps its 1000px initial rule and its own compact metrics.

## Evidence

- Built 2.17.2 and checked at 1440×950 and 900×700 on the real compositor:
  expanded with the header, collapsed to icons, expanded again, and expanded at
  900px after the choice. Name and version stay in the compact footer; the
  contact line is on About.
- `--platform-preview=windows` at 1440×950 before and after: the Fluent header,
  rail metrics and layout are unchanged.
- `node --check` on every frontend module; `node --test` 38 passed.
  `cargo test --all-targets` 268 passed, `cargo clippy --all-targets -- -D
  warnings` clean (no Rust changed).

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | no figure, limit or verdict touched; every screen still renders at both widths |
| Security & Build | 8/10 | no new call, write or dependency; the state lives in an attribute |
| Design & Steward | 8/10 | one header shared by both platforms, the Windows layer keeps its rules, and the breakpoint still decides until the operator does |

**Mean 8.0/10** — at the bar.

## Known limits

- The choice resets on the next launch, like the state before it; nothing is
  persisted.
- The rail does not animate between widths; it snaps, which keeps the layout
  work off the main thread.
- There is no automated visual test for the two states; they were checked at
  1440×950 and 900×700.
