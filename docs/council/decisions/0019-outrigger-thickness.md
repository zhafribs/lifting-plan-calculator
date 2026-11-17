# 0019 — Outrigger pad thickness guidance

**Status:** accepted
**Date:** 2026-11-18
**Release:** v2.13.3
**Change:** `src-tauri/src/outrigger.rs`, `src-tauri/src/report_outrigger.rs`,
the Outrigger Pad Size screen and the Thickness sheet in the example workbook
**Seats:** Release Manager, QA & Test Engineer, Technical Writer, Domain Safety
Engineer, Product Manager, Maintainer / Steward. The Release Manager chaired;
the implementer did not score.

## Proposal

The Outrigger Pad Size tab sized the bearing area but said nothing about the
pad's thickness. This adds preliminary thickness guidance for steel plates and
hardwood timber mats, based on the supplied crane-class research, and cleans up
the printed section:

- steel plate and hardwood timber ranges by light, medium and heavy crane class;
- the 45° timber overhang rule when the outrigger foot dimensions are entered;
- cross-layering for timber pads wider than 1.2 m;
- 40–50 mm or stiffeners for steel plates wider than 1.2 m × 1.2 m;
- the materials that must not be used.

The report drops the introductory and closing paragraphs, moves the ground note
below the result and removes the “at 50 mm increments” phrase. The minimum-pad
sentence keeps its normal size. The same research is structured on the
**Thickness** sheet of the example workbook, not pasted as prose.

## What the council saw

- **Domain Safety Engineer:** the class bands, steel and timber ranges, 45°
  overhang rule, cross-layering, steel stiffener limit and material bans match
  the supplied research. The screen and report still say the thickness guidance
  is preliminary and that the pad build-up and crane manufacturer's outrigger
  reaction are separate checks.
- **QA & Test Engineer:** `cargo test` → **244 passed**, including light steel,
  wide-steel stiffener, 45° timber overhang and missing-capacity cases.
  `cargo clippy --all-targets -- -D warnings` is clean. The UI was exercised with
  the example: 400 kN CM, 50 kN L, 150 kPa stiff clay, 1.950 m², 1.40 m square,
  medium crane class and 40–50 mm steel.
- **Technical Writer:** the tutorial now covers the material choice, class
  ranges, 45° rule, cross-layering and steel stiffeners. The release notes say
  the thickness guidance is preliminary. The example workbook has a structured
  Thickness sheet with class, rule and selection-note tables.
- **Product Manager:** the tab now answers the next practical question after the
  bearing area: what to put under the outrigger. The thickness card is separate
  from the minimum-pad result, so the formula's output stays the primary figure.
- **Maintainer / Steward:** the thickness data is a small static table in
  `outrigger.rs`; no dependency or network path is added. The workbook is the
  user-facing copy of the same guidance.
- **Release Manager:** this is a patch release. The bearing-area formula and
  ground table do not change. The report presentation and the new thickness
  guidance do.

## Decision

**Accepted.** Outrigger Pad Size ships the steel/timber thickness guidance, and
the report text is simplified as requested.

## Consequences

- The thickness card is guidance, not a structural design; a user still needs
  the manufacturer's outrigger reaction and a competent pad build-up.
- A 45° timber check needs the outrigger foot dimensions. Without them, the card
  gives the class range and asks for the foot size.
- The example workbook now has a populated **Thickness** sheet beside the BS 8004
  ground table.
- Still open, unchanged: Windows Authenticode, the documented `chrono` audit
  ignore, and reviewer coverage on the GitHub `release` environment.
