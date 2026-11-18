# 0023 — Dropdown affordance and narrow load row

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.15.1
**Change:** `ui/css/windows.css`, the Windows screenshots, release notes
**Seats:** Release Manager, UI/UX Designer, Windows Platform Engineer, QA & Test
Engineer, Accessibility Reviewer, Maintainer / Steward. The Release Manager
chaired; the implementer did not score.

## Proposal

Fix two control-level defects reported against the 2.15.0 Windows build:

- a dropdown was indistinguishable from a text field, because the Windows layer
  reset the select chevron and its right padding;
- in the narrow two-line load row, with the rail expanded, the Qty field spilled
  under the Measured-by select.

## What the council saw

- **UI/UX Designer:** the two controls now differ at a glance. The select keeps
  a Fluent chevron and the Qty field keeps its own 62 px column; the select takes
  the remaining width instead of the Qty field overrunning it. At 1440×950 the
  wide row is untouched.
- **Windows Platform Engineer:** the fix is plain CSS. The chevron is the same
  gradient pair the base stylesheet has always used, so WebView2 renders it the
  same way; the narrow rule is inside the existing 620 px container query and is
  scoped to the Windows platform marker. No Tauri, CSP or window config changes.
- **QA & Test Engineer:** the overlap was reproduced at 720×540 with the rail
  expanded, and the fixed layout was inspected at the same size. The 1440 light
  and dark previews were recaptured, and the compact 720 capture was refreshed
  against 2.15.1. `node --test ui/js/windows-theme.test.mjs` → **5 passed**;
  `cargo test` → **246 passed**; `git diff --check` is clean.
- **Accessibility Reviewer:** the select remains a native `<select>` with its
  keyboard and screen-reader semantics; the chevron is an additional visual
  affordance, not the only cue. Focus rings and high-contrast rules are
  unchanged.
- **Maintainer / Steward:** two CSS rules and no new dependency. The narrow
  override is Windows-scoped, so the base Linux and macOS layout is untouched.
- **Release Manager:** a stylesheet-only patch. The version moves to 2.15.1, with
  no data or report change. The Windows 2.15.1 build remains the confirmation of
  WebView2 rendering.

## Decision

**Accepted.** Dropdowns show a chevron again, and the Qty field and select no
longer overlap in the narrow load row.

## Consequences

- Every Windows dropdown reads as a dropdown next to a text field.
- The narrow row is usable with the rail expanded at the 720×540 minimum.
- The calculation engine, report, PDF and Linux appearance are unchanged.
