# 0043 — Load and tackle lines as ledger rows

**Status:** accepted
**Date:** 2026-11-20
**Release:** next tag
**Change:** `ui/js/screens/overall.js`, `ui/css/app.css`, `ui/css/tokens.css`,
`ui/css/windows.css`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The load line had been adjusted field by field until it was a table that no
longer fitted its own card. Two independent reviews — a product designer and a
frontend engineer — were asked for a fresh row system. The designer proposed a
"Ledger" (identity and result on one line, the working beneath, hairlines
between entries) and an "Equation" alternative; the Ledger was chosen because
it keeps its grammar at the 720px window and reads as a schedule at 1440. The
engineer measured the real widths from the shipped Inter variable font and
confirmed that no single-line row can exist at any supported window, so two
tiers is the correct architecture.

## What changed

Each line is now two tiers inside one card, separated from its neighbours by a
hairline rather than a nested card:

```
steel plate                                          30,000.00 kg   ×
Weight by Volume ▾   ×   1   [ 12.5 m3 ]   [ 2,400 kg/m3 ]
```

- **Tier 1** is the identity: the description carries a soft field edge so it
  reads as typeable, the subtotal is the result at a fixed 110px column and 600
  weight against the 400-weight inputs, and the remove sits in a 30px gutter.
  Every subtotal in the card shares one right edge.
- **Tier 2** is the working: measured-by at 12px in the muted colour (its full
  label always fits), a faint `×`, a 46px quantity, then one or two value
  boxes. Each box is fixed to the figure it holds — measured from the bundled
  Inter at 13.5px: `00.0` for m/m3, `0000.0` for kg/m and kg/m3 — plus the
  input padding and the unit's reserved width (m 69, m3 76, kg/m 107,
  kg/m3 114, kg 96). They never stretch; if a line is too narrow the pair
  moves to its own line rather than clip.
- The subtotal renders its unit at 11px muted, so the figure, not the unit,
  carries the weight.
- The add control is a dashed accent row, no longer the loudest filled button in
  the card.
- The Lifting tackle card uses the same ledger with one value box (`Weight
  each`) and no measured-by, so its right edge matches the load card.
- The Weight summary uses the same scale: figures at 600 weight and a small
  muted unit, with the overall weight still the only coloured figure.
- The shared `kvRows` now splits a formatted figure from its unit once, so the
  crane, sling, outrigger and tandem results use the same scale as well.
- The column header, per-field label markup, grid templates and narrow-window
  fallbacks are gone; each input keeps an `aria-label` for its name.

## Evidence

- At 728×699 with the navigation expanded (card ≈430px): fixed, per-metre and
  per-volume lines all fit on one working line, the `Weight by Volume` label is
  whole, the `m3` and `kg/m3` boxes are side by side and readable, and the
  subtotal stays on the identity line.
- At 1440×950 with two load lines: hairlines separate the entries and the
  subtotal column aligns with the tackle card.
- The designer's measured widths (select 153px, subtotal 105px, pair 180–188px)
  were cross-checked against the rendered rows.
- `node --check` on every frontend module, `node --test` 38 passed,
  `cargo test --all-targets` 268 passed (no Rust changed).

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | fields, units and store keys unchanged; no calculation touched |
| Security & Build | 8/10 | markup and CSS only; the select stays native underneath the app-drawn list |
| Design & Steward | 8/10 | identity and result share an edge, the working is subordinate, and the row survives the narrowest window without clipping |

**Mean 8.0/10** — at the bar.

## Known limits

- The quantity is named by the faint `×` and its position; that is the weakest
  label in the design and should be watched on first use.
- There is no column header at the two-tier widths (the real 1440 card); the
  units and the full measured-by label do the naming.
- The two-tier row is taller than a table row: about 75px before the value
  boxes grow.
- Widths were derived from font metrics and checked by hand, not by an
  automated visual test.
- A two-box measurement never stacks: if the working line is too narrow, the
  pair moves to its own line. Summary rows use a content-sized value column so
  a long figure cannot run into its label.
