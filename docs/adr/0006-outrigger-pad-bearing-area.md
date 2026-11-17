# ADR-0006 — Outrigger pad sizing uses the presumed bearing area

- **Status:** accepted, 2026-11-17
- **Owners:** Chief Engineer (crane and rigging), Safety & Compliance
- **Applies to:** `outrigger::solve_outrigger`, the Outrigger Pad Size tab, the printed plan

## Context

A lift plan needs a bearing pad under each outrigger, but the app had no way to
size one. The requested method is the published preliminary-design expression:

`A = 0.65 x (CM + L) / V`

where `CM` is the crane's operating mass in kN, `L` is the lifted load in kN and
`V` is the maximum permissible safe bearing capacity in kPa. The ground values
come from BS 8004 Table 1, adapted for Malaysian practice, and the same table is
supplied as an Excel example for users who want the reference outside the app.

The expression gives a **bearing area**, not a complete foundation design. It
does not size the pad thickness, the steel plate, the timber grade, the soil
profile or the crane manufacturer's outrigger reaction.

## Decision

- The app carries the 20-row BS 8004 Table 1 ground table and asks the user to
  choose a category and description. The published value loads into `V` and can
  be replaced with a site-specific value.
- A published range loads its **lower bound** for preliminary sizing. This is
  conservative and is stated on screen rather than hidden.
- Rows marked **Not applicable** are refused. Peat, organic soils and
  uncontrolled fill are not made to look like a shallow-pad solution.
- The result is the required area, the minimum square pad rounded up to the next
  50 mm, the selected pad's applied pressure and its utilisation. A pad above
  100% is shown as over the limit; 90–100% is caution.
- The screen and report both state that the calculation sizes the bearing area
  only and that the remaining checks are separate.

## Consequences

- A user gets a preliminary pad area and a practical square size without a
  separate spreadsheet, and the result can be printed with the plan.
- A range is not a promise of the upper value; the lower bound is used until a
  geotechnical assessment supplies a site-specific figure.
- The app will refuse a pad on unsuitable ground rather than inventing a value.
- The existing scope of use remains: the plan is a calculation aid, not a
  certified lift plan or a foundation approval.

## Alternatives rejected

- **Use the upper bound of every range.** Rejected: it is the least conservative
  reading and would make the pad look smaller than the preliminary table
  supports.
- **Calculate pad thickness from the same formula.** Rejected: the expression
  gives area only; thickness needs the pad material, plate design and the
  manufacturer's outrigger reaction.
- **Treat `Not applicable` as a very low bearing value.** Rejected: it would
  turn “use piling or ground improvement” into a pad that appears to pass.
