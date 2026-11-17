# 0017 — Outrigger form polish and linked loads

**Status:** accepted
**Date:** 2026-11-17
**Release:** v2.13.1
**Change:** Outrigger Pad Size shortcuts, spacing, field labels, and the linked
crane/lift values
**Seats:** Release Manager, QA & Test Engineer, Technical Writer, Domain Safety
Engineer, Product Manager, Maintainer / Steward. The Release Manager chaired;
the implementer did not score.

## Proposal

The 2.13.0 Outrigger Pad Size tab worked, but its form was harder to read than
the rest of the app and its two load fields had to be typed separately from the
data already on the sheet. This change:

- gives the tab **Ctrl+3** and moves the later tabs to Ctrl+4 through Ctrl+8;
- separates the `V` row from the ground selectors, narrows the value field and
  sizes the `kPa` unit with the rest of the form;
- renames the fields to **Crane mass (CM)** and **Lift load (L)**;
- defaults `CM` to the Crane tab's rated capacity and `L` to the Overall Weight
  gross load. Editing a field breaks the link; the two buttons restore it.

## What the council saw

- **QA & Test Engineer:** `cargo test` → **241 passed**; `cargo clippy
  --all-targets -- -D warnings` is clean. The UI was exercised with a 90 t crane
  rating and a 4,350 kg overall lift: the tab showed 882.60 kN CM and 42.66 kN L,
  and the result followed after the ground and pad were entered. `Ctrl+3`
  opened the tab; the left pane showed Ctrl+4 through Ctrl+8 on the later tabs.
- **Domain Safety Engineer:** the formula, ground table, range default,
  `Not applicable` refusal and report section are unchanged. The linked values
  are display defaults only; the operator can still enter a site-specific crane
  mass or lift load, and the report prints the values actually used.
- **Technical Writer:** the tutorial shortcut table now names Ctrl+3 through
  Ctrl+8, and the Outrigger section explains the two links and the two buttons.
  The Outrigger screenshot was refreshed to the 2.13.0/2.13.1 form.
- **Product Manager:** the form now reads in the same order as the calculation:
  ground, bearing value, crane, lift, pad. The linked defaults remove a duplicate
  transcription without hiding the source.
- **Maintainer / Steward:** the link flags are presentation-only and stay in the
  JS state; the Rust input shape and the report are unchanged. No dependency was
  added.
- **Release Manager:** the version is a patch because no calculation, report
  verdict or artifact format changed. The 2.13.1 notes list the shortcut changes,
  the field changes and the unchanged scope.

## Decision

**Accepted.** The Outrigger Pad Size tab gains Ctrl+3 and the linked load
defaults, and the later tabs move to Ctrl+4 through Ctrl+8.

## Consequences

- Existing user muscle memory for Ctrl+3 (Graph) changes to Ctrl+4. The left
  pane shows the new shortcuts, and the tutorial documents them.
- A user who edits `CM` or `L` keeps that value until they press the matching
  button or reload the app; no plan is stored between launches.
- The outrigger calculation, ground table and report remain as reviewed in
  council record 0016.
