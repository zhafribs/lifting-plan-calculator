# 0027 — Paint with the desktop's palette; keep headings with their first block

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.8
**Change:** `src-tauri/src/appearance.rs`, `src-tauri/src/report_style.rs`,
`ui/js/theme.js`, `ui/js/app.js`
**Seats:** Frontend Engineer, QA & Test Engineer, Product Manager. The Release
Manager chaired; the implementer did not score.

## Proposal

Two faults, both diagnosed from the operator's own files rather than a guess.

**The palette.** 0023 read the terminal's 16 colours and chose "the most vivid,
mid-light" one. On a pink wallpaper that scored the yellow (`#e6d989`) higher
than the pink (`#ffb3b2`) and the shell stayed warm-grey. The authoritative
palette was one level away: `~/.config/gtk-3.0/gtk.css` is a single `@import` of
`noctalia.css`, which declares `accent_color #ffb3b2`, `window_bg_color
#1a1111`, `window_fg_color #f0dede`, `headerbar_bg_color #271d1d`. The app now
follows that import, reads the roles first, and paints its surfaces from the
background mixed towards the text — the same warmth as the terminals and the
file manager.

**The page break.** The plan the operator saved ends page 1 with "2. LIFT INPUT
PARAMETERS". Reproduced in Chromium: that section is built in
`<section class="input-card">`, while the keep-with-next rule was scoped to
`.section-card`. The rule now covers every report heading and the block after it,
on both sides.

## What the council saw

- The broken case was reproduced from the operator's PDF (`~/Documents/lifting-plan.pdf`)
  and then in a synthetic document with the real stylesheet: before the change,
  page 1 ended on the heading; after, the heading moved to the page with its
  table. Page count unchanged (2 pages).
- Page cost on a six-section document: 4 pages before and after.
- The palette was verified against the operator's desktop: the running app logs
  `accent=#ffb3b2`, and the screenshot shows the pink accent and the warm dark
  surfaces of `#1a1111`.
- The desktop palette is only used when its background and text are valid
  `#rrggbb`; otherwise the previous hue-tinted fallback applies, and with no
  palette the neutral pair stays. Tests cover both rejections.
- 261 Rust tests, 29 frontend tests, `clippy -D warnings` clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 8/10 | one token function for the exact palette, one fallback chain, and the page rule is now stated once for every heading |
| QA & Test Engineer | 8/10 | both fixes were reproduced from the operator's files first, then re-measured after the change |
| Product Manager | 8/10 | the app finally looks like the desktop it runs on, and a printed section no longer starts with a stray title |

**Mean 8.0/10** — at the bar.

## Known limits

- The GTK roles are read from a generated stylesheet; a desktop that sets its
  palette only in a compiled GTK theme resource falls back to the accent path.
- The 20-second poll only runs while the window is in front; a wallpaper changed
  while it is hidden shows up when it is next focused.
- The measured-pagination method from 0026 remains unbuilt; the widened rule
  fixes the reported shape without a measuring pass.
