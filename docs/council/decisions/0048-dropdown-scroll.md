# 0048 — The dropdown list keeps its wheel

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `ui/js/components.js`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The app-drawn list was closed by a capture-phase `scroll` listener on the
window. Scrolling the list itself fires that listener too, so a long list —
the 19 sling arrangements — closed on the first wheel notch.

## What changed

The listener ignores a scroll whose target is the menu (or inside it), and
still closes the list when the page behind it scrolls.

## Evidence

- Uniform Load, arrangement list open: five wheel notches over the list moved
  it from the top entries to the bridle entries, with the list still open and
  its scrollbar visible.
- `node --check` and `node --test` 38 passed; no engine change.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | UI only; no figure or calculation touched |
| Security & Build | 8/10 | one listener guard; no new call or dependency |
| Design & Steward | 8/10 | the long list scrolls as a list should, and the page still closes it |

**Mean 8.0/10** — at the bar.

## Known limits

- The list still closes on a page scroll, resize or outside click; that is
  deliberate.
