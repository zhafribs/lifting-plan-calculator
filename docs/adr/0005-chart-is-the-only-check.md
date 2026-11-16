# ADR-0005 — The load chart is the only capacity check

- **Status:** accepted, 2026-11-16
- **Owners:** Chief Engineer (crane and rigging), Safety & Compliance
- **Supersedes:** [ADR-0002](0002-booked-rating-check.md)
- **Applies to:** `crane::solve_crane`, the Crane tab, the printed plan

## Context

ADR-0002 added a second check: the crane's rated capacity against
`gross / 0.75`. In use it duplicated the chart check and competed with it. The
chart figure is the rating that applies to the machine **at the boom and radius
in use** — a 35 t crane is not a 35 t crane at 30 m. A headline rating compared
against a capacity requirement at radius is not a second opinion; it is a
different question, and mixing the two made the verdict harder to read and the
plan longer.

## Decision

The chart check is the only capacity check. The crane's rated capacity is
entered (or taken from the workbook's title line), printed on the plan and in
the tab for the record, and **not** tested. The `CraneResult` no longer carries
`booked_ok` or a second required-capacity figure; the report's usage card keeps
one band, the chart's.

`gross / 0.75` is still printed as the design load `W` and as "Required capacity
at 75%", because the operator reads it against the chart; nothing is compared
against the crane's plate.

## Consequences

- The verdict has one meaning again: the chart at the working point.
- The Crane tab's capacity-check panel loses the "Booked rating check" row.
- A crane that is too small for the lift shows up where it always did: the
  chart cannot carry the load at that radius, or the radius is outside the
  chart.
- ADR-0002 is withdrawn, not deleted, so the reasoning stays auditable.
