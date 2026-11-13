# 0003 — Printed-plan cards, PDF export, footer and working-range diagram

**Status:** accepted — owner override; the eight-seat re-score was not run for
this accumulated UI work
**Date:** 2026-11-14
**Release:** v2.5.3
**Change:** the working tree since v2.5.2 — the printed report, the PDF export,
the footer and the working-range diagram
**Seats:** the last full council pass was [0002](0002-plan-head-summary-card.md)
on the report head. The subsequent work — the head refinements, the card layout,
the PDF export, the footer and the diagram — was checked in the working sessions
but **not re-scored by the eight seats**. The owner directed the 2.5.3 release.

## Proposal

2.5.3 turns the printed plan into a consistent card layout (Plan details,
lift inputs and every following section), adds a real PDF export with fixed
one-inch margins, puts a footer on every page, and lets the Summary screen
attach the Graph tab’s working-range diagram to the plan. It also finishes the
2.5.2 head work: the crane name is inline, the six figures are cards, and the
two workings sit side by side.

## What was reviewed

- `cargo test --all-targets` → 186 passed.
- `cargo clippy --all-targets -- -D warnings` → clean.
- The dedicated PDF export was exercised with Chromium: A4, seven pages,
  content margins 25.4 mm on all four sides, footer `1/7` … `7/7`.
- The diagram is rendered offscreen from the crane/jib state, so it no longer
  needs the Graph tab to be open, and it refreshes on every report build.
- The footer’s centre text is editable in Summary and is applied through
  `@page` margin boxes; the plan reference and page counter are unchanged.

## Scores

Not re-run. The eight-seat 8/10 gate was not performed for this accumulated UI
change. This record is an **owner override**, not a council pass.

## Required changes

None outstanding; the owner accepted the release as it stands. The known
residuals are recorded rather than waived silently:

- The printed plan’s foot no longer carries the scope sentence; it remains in
  the README’s *Scope of use*.
- *Save as PDF* depends on Chromium (Linux) or Edge (Windows); the app returns an
  actionable error when neither is present.
- The diagram is the Graph tab’s geometry, not a scale drawing derived from the
  Crane tab’s three figures, and is inverted for print.
- The card layout is presentation only; no computed figure changed.

## Decision

**Accepted by owner override.** v2.5.3 is released on the owner’s direction. The
gate was not re-run; the next change should return to the normal eight-seat
review before it lands.

## Consequences

The printed plan now has a single card rhythm, a per-page footer and an optional
working-range diagram, and Summary can write a 1-inch-margin PDF directly. The
review debt for the 2.5.3 UI work is recorded here: a later council pass should
cover the PDF exporter’s failure paths, the diagram’s print appearance and the
removed scope sentence.
