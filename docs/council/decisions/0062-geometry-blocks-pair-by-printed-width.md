# 0062 — Geometry blocks pair by their printed width

**Status:** accepted
**Date:** 2026-11-23
**Release:** v2.17.15
**Change:** `src-tauri/src/report.rs`, `src-tauri/src/report_uniform.rs`,
`CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The geometry grid decided "too wide for half a row" from the LaTeX source
length. Markup such as `\text{}` and `\frac{}{}` inflated that count, so
workings that fit a half-column were pushed onto their own full row. Pair them
by what actually prints.

## What changed

- A visible-width estimate ignores command names and braces, so only glyphs
  count.
- The split chooses its two-line break by that estimate, and marks a block
  full-row only when a line is genuinely wide.
- The bridle's Centroid Radius is forced to its own row as the first step,
  because nothing before it can pair with it.
- The angle and vertical height then share a row.

## Evidence

- The fixture report exports as 8 pages at 100% scale, down from 9, with the
  title still at 14 (100%).
- A test pins the bridle layout: the centroid row carries `data-wide`; the
  angle and height share one `<tr>` without it.
- `cargo test --all-targets` 282 passed; clippy clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | layout only; formulas and literals are unchanged |
| Security & Build | 8/10 | one width estimate and a row marker; no new call or write |
| Design & Steward | 8/10 | the page uses both columns where a working fits, and keeps the first step alone |

**Mean 8.0/10** — at the bar.

## Known limits

- The estimate is character-based, not a font measurement; a very wide glyph
  run can still take the full row, which is the safe direction.
