# 0025 — One page for the controls

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.6
**Change:** `ui/js/screens/settings.js`, `ui/js/screens/about.js`,
`ui/js/app.js`
**Seats:** Frontend Engineer, QA & Test Engineer, Product Manager. The Release
Manager chaired; the implementer did not score.

## Proposal

0024 gave the appearance a Settings page but left Updates on About — the operator
asked for the obvious follow-up: the update controls belong with the other
controls.

- The Updates card moved to Settings, under Appearance, with every path it had:
  check, AppImage swap, Windows installer, Arch package, the polkit help and the
  manual `sudo pacman -U` command.
- About keeps the hero, what the app does, the developer and the credits.
- The launch reminder stopped carrying its own installer buttons. It now says a
  release exists, links to the notes, and opens Settings. That removed a second
  copy of the install logic from `app.js` — the copy that had already fallen
  behind (it never learned about the Arch package).

## What the council saw

- Both pages were opened in the running app: Settings shows Appearance then
  Updates; About shows hero, what this does, developer, credits.
- The reminder's buttons are three: Release notes, Show updates, Dismiss; Show
  updates goes to Settings.
- `app.js` lost its local `runInstaller` and `runAppImageUpdate`, so the update
  paths exist once.
- The Rust side is untouched: 261 tests, 27 frontend tests, `clippy -D warnings`
  clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 8/10 | fewer lines than before the move, because the duplication was deleted rather than relocated |
| QA & Test Engineer | 8/10 | both screens were checked in the running app, and the removal was mechanical after the move |
| Product Manager | 8/10 | there is one place to look for updates, and About answers the question it is named after |

**Mean 8.0/10** — at the bar.

## Known limits

- The polkit path is unchanged, including its untested-on-a-real-desktop prompt.
- The reminder still links straight to the release notes as well as to Settings;
  that is a link, not a second copy of the install logic.
