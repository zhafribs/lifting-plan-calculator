# 0022 — Follow the wallpaper without losing contrast

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.3
**Change:** `src-tauri/src/appearance.rs`, `ui/js/theme.js`, `ui/js/app.js`,
`ui/js/screens/about.js`, `docs/architecture.md`
**Seats:** Frontend Engineer, Application Security Engineer, QA & Test Engineer,
Product Manager. The Release Manager chaired; the implementer did not score.

## Proposal

On the operator's Hyprland desktop every native application follows the
wallpaper, because pywal/wallust publish a 16-colour palette that the rest of
the desktop reads. The app now reads the same palette:

- `~/.cache/wal/colors.json`, `~/.cache/wallust/colors.json`, then the plain
  `~/.cache/wal/colors`. `color0` (the background) and the greys are skipped.
- The **most usable** colour is chosen: vivid and mid-light, so it can sit on
  either surface. Ties keep the palette's own order.
- The accent is not the only thing that follows. The surface ladder is rebuilt
  in the accent's hue at our own lightness values, so the shell is tinted but
  the contrast is still ours, not the wallpaper's.
- The palette is re-read when the window is focused (throttled), so a wallpaper
  change does not need a restart.
- About says where the accent came from, so an unexpected colour is explainable
  rather than mysterious.

## What the council saw

- Both schemes were photographed in the running app with a sample palette
  (`#ff6b8a` chosen), and with no palette at all: the neutral shell is unchanged
  when nothing publishes a palette.
- The choice and the tint are pure functions with `node --test` cases: the
  palette's background and greys are rejected, the hue survives the tint, and
  text on background and surface stays at a 7:1 ratio or better for five
  different accents in both schemes.
- The Rust reader is tested against a pywal document, a near-monochrome palette
  and the plain list form; the JSON is parsed as data, never evaluated.
- 253 Rust tests and 28 frontend tests pass; `clippy -D warnings` is clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 8/10 | the tint is one pure function off the accent, and the neutral palette returns by clearing the same twelve variables |
| Application Security Engineer | 8/10 | the file is read from fixed cache paths, parsed as JSON, and reduced to a `#rrggbb`; nothing from it becomes a path, a command or markup |
| QA & Test Engineer | 8/10 | palette selection, tint hue and contrast are asserted, and both schemes were photographed with and without a palette |
| Product Manager | 8/10 | the app stops being the one window that ignores the wallpaper, without gaining a colour picker nobody asked for |

**Mean 8.0/10** — at the bar.

## Known limits

- Only pywal and wallust are discovered. matugen, the other common Hyprland
  generator, has no fixed output path; it is not read.
- A wallpaper palette can be garish. The app follows it on purpose; switching
  About to Light or Dark does not override the accent, only the scheme.
- The surface tint is deliberately low-chroma, so a vivid wallpaper reads as a
  tint rather than a full theme.
- Re-reading happens on focus, not on a file watch; a wallpaper change made
  while the app stays focused shows up at the next focus.
