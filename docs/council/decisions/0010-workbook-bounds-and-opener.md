# 0010 — The workbook cannot exhaust memory, and Windows opens files without a shell

**Status:** accepted
**Date:** 2026-11-16
**Release:** v2.9.3
**Change:** workbook parsing limits, the workbook file-size check, and the
Windows desktop opener

## Problem

A full security audit of the repository (four hunters, every confirmed claim
verified against source; records in
`~/security-audit-skill/lifting-plan-calculator/run-1/`) found two medium issues
that need no unusual attacker capability.

1. **A workbook could abort the app.** `read_xlsx` called calamine's
   `worksheet_range`, which builds a dense array for the sheet's declared used
   area before returning. A file whose only cells are `A1` and `XFD1048576`
   declares 16384 x 1048576 cells, so the parser allocated ~17 billion entries
   (hundreds of GB) before this crate's limits ran. The realistic case is a
   malformed supplier chart, not an attacker.
2. **Windows opened files through `cmd`.** `open_with_desktop` built
   `cmd /C start "" <target>`. Rust quotes arguments by C runtime rules, not
   cmd's, so `&`, `|` and `%` in a file name or URL were parsed by cmd and a
   second command ran.

Two lower-severity issues were fixed in the same change: the 24 MB size limit
was applied after the whole file had been read, and the same `cmd` path served
links as well as files.

## Proposal

- **Stream the sheet.** Read cells through `worksheet_cells_reader` with a row
  ceiling (the existing 2000, now cutting the sheet rather than copying), a
  column ceiling (256) and a workbook-wide budget of 400,000 cells. A sheet over
  the column or cell ceiling is refused with a message; a tall sheet is cut, as
  before. No code path builds the declared rectangle.
- **Check the file before reading it.** `read_chart_file` stats the path, refuses
  anything over 24 MB, and then reads through `Read::take` so the cap holds even
  if the file grows.
- **Drop the shell.** Windows calls `ShellExecuteW` with the `open` verb. The
  target is an argument to a shell API, never a command line, so quoting rules
  stop mattering.

## What was reviewed

- `cargo test --all-targets` → 209 passed, including three new tests: a crafted
  workbook with a huge declared area is refused; a tall sheet is cut rather than
  allocated; a small sheet still parses cell-for-cell.
- A fourth test creates a sparse 24 MB + 1 byte file and asserts it is refused
  before being read.
- `cargo clippy --all-targets -- -D warnings` → clean.
- The existing Excel tests (both example charts) still pass unchanged, so the
  streaming reader preserves the parsed shapes the chart detect relies on.
- The Windows branch is compile-checked by CI's `x86_64-pc-windows-msvc` job; the
  injection is removed by construction because no shell is involved. Runtime
  opening was not exercised here (no Windows host).

## Scores

| Seat | Score | Position |
| --- | --- | --- |
| Application Security Engineer | 8/10 | Both medium findings closed at the root; the remaining audit items are tracked separately |
| QA & Test Engineer | 8/10 | Hostile fixtures cover the new bounds; the Windows opener is compile-only evidence and is labelled as such |

## Decision

**Accepted.** The two medium findings are fixed; the release chain findings
(signing, checksum verification, pinned build tools) are separate work and are
not claimed here.

## Consequences

- A workbook with a used area wider than 256 columns, or more than 400,000 cells
  across all sheets, is refused with "larger than a load chart". Real load
  charts are far smaller; the example charts are unaffected.
- Sheets taller than 2000 rows are still cut, as before.
- Windows now depends on the `windows` crate's `Win32_UI_Shell` feature for the
  opener. It is the same `windows` version the WebView2 bindings already use.
- The remaining audit findings — unverified installer download, self-referential
  checksum manifest, empty zsync signature sections, unpinned build tools, no
  CSP — are unchanged and still listed in the audit run.
