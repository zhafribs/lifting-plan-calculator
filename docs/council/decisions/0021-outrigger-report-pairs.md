# 0021 — Outrigger report pairing and form separation

**Status:** accepted
**Date:** 2026-11-18
**Release:** v2.13.4
**Change:** `src-tauri/src/report_outrigger.rs`, the Outrigger Pad Size screen and
its styles
**Seats:** Release Manager, QA & Test Engineer, Technical Writer, Domain Safety
Engineer, Product Manager, Maintainer / Steward. The Release Manager chaired;
the implementer did not score.

## Proposal

Make the outrigger report easier to scan and stop the outrigger form's extra
fields crowding the shape row:

- Design load and required bearing area share one row; the bearing-area formula
  is a single equation.
- Pad thickness and ground share one row.
- Labels mark their variables: `Design (V)`, `Design load (P)`, `Required pad
  area (A)`.
- The rectangular width and timber 45° foot fields sit under their own labelled
  sections, and each load button sits under its matching field.

## What the council saw

- **Domain Safety Engineer:** no figure or limit changes. The symbolic formulas
  and substituted values are the same; the bearing-area equation is now a
  single chain, and the labels only mark the variables.
- **QA & Test Engineer:** `cargo test` → **246 passed**; `cargo clippy
  --all-targets -- -D warnings` is clean. The example was exercised at 400 kN
  CM, 50 kN L, 150 kPa and 1.950 m²; the result card shows the marked labels and
  the thickness card. The report section was inspected with the design/area and
  thickness/ground pairs.
- **Technical Writer:** the tutorial now says `Required pad area (A)`. The
  release notes describe the single-line formula and the field grouping.
- **Product Manager:** the report reads in two natural comparisons instead of a
  long stack, and the form's secondary fields no longer look like part of the
  shape row.
- **Maintainer / Steward:** the changes stay inside the existing report helper
  and screen; no dependency or new network path is added.
- **Release Manager:** this is a patch release. The calculation, ground table,
  thickness data and artifact format are unchanged.

## Decision

**Accepted.** The report pairs design/area and thickness/ground, and the form's
extra fields are separated under their own headings.

## Consequences

- The printed bearing-area formula is one equation line.
- The labels with `(V)`, `(P)` and `(A)` are the same variables used in the
  formulas.
- The `Use crane tab` and `Use overall weight` buttons follow their own fields.
- The 2.13.3 thickness guidance and update feed fallback remain.
