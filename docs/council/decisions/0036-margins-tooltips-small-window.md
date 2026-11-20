# 0036 — Margins, tooltips and the small-window rail

**Status:** accepted
**Date:** 2026-11-20
**Release:** next tag (no new tag; landed on `main`)
**Change:** `ui/css/app.css`, `ui/css/windows.css`, `ui/js/app.js`,
`ui/js/screens/report.js`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Run the built 2.17.1 app at the window sizes it allows, from the 720×540
minimum to 1600×1000, on every screen. Fix what the pass finds. Three defects
came out of it, all reported from the running app rather than from reading the
code.

## What was wrong

1. **The millimetre suffix sat on the native number spinner.** The Summary
   tab's page-margin fields are the only `type="number"` inputs in the app. The
   Windows stylesheet hid their spinner; the shared stylesheet did not, so on
   Linux the arrows were drawn under "mm". The shared stylesheet now hides it
   on both platforms, and the input carries the same `num` class as every other
   figure field, which reserves the unit's space.
2. **A tooltip was a large black window.** A `title` attribute is rendered by
   WebKitGTK as a window the height of the rail, with the text at one edge.
   `app.js` now swaps `title` for `data-tip` on the first pointer event, which
   stops the native window from opening, and draws a small bubble for pointer
   and keyboard focus. The report iframe uses `aria-label` for its name so it
   has no hover hint at all.
3. **The rail footer did not fit.** Under 1100px the footer wraps into a 54px
   column; the email broke mid-word, and at 720×540 the last line ran past the
   bottom edge. The contact line is hidden in the compact rail and remains on
   the About screen.

## Evidence

Checked by hand on the built 2.17.1 debug binary at **720×540, 800×600,
980×700, 1100×800, 1280×720, 1440×950 and 1600×1000**, with the real desktop
compositor:

- every screen (Overall, Crane, Graph, Outrigger, Uniform, Nonuniform, Tandem,
  Summary, Settings, About);
- the example crane loaded, 10,000 kg typed in, the capacity verdict reading
  "Warning — 625.00% of chart capacity";
- the margins panel at each width, a top margin of 40 typed and applied, with
  the hint following to 159 × 232 mm and the preview reflowing;
- the rail at the minimum size, with hover and with Tab focus.

Automated:

- `cargo test --all-targets` → **268 passed**.
- `cargo clippy --all-targets -- -D warnings` → clean.
- `node --check` on every frontend module, `node --test` → **38 passed**.
- `packaging/check-release-notes.sh` → 55 notes against 55 tags.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | no figure, limit or verdict changed; the margins still apply and the hint still follows the inputs |
| Security & Build | 8/10 | no new call, write or dependency; the tooltip only reads attributes already in the DOM |
| Design & Steward | 8/10 | one shared bubble replaces the platform's, the unit has the same treatment as every other field, and no new file |

**Mean 8.0/10** — at the bar.

## Known limits

- There is no automated visual test; the checks above were done by hand at the
  sizes listed. A CSS regression in a future change would be caught the same
  way.
- The tooltip has no delay on keyboard focus (deliberate) and no arrow pointing
  at its control.
- Below 1100px the contact name and address are only on the About screen; the
  rail keeps the product name and version.
