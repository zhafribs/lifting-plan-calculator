# 0024 — Settings gets a page; a missing polkit gets an answer

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.5
**Change:** `ui/js/screens/settings.js`, `ui/js/screens/about.js`,
`ui/js/app.js`, `ui/js/dom.js`, `src-tauri/src/update.rs`
**Seats:** Frontend Engineer, Application Security Engineer, QA & Test Engineer,
Product Manager. The Release Manager chaired; the implementer did not score.

## Proposal

Two requests from the operator:

- The appearance controls did not belong on the About page. They now have a
  **Settings** tab in the rail, above About, with the scheme and a sentence
  saying where the accent came from. The caption is refreshed when the page is
  opened, because the palette is re-read while the app runs.
- An Arch install with no polkit could be updated, but the app only said "no
  prompt answered". It now names the missing piece and hands over the command:
  `polkit` when `pkexec` is absent, `hyprpolkitagent` + its user service on
  Hyprland, `polkit-gnome` elsewhere. The manual `sudo pacman -U` command is
  always shown beside it.

The three outcomes are distinguished from what pkexec reports: a missing binary,
"no authentication agent", exit 126 (the prompt was dismissed), and anything
else. Only the first two produce a suggestion; a cancellation is the operator's
answer, not a fault.

## What the council saw

- `auth_failure` and `setup_for` are pure functions with unit tests for all four
  failure kinds and both desktops; the suggestion for a cancelled prompt is
  asserted to be empty.
- `pkexec` is called with a fixed argv (`pacman -U --noconfirm <verified path>`);
  the commands the app *suggests* are only displayed, never run by the app.
- The Settings page, the rail entry and the icon were checked in the running
  app; About still renders its update card after the move.
- 261 Rust tests and 27 frontend tests pass; `clippy -D warnings` is clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Frontend Engineer | 8/10 | one screen per file; About lost the controls rather than growing a second copy of them |
| Application Security Engineer | 8/10 | suggestions are text with copy buttons; nothing is installed without an explicit operator action, and the argv stays fixed |
| QA & Test Engineer | 8/10 | every failure kind is tested; the no-agent and cancellation paths are the ones a minimal Hyprland install will hit |
| Product Manager | 8/10 | a missing dependency now has a next step instead of a dead end, and the manual route is never taken away |

**Mean 8.0/10** — at the bar.

## Known limits

- The polkit prompt itself was not exercised on a real desktop here; the
  detection is tested from pkexec's own messages and exit codes.
- The agent suggestion is Hyprland-or-generic; a desktop whose agent package has
  another name (LXQt, for instance) shows the generic `polkit-gnome` command.
- Package names are Arch's; other distributions keep their package managers'
  own updates.
