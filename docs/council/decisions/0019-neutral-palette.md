# 0019 — A palette of our own, on every desktop

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.0
**Change:** `ui/css/app.css`, `ui/js/theme.js`, `ui/js/app.js`,
`ui/js/screens/about.js`, `ui/js/api.js`, `src-tauri/src/appearance.rs`,
`src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `ui/index.html`
**Seats:** Frontend Engineer, Application Security Engineer, QA & Test
Engineer, Product Manager. The Release Manager chaired; the implementer did not
score.

## Proposal

The Linux build had a single dark theme, and the Windows build painted a Fluent
layer over it. The operator asked for one shell of our own across every
distribution and desktop, Hyprland included, which means the look cannot hang
off one desktop's theme engine or one distro's defaults.

- Two palettes — light and dark — built from the same token names. The 1 300
  lines of screen CSS are untouched; only the token block has a light variant,
  so a rule never needs a per-theme copy.
- The scheme follows the desktop's `prefers-color-scheme` unless the operator
  chose Light or Dark in About; the choice is remembered in
  `~/.config/lifting-plan-calculator/appearance.json`.
- The accent is the desktop's when it publishes one (`gsettings` on GNOME,
  `kreadconfig6/5` on KDE), and our own blue otherwise — one fallback per
  palette, because a light accent needs a darker pair.
- Windows keeps its Fluent layer, but takes its accent from the same Rust
  command, so the two platforms no longer have separate accent code.
- Filled buttons (primary, add, danger) now set their own text colour; they
  previously inherited the surface text token, which is unreadable on a light
  fill.

## What the council saw

- Rendered in the running app: the same screens in both palettes, at 1400 × 950,
  with the desktop accent in place. The light palette was captured with the
  stored preference, the dark one with "Match the desktop", which also proves
  the setting survives a launch.
- The accent path was exercised for real: the GNOME accent on the test machine
  (`blue`) arrived as `#3584e4` in the UI log and painted the primary buttons.
- The palette rules are pure functions with `node --test` cases: unknown
  preferences fall back to the desktop, an explicit choice always wins, an
  invalid or missing accent falls back per palette.
- The Rust side is split into mapping functions (`accent_from_name`,
  `accent_from_kde`) with unit tests, so a wrong colour is a test failure rather
  than a screenshot review.
- The gates are unchanged: 249 Rust tests, `clippy -D warnings`, the frontend
  suite, and the vendored-asset checks.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 8/10 | one token layer, no duplicated rules; the screen CSS never learns which palette it is on |
| Application Security Engineer | 8/10 | the setting is a single JSON value with a fixed vocabulary, written under the operator's own config directory; the accent arrives as a validated `#rrggbb` and is only used as a CSS colour |
| QA & Test Engineer | 8/10 | both palettes photographed in the running app, the persistence path exercised, and the pure rules covered by tests |
| Product Manager | 8/10 | the app stops looking like a dark island next to the rest of the desktop without gaining a theme editor nobody asked for |

**Mean 8.0/10** — at the bar.

## Known limits

- Wayland itself was not exercised: the captures ran under Xvfb, which is the
  same GTK/WebKit stack but not Hyprland. The palette depends on no window
  manager, so the risk is in decoration, not colour.
- Accent discovery covers GNOME and KDE. A desktop that publishes nothing gets
  our blue; there is no manual colour picker.
- Windows was compile-checked and its accent path reuses the existing DWM call;
  the Fluent layer was not re-photographed for this release.
- The report is printed on white and deliberately ignores both palettes.
