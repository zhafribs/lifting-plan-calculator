# 0016 — Outrigger pad sizing and the AppImage updates itself

**Status:** accepted
**Date:** 2026-11-17
**Release:** v2.13.0
**Change:** `src-tauri/src/outrigger.rs`, `src-tauri/src/report_outrigger.rs`,
the new Outrigger Pad Size screen, the BS 8004 ground table workbook, and the
Linux AppImage self-update
**Seats:** Release Manager, Build & Platform Engineer, Application Security
Engineer, QA & Test Engineer, Technical Writer, Domain Safety Engineer, Product
Manager, Maintainer / Steward. The Release Manager chaired; the implementer did
not score.

## Proposal

Two user-facing gaps are closed together:

- A lift plan has no way to size the bearing pad under an outrigger. The new
  **Outrigger Pad Size** tab uses the published BS 8004 Table 1 presumed
  allowable bearing values adapted for Malaysian practice, with the formula
  `A = 0.65 x (CM + L) / V`. It carries the ground row, the crane mass and lift
  load, a square or rectangular pad, the required area, the selected-pad
  utilisation and a printable report section.
- The Linux AppImage had no working in-app update: the reminder only opened the
  release page. The AppImage can now download the new file, prove the signed
  checksum, replace the running file and restart.

## What the council saw

- **Domain Safety Engineer:** the formula and the 20-row ground table match the
  supplied BS 8004 Table 1 extract. Ranges load their lower bound for
  preliminary sizing and remain editable for a site-specific value. `Not
  applicable` rows are refused for shallow pads. The result is labelled as a
  bearing-area calculation only; pad thickness, plate design and the crane
  manufacturer's outrigger reaction remain separate checks. The report carries
  the same scope language.
- **QA & Test Engineer:** `cargo test` → **241 passed**, including ten outrigger
  engine tests and two report tests. The worked example gives 292.50 kN design
  load, 1.950 m² required area, a 1.40 m rounded square and 48.75% utilisation
  on a 2.00 m pad. The UI was exercised with the ground picker, example, clear,
  rectangular pad controls and report. The AppImage update helper was exercised
  against a disposable copy of the official AppImage: it replaced the file,
  removed the `.old` backup and relaunched the new build.
- **Application Security Engineer:** the updater still accepts only this
  repository's HTTPS release assets. It verifies the signed `SHA256SUMS.txt`
  before reading a digest, refuses a downgrade, rejects path-like asset names,
  writes a new staged file rather than overwriting one, and writes the helper
  script with mode 0700 and a create-new operation. The helper passes the PID,
  target and replacement as positional arguments, so no path reaches a shell
  unquoted. The old file is restored if the new AppImage does not stay alive.
- **Build & Platform Engineer:** `cargo build` and `cargo clippy --all-targets
  -- -D warnings` are clean. No new crate was added; the helper uses the standard
  shell and `nohup`. The Linux AppImage and Windows installer packaging paths are
  unchanged.
- **Technical Writer:** README, tutorial, About screen, changelog and release
  notes now describe the Outrigger tab and the AppImage update. The release notes
  state the one-time manual install needed by older AppImages. Local link and
  anchor checks pass.
- **Product Manager:** the new tab removes a separate manual pad calculation and
  keeps it in the same plan and report. The update fix closes the only platform
  where the in-app button did nothing. The first manual 2.13.0 install is called
  out rather than hidden.
- **Maintainer / Steward:** the change follows the existing screen, DTO, solve
  and report conventions. The new maintenance surface is a small POSIX helper
  and one ground table; both are documented. The BS 8004 workbook ships beside
  the existing examples.

## Decision

**Accepted.** The Outrigger Pad Size tab and report section ship in 2.13.0, and
the Linux AppImage can replace itself from the update button.

## Consequences

- The outrigger tab is a preliminary bearing-area aid, not a foundation design.
  A user who needs a pad thickness, plate design or a site-specific bearing
  value must obtain it separately.
- Existing AppImages before 2.13.0 cannot use the new button; 2.13.0 is the
  one-time manual install that enables later in-app updates.
- The update helper leaves a diagnostic log at
  `/tmp/lifting-plan-calculator-update.log`.
- Still open, unchanged: Windows Authenticode, the dependency audit's remaining
  documented ignore for the unmaintained `chrono` advisory, and reviewer
  coverage on the GitHub `release` environment.
