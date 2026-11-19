# 0031 — An explicit scheme owns every surface, and the window chrome

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.12
**Change:** `ui/js/theme.js`, `ui/js/app.js`, `ui/js/api.js`,
`src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
**Seats:** Frontend Engineer, Desktop Integration Engineer, QA & Test Engineer,
Product Manager, Release Manager. The Release Manager chaired; the implementer
did not score.

## Proposal

Two fixes for the appearance setting:

- When the operator chose Light or Dark, the resolved scheme repainted the
  stylesheet tokens but the desktop's exact palette was still applied to the
  tinted surfaces. On a dark Noctalia desktop, choosing Light left the
  backgrounds, text and borders dark, so the switch looked partially applied.
  The desktop palette is now used only when its scheme matches the resolved
  scheme; an override falls back to our own palette for that scheme, tinted from
  the wallpaper accent when one exists.
- On Windows, the native title bar and Mica kept following the system theme
  while the page changed. The window theme is now set with the page:
  `system` follows Windows, and an explicit Light or Dark applies to both.

## What the council saw

- **Frontend Engineer:** the bug reproduces with a dark desktop palette and
  `preference: "light"`. The log said `scheme=light ... source=wallpaper
  tinted=true`, while the page stayed dark. After the change, the same log
  produces a light tinted palette, and `Match the desktop` still uses the exact
  desktop colours.
- **Desktop Integration Engineer:** the Windows fix goes through Tauri's
  `Window::set_theme`, which sets the DWM title bar and Mica. `None` means
  "follow the system", so the default is unchanged. Other platforms keep their
  app-wide theme: the command is a no-op there.
- **QA & Test Engineer:** `node --test` → **19 passed**, including three new
  regression tests for the exact-palette rule; `cargo test` → **264 passed**,
  including the scheme-to-window-theme mapping; `cargo clippy --all-targets -- -D
  warnings` is clean. Light and Dark were inspected on the affected PC, and the
  saved preference was restored.
- **Product Manager:** the reason the same Noctalia setup behaved differently on
  two machines is now explainable: the bug needed a published palette whose
  scheme was the opposite of the chosen one. A machine with no published palette,
  or one where the choice matched the desktop, looked correct.
- **Release Manager:** a patch release. The window-chrome path is compiled by
  the Windows CI job; a physical Windows run remains the confirmation of the
  title bar and Mica.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 9/10 | the override and the desktop-follow path are now separate, with tests for both |
| Desktop Integration Engineer | 8/10 | the chrome follows the choice without changing the default system behaviour |
| QA & Test Engineer | 8/10 | the reported mismatch is reproduced and pinned by tests on both layers |
| Product Manager | 8/10 | Light and Dark now mean the same thing on every machine |

**Mean 8.25/10** — at the bar.

## Known limits

- The Windows title bar and Mica are still confirmed by a Windows build, not by
  the Linux preview; the CI job compiles the path but cannot see DWM.
- On Linux the window theme is app-wide and stays with the desktop; only the page
  palette is overridden.
- The desktop's exact palette is intentionally not transformed into the other
  scheme: an override uses our own ladder, with the wallpaper accent as its hue.
