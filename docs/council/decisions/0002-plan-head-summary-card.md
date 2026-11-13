# 0002 — A human-friendly plan head

**Status:** accepted
**Date:** 2026-11-13
**Release:** v2.5.2
**Change:** the uncommitted working tree — `src-tauri/src/report.rs`,
`src-tauri/src/crane.rs`, `src-tauri/src/report_capacity.rs`, `CHANGELOG.md`,
`README.md`
**Seats:** Domain Safety Engineer, QA & Test Engineer, Technical Writer. The
Release Manager chaired the summoning; the implementer did not score.

## Proposal

The printed plan's head was a dense bordered grid: a centred title, a five-row
sign-off table and a four-row crane table, every cell outlined, the verdict a
thin green strip. It was compact, but it read like a form and the reader had to
work down a wall of boxes to find the figures that decide the lift. This change
replaces it with a summary card: the title and crane, a scoped capacity-check
pill, the plan details as a quiet strip, six metric cards, the verdict sentence,
and the chart source as a caption. No computed value changes.

## What the council saw

- `git diff` of the three Rust files; `cargo test --all-targets` = **178 passed**
  (176 before, plus the pill-mapping and booked-below tests); `cargo clippy
  --all-targets -- -D warnings` clean.
- A rendered A4 page with the 25.4 mm print margin, the long-reference case and a
  filled sign-off. The one-third line is at 33.33% of the sheet.
- The band words are `UsageState::wording()`; the usage card reads
  `capacity_usage_percent`; the verdict string in `crane.rs` now says
  "of **chart** capacity at working radius".
- The error path renders `Crane check / NOT INCLUDED`; the blank path renders
  `Capacity check / NOT EVALUATED`, "Not evaluated", "Not entered",
  "not checked" and the engine's own "Enter the load chart to check capacity."

## Scores — first pass

| Seat | Score | What blocked it |
| --- | --- | --- |
| Technical Writer | 6/10 | the booked explanation was circular and used "rated capacity" ambiguously; the verdict said "capacity" beside a booked-capacity card; the equation printed 16.9% where the card said 16.88% |
| Domain Safety Engineer | 7/10 | the scope-free green pill sat on the title line and read as plan approval, though it judges only the chart and booked checks |
| QA & Test Engineer | 7/10 | a long plan reference wrapped and put the head 3 mm past the top third; the caution/over/neutral pills were untested; the error head said "NOT EVALUATED" while the body said "not part of this plan" |
| **Mean** | **6.67/10** | all three below the bar |

## Required changes

1. **TW-1 — explain the check.** The note now reads: "Booked rating is the
   capacity on the crane's plate as booked. Chart capacity is what the chart
   allows at this radius. The booked check catches a machine that is too small at
   any radius." (ADR-0002's here-vs-at-all distinction, in plain words.) Done.
2. **TW-2 — name the capacity.** `crane.rs` builds "…% of **chart** capacity at
   working radius" for the within and caution verdicts, so the boldest line is
   not ambiguous beside the booked-rating card. Done.
3. **TW-3 — one value.** The card reads `capacity_usage_percent` and formats
   `{:.2}`; the usage equation also formats `{:.2}`. Card, verdict and working
   all print 16.88%. Done.
4. **SAFE-1 — scope the pill.** The pill moved off the title line into the crane
   row, under a "Capacity check" label, beside the crane model and
   configuration. The verdict sentence says "of chart capacity at working
   radius". The pill text comes from `UsageState::wording()`, so it cannot drift
   from the band. Done.
5. **QA-1 — the top third.** The five-column meta strip has explicit widths
   (Reference 30%, Issued 22%, the rest 16%) and the head's vertical rhythm was
   tightened. The long reference no longer wraps; the worst realistic case (long
   reference + timestamp + three ordinary names) ends 2.3 mm inside the line.
   Done.
6. **QA-2 — test the pill.** New tests cover caution → CAUTION, over → WARNING,
   the blank neutral pill and "not checked" note, and a booked rating below the
   requirement. Done.
7. **QA-3 — the error head.** The head prints "Crane check / NOT INCLUDED", the
   body prints "The crane check is not part of this plan.", and a test covers it.
   Done.

## Re-score

| Seat | Second score | Basis |
| --- | --- | --- |
| Technical Writer | 9/10 | explanation, verdict wording and the single 16.88% value all verified |
| Domain Safety Engineer | 8/10 | pill scoped, figures read from the engine, blank/error states honest |
| QA & Test Engineer | 8/10 | 178 tests, clippy clean, layout measured, all band paths exercised |
| **Mean** | **8.33/10** | every seat at or above the bar |

Residual, accepted and not blocking:

- Three unusually long sign-off names (20–29 characters) each wrap to a second
  line and put the card 0.8 mm past the one-third line. Two wrapped lines fit.
  The layout has no automated guard; a rendered-fixture comparison would close
  that gap.
- The usage equation's TeX label still reads "Chart capacity at radius" while
  the card says "Chart capacity" with "at this radius / boom angle". The
  qualifier is present either way.
- The empty states use "Not evaluated", "Not entered", "not checked" and "—";
  each is accurate, but a single vocabulary would be tighter.

## Decision

**Accepted.** Every participating seat scored 8 or more, mean 8.33. The change
is presentation only: no figure, verdict or limit changed. It keeps the plan's
scope statement, the engine's band words and the chart/booked distinction, and
makes the booked rating explain itself on the page.

## Consequences

The printed plan's head is now a card with a scoped capacity pill and six figure
cards; the title is "Lifting Plan", and the sign-off labels are shorter. The
verdict wording in `crane.rs` is part of the user-facing output and is now
"of chart capacity at working radius". The layout goal (head inside the top
third) is measured by hand, not by CI; a rendered comparison would be the next
step if the head is revised again. This decision relies on
[ADR-0001](../../adr/0001-rated-capacity-envelope.md) for the chart band and
[ADR-0002](../../adr/0002-booked-rating-check.md) for the booked check.
