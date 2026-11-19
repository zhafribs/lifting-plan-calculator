# 0023 — Read the shell that is actually theming the desktop

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.4
**Change:** `src-tauri/src/appearance.rs`, `ui/js/theme.js`, `ui/js/app.js`,
`ui/js/screens/about.js`
**Seats:** Frontend Engineer, QA & Test Engineer, Product Manager. The Release
Manager chaired; the implementer did not score.

## Proposal

0022 read pywal and wallust and assumed that was "the wallpaper palette". The
operator's desktop is **Noctalia**, which themes the terminals and GTK
applications from the wallpaper but writes neither of those files:

- the palette goes into each application's own theme
  (`~/.config/kitty/themes/noctalia.conf`), and
- the light/dark choice goes into `~/.config/noctalia/config.toml` (`[theme]
  mode`), which GTK does not necessarily advertise through
  `prefers-color-scheme`.

Three additions:

- read a kitty theme file (`color1`…`color15`, background skipped) as a palette;
- read `@define-color accent_color` (then `accent_bg_color`,
  `theme_selected_bg_color`) from a generated `gtk.css`, which also covers
  matugen-style setups;
- read Noctalia's `[theme] mode` and let it win over the webview's guess for the
  "Match the desktop" choice.

## What the council saw

- Verified against the operator's own desktop, not a sample: the running app
  logged `source=wallpaper accent=#fad556 tinted=true` from their Noctalia
  palette and `scheme=dark` from their Noctalia config, and the screenshot shows
  the yellow accent and warm surfaces.
- The parsers are pure functions with unit tests: a kitty theme yields its most
  usable colour, a pastel-only palette still yields one of its own colours, a
  `@define-color` accent is read and a foreground is not, and Noctalia's mode is
  read from `[theme]` and not from another section.
- The scheme hint is a pure function (`desktopIsDark`) with tests, so the
  webview fallback stays covered.
- 257 Rust tests and 27 frontend tests pass; `clippy -D warnings` is clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 8/10 | the source order is one function, and the scheme hint is a two-line rule on top of the existing resolution |
| QA & Test Engineer | 8/10 | every parser is tested with the real file shapes, and the whole path was proven against the operator's desktop |
| Product Manager | 8/10 | the app stops requiring a specific palette generator to look like the rest of the desktop |

**Mean 8.0/10** — at the bar.

## Known limits

- The source order is fixed, not configurable: a desktop that runs two generators
  gets the earlier one.
- Only the kitty theme is read, because that is what Noctalia writes by default.
  A setup that disables the kitty template but keeps GTK falls back to `gtk.css`;
  with neither, the desktop accent or our blue.
- A palette change still arrives on focus rather than instantly.
