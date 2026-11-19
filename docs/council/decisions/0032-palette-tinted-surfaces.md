# 0032 — Palette-tinted surfaces and Add buttons

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.13
**Change:** `ui/js/theme.js`, `ui/css/app.css`, `ui/css/tokens.css`,
`ui/css/windows.css`, `ui/js/theme.test.mjs`
**Seats:** Frontend Engineer, UI/UX Designer, Accessibility Reviewer, QA & Test
Engineer, Product Manager. The Release Manager chaired; the implementer did not
score.

## Proposal

Two complaints from the operator:

- The **Add load item / Add tackle item** buttons were a fixed green, unrelated
  to the desktop palette. They were especially out of place on a lavender or
  brown Noctalia desktop.
- An explicit **Light** or **Dark** choice read as flat white or flat black: the
  accent tint on the app's own surface ladder was almost invisible.

The change makes the Add buttons accent-toned (soft accent fill, accent border,
normal text), and strengthens the accent tint used by explicit Light/Dark so a
brown accent gives cream/light brown and dark brown, and lavender gives its own
light/dark pair. `Match the desktop` still uses the desktop's exact palette.

## What the council saw

- **Frontend Engineer:** the palette functions now return a visibly tinted
  ladder; `Match the desktop` and the explicit alternatives stay separate. The
  now-unused `--add` token is removed from both palettes and the Windows layer.
- **UI/UX Designer:** the Add buttons read as a palette action instead of a
  separate green system. Success, caution and over-capacity bands keep their
  semantic colours, so meaning is not lost.
- **Accessibility Reviewer:** the Add label uses the normal text colour on the
  accent-tinted fill, which keeps contrast for pale accents; the fill is not the
  only cue, the border and label remain. Focus rings are unchanged.
- **QA & Test Engineer:** `node --test` → **20 passed**, including the explicit
  Light/Dark tint and the exact-palette rule; `cargo test` → **264 passed**;
  `git diff --check` is clean. Light and dark were inspected on the affected PC
  at 1440×950 and 720×540.
- **Product Manager:** the result is a cohesive shell on a brown, lavender or
  blue desktop without a per-desktop setting.
- **Release Manager:** a styling-only patch; the report and arithmetic are
  untouched. The Windows layer already used the accent for Add, so it only
  aligns the text colour and drops the green token.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 8/10 | one palette path, one button rule, no dead token |
| UI/UX Designer | 8/10 | add actions now belong to the shell; status colours keep their meaning |
| Accessibility Reviewer | 8/10 | label contrast does not depend on a pale accent |
| QA & Test Engineer | 8/10 | both palette rules and the visual result are pinned |

**Mean 8.0/10** — at the bar.

## Known limits

- The tint is strongest with a wallpaper accent (Noctalia, pywal, wallust). A
  desktop that publishes only an accent and no palette uses the neutral ladder
  for the surfaces; only controls carry the accent.
- The effect is a flat tint, not wallpaper blur; the compositor owns blur.
- The printed report is unaffected.
