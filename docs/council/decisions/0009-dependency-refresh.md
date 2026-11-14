# 0009 — Taking the dependency refresh

**Status:** accepted
**Date:** 2026-11-14
**Release:** v2.8.2
**Change:** dependency versions only

## Problem

Two dependency pull requests were open. One carried seven Rust updates, including
four that touch code this project relies on — the PDF writer (`lopdf` 0.42 → 0.47),
the workbook reader (`calamine` 0.36 → 0.38), the file dialogs (`rfd` 0.17 →
0.18) and `zip` 8.6 → 9.5. The other moved `actions/download-artifact` from 7.0.0
to 8.0.2 in the release workflow.

An earlier attempt at the same group also moved `windows-core`, which broke the
Windows printer because `webview2-com` and tauri still build against the previous
version. That pair is now excluded from automatic updates (decision 0008), and
the regenerated pull request no longer touches them.

## Proposal

Take both. The question is not whether the versions are newer, but whether this
codebase still behaves — and that is answerable from evidence already in the
pipeline.

## What was reviewed

- 203 tests pass on Linux **and on Windows** (the Windows job now runs the suite).
  That includes the stamping tests against a committed WebKitGTK-printed plan,
  and the Excel tests over the two example charts.
- The export probe printed and stamped a real PDF with `lopdf` 0.47 on Linux, and
  the Windows probe did the same through WebView2 — four save locations and two
  refusals. So the new PDF writer is exercised on both PDF engines.
- Both pull requests were green on all four required checks before merging.
- The new `actions/download-artifact` is validated by this release itself: the
  publish job downloads the three build artifacts with it and publishes them.

## Decision

**Accepted.** Merged as-is, no source changes.

## Consequences

- `webview2-com` and `windows-core` remain at the versions tauri resolves. When
  tauri is bumped across a WebView2 binding change, they move together and the
  dependabot ignore is removed.
- A dependency-only release exists so the shipped binaries carry the updated
  libraries, and so the action bump was exercised now rather than at the next
  feature release.
