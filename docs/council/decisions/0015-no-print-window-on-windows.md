# 0015 — Windows prints without a window

**Status:** accepted
**Date:** 2026-11-17
**Release:** v2.12.1
**Change:** `src-tauri/src/commands.rs`, `src-tauri/src/pdf_windows.rs`,
`CHANGELOG.md`
**Seats:** Build & Platform Engineer, QA & Test Engineer, Product Manager. The
Release Manager chaired; the implementer did not score.

## Proposal

The Summary page view prints the report to a PDF on every refresh. On Windows
that print used a WebView2 window created at `-32000, -32000` with no
decorations, on the theory that off-screen is invisible. It is not reliably
invisible: Windows could present it, so the operator saw a white window holding
the report for about a second on every refresh, then it vanished and the page
view updated.

The fix prints with **headless Edge first**. It opens no window at all, and
Windows 10/11 always carries Edge. The app's own WebView2 printer stays as the
fallback for a machine where Edge has been removed, and that path now creates
its window hidden and shows it only after the report has loaded.

## What the council saw

- The Windows CI job runs the export probe through the real app and passed on
  the new path: a PDF was written, carries its footer and has pages.
- The Edge path already existed as the fallback and is exercised by the same
  probe; it uses the report's own `@page` margins, which carry the operator's
  page-margin choice, and the footer is stamped afterwards as on Linux.
- No Linux code path changed; the 226-test suite is unchanged and green.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | no window is created on the common path, and the fallback is still there for a machine without Edge |
| QA & Test Engineer | 8/10 | the Windows job exercises the new first choice end to end; the fallback is compile-checked only, which is stated |
| Product Manager | 8/10 | a one-second white flash on every refresh is exactly the kind of thing an operator reads as a fault |

**Mean 8.0/10** — at the bar.

## Known limits

- The WebView2 fallback cannot be exercised on a runner with Edge present; it is
  compile-checked and unchanged except for the hidden window.
- Edge's headless print is a different renderer from the app's WebView2; the
  report is plain HTML with its own CSS and bundled fonts, so the pages match,
  and the Windows probe checks the output is a real PDF with a footer.
