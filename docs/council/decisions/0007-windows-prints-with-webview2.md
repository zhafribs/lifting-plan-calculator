# 0007 — Windows prints with WebView2; the page view follows typing

**Status:** accepted
**Date:** 2026-11-14
**Release:** v2.8.0
**Amends:** [0006](0006-report-preview-is-the-pdf.md) (the page view is no longer
"on demand, never while typing")
**Change:** the Windows PDF export path, the footer's owner, and when the
Summary page view redraws

## Problem

Two reports from the owner.

**Windows printing depended on the Edge browser.** *Save as PDF* spawned
`msedge.exe`. A machine where Edge has been disabled or uninstalled has a working
app — Tauri needs the WebView2 *runtime*, not the browser — but no way to save a
PDF.

**The page view lagged behind typing.** 2.7.0 refreshed it only when a field was
left, a section toggled, or the tab opened. In practice the operator typed a
reference and looked at a page that had not caught up.

## Proposal

- **Windows prints with the app's own engine.** The report loads in an off-screen
  Tauri window and the export calls `ICoreWebView2_7::PrintToPdf` on the WebView2
  the app is already running on. Edge and Chrome remain the fallback when the
  runtime cannot print.
- **The app owns the footer on both platforms.** The print copy of the report is
  generated without its `@page` margin boxes, and `pdf::stamp` writes the
  reference, credit and page counter onto every page. No engine version can
  decide whether a plan carries them, and Linux and Windows produce the same
  footer.
- **The page view redraws after a pause.** About a second after typing stops
  (900 ms after the 250 ms rebuild), guarded by the existing single-flight gate
  and the preview cache. Live stays instant.

## What was reviewed

- The four seats reviewed the branch at 6/10 each, and blocked on: Windows
  behaviour proven only by compilation; the footer left to the runtime; a print
  that could still be writing when the fallback reused its file; the in-app
  error discarded; no memory of a runtime that cannot print; and a 20 s + 60 s
  budget.
- All of them are addressed: each attempt prints to its own file inside the
  private workspace and is moved into place only when it is a real PDF
  (`%PDF-`); one 30 s deadline covers load and print; a failed runtime is
  remembered for the session; both failures are reported; and the footer is
  stamped on every platform.
- A new QA hook (`--export-pdf-test=<path>`) renders a probe report through the
  full command path and exits with the result. CI runs it on Linux under a
  virtual display and on Windows through WebView2, so the Windows export is
  exercised on every change rather than only compiled.

## Scores

| Seat | Score | Position |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | The Windows smoke test, the per-attempt file and the shared footer close the first pass's 6 |
| Application Security Engineer | 8/10 | Both errors kept, no lossy path, no shared-file race, policy text corrected |
| QA & Test Engineer | 8/10 | The probe runs on both platforms and the output is validated, not assumed |
| Product Manager | 8/10 | Version and notes corrected, the untested-platform claim replaced by evidence |

## Decision

**Accepted.** Windows exports without the Edge browser, both platforms draw the
same footer, and the page view keeps up with typing.

## Consequences

- On Windows the export briefly creates an off-screen window through WebView2.
  It is not visible and is closed after the print, but it is a window: a machine
  with an unusual window policy may still refuse it, in which case the Edge
  fallback runs.
- A runtime that cannot print is not retried for the rest of the session, so a
  broken installation costs the timeout once rather than on every typing pause.
- The preview now prints once per typing pause in Pages mode (about a second
  after the last keystroke), not only when a field is left. The preview cache
  means an unchanged plan is not printed twice.
- Linux behaviour is unchanged, including the footer position and the stamping.
- The browser's own Print button still prints the live HTML with its margin
  boxes; only the PDF export uses the stamped footer.
