# 0041 — App-drawn dropdowns, and 2.17.5

**Status:** accepted
**Date:** 2026-11-20
**Release:** v2.17.5
**Change:** `ui/js/components.js`, `ui/css/app.css`, `CHANGELOG.md`,
`docs/releases/v2.17.5.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

On this desktop, opening a dropdown drew a platform window the height of the
card over the control, and the selected text appeared clipped. The same class
of window caused the tooltip defect fixed in 0036. Replace the dropdown popup
with a list the app draws, keeping the native `<select>` underneath.

## What changed

- `selectInput` keeps the `<select>` for value, focus, keyboard and the change
  event. It now blocks the platform popup on pointer and on Enter/Space/Alt+Down.
- One shared `role="listbox"` panel is created on first use, populated from the
  select's own options and optgroups, and positioned under the control (above
  when there is no room). It scrolls at 300px.
- Arrow keys move the highlight, Enter chooses, Escape closes, Tab closes, a
  click elsewhere closes, and resize/scroll closes. Choosing sets the select's
  value and dispatches the same `change` event the platform would, only when
  the value actually moves.
- The list uses the app's tokens, so it matches the dark and light shells and
  no longer looks like an OS menu.

## Evidence

- Reproduced the defect in the window that ships the bug (XWayland, 720×540):
  the platform list covered the row and the closed control showed only "ed".
- After the change, at 720×540: the load-type list opens as a 190px panel under
  the select, selecting "Weight by Metre" and "Weight by Volume" updates the row
  and the subtotal; the 19-option sling arrangement list opens grouped ("Single
  leg", "Two legs"), scrolls, and selecting "Round choke" reveals the choke
  fields.
- `node --check` on every frontend module, `node --test` 38 passed,
  `cargo test --all-targets` 268 passed (no Rust changed).

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | the select still holds the value and fires the same change event; no field or calculation touched |
| Security & Build | 8/10 | no new call, write or dependency; one body-level panel and one set of listeners |
| Design & Steward | 8/10 | every dropdown gets the same compact list, groups and scrolling included; the platform popup is gone |

**Mean 8.0/10** — at the bar.

## Known limits

- Type-ahead (typing a letter to jump) is not implemented in the app-drawn
  list; the native select still handles arrows when the list is closed.
- Screen readers read the select's value, not the open panel's highlight; the
  panel carries listbox/option roles but focus stays on the select.
- The panel is positioned on open; it closes rather than following a scroll.
