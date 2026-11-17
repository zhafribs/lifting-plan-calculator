# 0018 — The outrigger result is the minimum pad

**Status:** accepted
**Date:** 2026-11-18
**Release:** v2.13.2
**Change:** `src-tauri/src/outrigger.rs`, `src-tauri/src/report_outrigger.rs`,
the Outrigger Pad Size screen and report output
**Seats:** Release Manager, QA & Test Engineer, Technical Writer, Domain Safety
Engineer, Product Manager, Maintainer / Steward. The Release Manager chaired;
the implementer did not score.

## Proposal

The 2.13.x Outrigger Pad Size tab calculated a required area and then checked a
selected pad with applied pressure and a 90/100% utilisation band. That second
test is not part of the requested method: the formula itself gives the minimum
pad size. This change removes the selected-pad check and reports the minimum
pad directly:

- square: the minimum square side, rounded up to 50 mm;
- rectangular: the minimum length for the width the user enters.

The form is also reflowed: the bearing value and the published table value share
a row, and the crane/lift buttons sit on their own line.

## What the council saw

- **Domain Safety Engineer:** the formula, the BS 8004 table, the range default
  and the `Not applicable` refusal are unchanged. The output is now the minimum
  bearing size, which is what the published expression gives. The scope note
  still says the pad thickness, plate design, timber grade and the crane
  manufacturer's outrigger reaction are separate checks.
- **QA & Test Engineer:** `cargo test` → **239 passed**. New tests cover a
  rectangular pad's minimum length and a rectangular pad with no width. The UI
  was exercised with the example: 400 kN CM, 50 kN L, 150 kPa stiff clay,
  1.950 m², 1.40 m minimum square. The result card no longer shows a percentage
  or a pass/fail band.
- **Technical Writer:** the tutorial now describes the minimum square and
  rectangular result, and the README screenshot is refreshed. The release notes
  say the formula and ground table are unchanged and the old utilisation test is
  removed.
- **Product Manager:** the form now reads in the order of the calculation:
  ground, bearing value, loads, pad shape, minimum result. The 90/100% band was
  a second verdict the formula did not ask for, so removing it makes the screen
  honest.
- **Maintainer / Steward:** the result struct loses the unused pad-area,
  pressure, utilisation and status fields. No dependency or new network path is
  added.
- **Release Manager:** this is a patch release. The calculation formula, ground
  table and artifact format do not change; the result presentation and report
  text do.

## Decision

**Accepted.** Outrigger Pad Size reports the minimum square or rectangular pad,
and the selected-pad utilisation band is removed.

## Consequences

- A user who wants a larger pad can use one; the app does not mark it as a
  pass/fail.
- The report prints the minimum pad dimensions, not a utilisation figure.
- The 2.13.1 shortcut and linked-load changes remain.
- Still open, unchanged: Windows Authenticode, the documented `chrono` audit
  ignore, and reviewer coverage on the GitHub `release` environment.
