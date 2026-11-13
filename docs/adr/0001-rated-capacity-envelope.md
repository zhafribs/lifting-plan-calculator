# ADR-0001 — Rated capacity comes only from a charted point

- **Status:** accepted, 2026-11-13
- **Owners:** Chief Engineer (crane and rigging), Safety & Compliance
- **Applies to:** `calc::lookup_chart_capacity`, the Crane tab, the printed plan

## Context

The chart lookup carried the nearest charted capacity in from either end of a
load chart: below the first radius and beyond the last one, the lookup returned
the closest row's figure. That kept a number on screen for a half-entered form,
but it answered a question the manufacturer never rated:

- a single manual row of `10.00 m / 16 000 kg` answered for **every** radius;
- the shipped worked example read 8.00 m against a row charted at 10.00 m and
  published 12.5% — `WITHIN LIMIT`;
- a jib check at a boom angle below the jib sheet's lowest angle clamped to the
  sheet's **highest** capacity.

Load charts and the standards behind them (ASME B30.5, BS 7121, EN 13000) state
capacity as a rating **at a specified radius**. Between charted points a linear
interpolation is the industry's own reading of the same table. Outside the
charted span there is no rating, and a plan that prints one is asserting a
figure nobody published.

## Decision

A lookup outside the chart's own span is refused. The refusal names the range
the chart rates, and it is a first-class result rather than a blank:

- the Crane tab shows *Could not check* with the sentence
  (“The working radius is outside the load chart, which rates 3.00 m to 8.00 m.”);
- the printed plan states the same refusal under *Crane capacity check*, so a
  plan cannot look like it simply had no crane section;
- interpolation between charted points is unchanged.

Only the chart in charge is read. A workbook row mirrored into the manual entry
is not a rating of its own, so it cannot refuse a radius the workbook is about
to answer — and the operator's own entry cannot be refused by a stale workbook
copy.

## Consequences

- A lift whose radius sits outside the chart now yields no usage band at all —
  the honest outcome, and the one an auditor expects.
- `example_crane_inputs()` charts the radius it reads (8.00 m), so the Load
  example demonstrates a valid plan instead of the old clamp.
- Refusals are covered by tests at each layer: `calc` (span and wording),
  `crane` (radius and jib angle), `report` (the refusal reaches the plan).

## Alternatives rejected

- **Publish the clamped figure with an “unrated” flag.** Rejected: the number
  is what gets read off a plan and typed into a permit; a flag beside it is not
  a control.
- **Keep the clamp.** Rejected by Safety & Compliance: it renders “not rated”
  and “within limit” identically, which is the highest-severity class of defect
  in this product.
