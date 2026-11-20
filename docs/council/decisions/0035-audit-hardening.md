# 0035 — Audit hardening: bounded downloads, atomic report save

**Status:** accepted
**Date:** 2026-11-20
**Release:** v2.17.1 (no new tag; hardening on `main`)
**Change:** `src-tauri/src/update.rs`, `src-tauri/src/commands.rs`,
`AGENTS.md`, `SKILLS.md`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

A three-seat audit of the app found two defects worth fixing before anything
else lands, and one finding that turned out to be wrong. Fix both, retract the
third with evidence, and adopt the gate itself in `AGENTS.md`.

## Findings and outcome

1. **Unbounded update download — fixed.** `fetch_bytes` read a body with
   `read_to_end`, and `verified_download` copied the asset to disk and then read
   it whole again to hash. Nothing capped the size, so an endless or huge body
   could exhaust memory or disk before the signature was ever consulted. The
   manifest is now capped at 1 MiB, the asset at 512 MiB, and the digest is
   computed in 64 KiB chunks.
2. **Oversized workbook — retracted.** The audit claimed `import_chart` parsed
   any `.xlsx` without a size limit. It does not: `excel.rs:167` rejects a file
   over 24 MB from `metadata` and again from a `take()`-limited read, and a test
   at `excel.rs:2362` already covers it. The finding was wrong and is withdrawn.
3. **Non-atomic HTML save — fixed.** `save_report` wrote the page straight to
   the chosen path, so a crash mid-write could leave a truncated report under a
   name the operator believed was saved. It now uses the same staging file and
   rename as the PDF path, with messages that name the report rather than the
   PDF.

## Evidence

- `cargo test --all-targets` → **268 passed** (266 before; two new tests cover
  the read cap and chunked hashing).
- `cargo clippy --all-targets -- -D warnings` → clean.
- `cargo audit --file src-tauri/Cargo.lock --deny warnings` → clean, 520 crates.
- The new cap is exercised with a body one byte over the limit; the chunked
  hash is compared against the existing whole-buffer digest on 200,000 bytes.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | two new tests at the seam; no engine figure touched; the retraction is recorded rather than quietly dropped |
| Security & Build | 8/10 | both unbounded reads removed; signature and digest order unchanged; audit and lockfile still clean |
| Design & Steward | 8/10 | error text still says what happened; no new dependency; one shared helper instead of a second copy of `take` |

**Mean 8.0/10** — at the bar.

## Known limits

- The cap bounds memory and disk, not wall-clock time; a slow endpoint can still
  hold the download open for `DOWNLOAD_TIMEOUT` (300 s).
- Workbooks are capped on compressed size; a hand-crafted 24 MB archive could
  still expand far beyond that inside the parser. Fuzzing `excel.rs` is the
  follow-up, not this change.
- The remaining audit findings (postMessage source check, `file_url`
  percent-encoding, poisoned-lock panics, reduced-motion in the base stylesheet)
  are unchanged and are ordinary cleanup.
