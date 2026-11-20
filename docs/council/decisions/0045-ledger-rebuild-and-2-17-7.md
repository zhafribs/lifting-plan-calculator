# 0045 — The ledger rebuild, and 2.17.7

**Status:** accepted
**Date:** 2026-11-21
**Release:** v2.17.7
**Change:** `ui/js/screens/overall.js`, `ui/js/components.js`, `ui/css/app.css`,
`ui/css/windows.css`, `ui/css/tokens.css`, `src-tauri/src/outrigger.rs`,
`CHANGELOG.md`, `docs/releases/v2.17.7.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

After the item rows had been adjusted field by field, a product designer and a
frontend engineer reviewed them independently. The designer proposed a ledger
(name and result on one line, working beneath, hairlines between entries) and
an equation-style alternative; the ledger was chosen. The engineer measured
the real widths and showed that a single-line row cannot fit at any supported
window, so two tiers is the correct shape. The same pass removed a report note
that repeated its own row.

## What changed

1. **Item rows are ledger rows.** Identity and subtotal share the first line;
   the working (measured-by, quantity, measurement) sits beneath. No nested
   cards, a hairline between entries, one right edge for every subtotal.
2. **Measurement boxes are fixed to their figures.** `00.0` for m/m3,
   `0000.0` for kg/m and kg/m3; the pair never stacks — it moves to its own
   line when the working line is too narrow.
3. **The description looks typeable** (soft field edge) and the add row is
   dashed in the accent colour.
4. **The value scale is shared.** `kvRows` splits a formatted figure from its
   unit once, so every result list uses 600-weight figures with a small muted
   unit.
5. **The outrigger note is gone.** The `Stiffeners` row already carries the
   rule; the prose duplicate was removed (record 0044).

## Evidence

- 728×699 with the navigation expanded and 1440×950: fixed, per-metre and
  per-volume lines; the pair side by side; the select label whole; the summary
  and result lists with the small unit.
- `cargo test --all-targets` 268 passed, `cargo clippy --all-targets -- -D
  warnings` clean. `node --test` 38 passed, `node --check` on every module.
- Release notes and the tag record checked with
  `packaging/check-release-notes.sh --tag v2.17.7`.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | no figure, limit or verdict changed; the removed note is still stated in the Stiffeners row |
| Security & Build | 8/10 | UI text and layout only; the select stays native under the app-drawn list |
| Design & Steward | 8/10 | one grammar for the rows, one figure scale for the app, and the narrow window keeps every label |

**Mean 8.0/10** — at the bar.

## Known limits

- The quantity is named by the faint `×` and its position; the weakest label in
  the design.
- No automated visual test covers the rows; the states were checked by hand at
  the two widths.
- The removed report note remains in older printed plans; a fresh print is the
  way to see the change.
