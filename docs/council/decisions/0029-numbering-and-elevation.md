# 0029 — Numbers that follow the plan; an installer that can be started

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.10
**Change:** `src-tauri/src/report.rs`, `src-tauri/src/report_uniform.rs`,
`src-tauri/src/update.rs`, `ui/js/screens/report.js`
**Seats:** Build & Platform Engineer, QA & Test Engineer, Product Manager. The
Release Manager chaired; the implementer did not score.

## Proposal

- **Renumber in print order.** Section builders wrote their own numbers (`2. LIFT
  INPUT PARAMETERS`, `3. GEOMETRY`, …). Inserting the outrigger pad produced two
  "2." headings and left everything after it out of step. `build_report_html`
  now renumbers every numbered `<h2>` as it joins the parts, so the numbers
  always match the cards the operator sees. The uniform inputs also name their
  arrangement: `LIFT INPUT PARAMETERS - UNIFORM LOAD`.
- **One sling arrangement.** Uniform, Nonuniform and Tandem are three
  descriptions of the same lift. Ticking one now clears the other two, including
  their page-break ticks.
- **Windows 740.** The operator's machine reported `os error 740` — the
  per-machine setup cannot be created by a non-elevated process. The app now
  starts it with `ShellExecuteW` and the `runas` verb, which raises the UAC
  prompt; `msiexec` keeps its own path, since the installer service handles the
  elevation.

## What the council saw

- The renumbering is a pure function with a test that feeds colliding numbers
  (1, 2, 2, 3, 6) and gets 1–5 back, including a heading without a number being
  left alone.
- Only one of the three arrangement checkboxes stays ticked; checked in the
  running app (Uniform then Nonuniform leaves Nonuniform alone, and the preview
  shows `2. NONUNIFORM LOAD`).
- 263 Rust tests and 29 frontend tests pass; `clippy -D warnings` is clean, and
  the Windows job compiles the `ShellExecuteW` path.
- The release manifest was verified independently: the checksums are signed with
  the embedded key and list both Windows installers, so the 740 error was the
  launch, not the download.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | the elevation path uses the shell verb Windows provides for exactly this, with no new dependency |
| QA & Test Engineer | 8/10 | numbering is unit-tested with the colliding case; the exclusivity and the renumbered heading were checked in the running app |
| Product Manager | 8/10 | the printed plan reads in order, and a Windows update can finally finish |

**Mean 8.0/10** — at the bar.

## Known limits

- The `ShellExecuteW` path is compile-checked and reasoned from the operating
  system's contract, not executed on a Windows desktop here; if the UAC prompt is
  declined, nothing is installed and the saved installer is named in the message.
- Renumbering rewrites `<h2>` numbers only; the document's own sub-headings
  (`<h3>`) are unaffected.
- The report still lays itself out; the New page ticks remain the operator's
  override.
