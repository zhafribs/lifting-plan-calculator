# 0008 — Saving a PDF cannot lose the plan you had

**Status:** accepted
**Date:** 2026-11-14
**Release:** v2.8.1
**Fixes:** the Windows export path added in [0007](0007-windows-prints-with-webview2.md)
**Change:** how a finished PDF replaces the file at the chosen path, where the
footer is written, and how the save's failure cases are proven

## Problem

The owner reported that the Windows build could not save a PDF to any location,
and asked why a routine dependency pull request failed.

Investigating turned up three defects, two of them introduced by 2.8.0:

1. **The move into place deleted the target first.** `move_into_place` removed
   whatever was at the chosen path, on the belief that Windows cannot rename over
   an existing file. It can — `fs::rename` maps to `MoveFileEx` with
   `MOVEFILE_REPLACE_EXISTING` — so that removal was unnecessary and
   destructive: when the following rename failed, the operator's previous plan
   was gone and the new one was not there either.
2. **The footer was stamped over the file the engine had just printed.**
   WebView2 can still hold its output when it reports completion, so writing into
   that file could be refused, failing every save.
3. **A dependency bot split a version-coupled pair.** It raised `windows-core` to
   0.100 on its own while `webview2-com` and tauri still build against 0.62, so
   the `Interface::cast` between the COM interfaces applied to a different crate
   instance and the Windows build stopped compiling.

## Proposal

- **One replacing rename.** The finished PDF is moved with a single rename, which
  replaces on both platforms. The retry applies only to Windows sharing and lock
  violations (codes 32 and 33), four times at 50–400 ms; a read-only target or
  folder fails immediately with a message that names the path and the reason.
- **The footer is written beside the print.** `pdf::stamp_to(source, target)`
  produces a new file, which is verified as a PDF and then moved into place. The
  engine's output is never opened for writing.
- **The probe proves the refusals.** Alongside the four successful locations
  (new file, replace, spaces and non-ASCII, nested), the export probe now forces
  two failures — a PDF held open by another program on Windows, and a folder that
  cannot be written to on Unix — and requires the earlier file to be
  byte-identical afterwards.
- **Windows runs the test suite.** The Windows CI job ran only `cargo check`; the
  Windows-only paths had no test coverage anywhere. It now runs `cargo test`.
- **The dependency pair is pinned by policy.** `webview2-com` and `windows-core`
  are excluded from dependabot, with the duty recorded beside the dependency:
  when tauri moves, move both to the versions its lockfile resolves.

## What was reviewed

- `cargo test --all-targets` → 203 passed, including a new test that an
  interrupted save's leftovers are swept while another plan's file and a save in
  flight are left alone.
- `cargo clippy --all-targets -- -D warnings` → clean.
- The probe on Linux: four locations and one refusal.
- The probe on Windows through WebView2 (CI): the same, plus the held-file
  refusal.
- Two defects the reviewers caught in my first attempt at this patch were fixed
  before release: retrying permanent errors, and never verifying the footered
  file.

## Scores

| Seat | Score | Position |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | Retried cleanup, the refusal cases and the recorded version duty close the first pass's 7 |
| QA & Test Engineer | 8/10 | Windows runs the suite, refusals are proven, and the notes claim only what is tested |

## Decision

**Accepted.** A failed save leaves the previous PDF in place, the footer cannot
destroy a print, and the Windows build is no longer at the mercy of an
independent dependency bump.

## Consequences

- A save that is refused now names the path and whether it is read-only or in
  use, instead of a generic write error.
- Leftovers beside a plan from an interrupted save are removed on the next save
  of the same name, once they are an hour old.
- Network shares, OneDrive-redirected folders and paths near Windows' length
  limit remain unexercised by CI; they fail with a message rather than a crash,
  and the release notes say so.
- `webview2-com` and `windows-core` no longer update automatically. That is a
  deliberate duty: a tauri bump must move them together and remove the ignore.
