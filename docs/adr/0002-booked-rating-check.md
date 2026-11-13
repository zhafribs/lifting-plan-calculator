# ADR-0002 — A plan states two checks: the chart and the booked crane

- **Status:** accepted, 2026-11-13
- **Owners:** Chief Engineer (crane and rigging), Safety & Compliance
- **Applies to:** the Crane tab, `CraneResult`, the printed plan

## Context

The Crane tab checks the lift against the **load chart** at the working radius.
The crane's **booked rating** (the 90 t on the machine's plate) was published in
the header but never checked. The chart figure depends on the boom and radius in
use, so a machine can pass the chart check while still being too small for the
lift — the chart only answers “here”, not “at all”.

## Decision

The plan carries both checks:

1. **Chart check** — unchanged: gross load against the rated capacity at the
   working radius, banded by the 75% rule.
2. **Booked check** — the crane's booked rating against `gross / 0.75`, the same
   required capacity the header publishes.

The booked check **governs when it fails**: the status becomes
“WARNING — the crane's booked rating 10,000.00 kg is below the 26,666.67 kg
required at the 75% rule”, the band drops to *over*, and the plan prints
“BELOW the … required at 75%” in its own row. The chart figure stays visible, so
the operator sees both the machine that was booked and the capacity at the
radius.

No booked rating, or no load entered, means no booked check — the plan says
“Not checked” rather than inventing a pass.

## Consequences

- A crane that cannot do the lift at any radius cannot produce a *within limit*
  result.
- The chart check keeps its meaning; the booked check never hides it.
- The plan table gains one row, and the tab one row beside it.
