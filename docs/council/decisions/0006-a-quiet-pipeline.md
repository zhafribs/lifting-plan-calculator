# 0006 — A quiet pipeline: audited dependencies, no compiler warnings

**Status:** accepted
**Date:** 2026-11-16
**Release:** v2.9.1
**Change:** `src-tauri/Cargo.lock`, `src-tauri/src/commands.rs`,
`.cargo/audit.toml`, `.github/workflows/ci.yml`,
`.github/workflows/release.yml`
**Seats:** Build & Platform Engineer, Application Security Engineer, QA & Test
Engineer. The Release Manager chaired; the implementer did not score.

## Proposal

The 2.9.0 logs carried warnings that would hide the next real one:

- **Seven vulnerabilities and four advisory warnings** from `cargo audit`, on a
  job that reported rather than gated, so the line "7 vulnerabilities found"
  was printed on every green build.
- **A Windows compiler warning** (`variable does not need to be mutable`) that
  only the Windows job could see.
- **A git default-branch hint** on every job, from `actions/checkout` running
  `git init`.
- **An accessibility-bus warning** per print in the PDF job.
- **A Node deprecation** from the third-party action that published releases.

This release clears all five and makes the audit a gate.

## What the council saw

- `cargo update` refreshed the vulnerable crates; `cargo audit --deny warnings`
  now exits 0 with no output at all.
- Two advisories are left and cannot be acted on here: `proc-macro-error`
  (unmaintained, no fixed upgrade, build-time only) and `glib 0.18` (unsound
  iterator the app never uses; the version is Tauri's GTK binding). Both are
  recorded in `.cargo/audit.toml` with their reasons, so the ignore is auditable
  rather than invisible.
- The `mut` warning is fixed by a cfg-split binding: Unix sets the directory
  mode, Windows has nothing to mutate.
- The git hint is suppressed with `init.defaultBranch=main` in the workflow
  environment (git reads `GIT_CONFIG_*` for the checkout action's `git init`
  too); the AT-SPI warning with `NO_AT_BRIDGE=1` in the two print steps.
- The publish step is `gh release create`/`delete` instead of a Node action, so
  the release keeps its draft-then-publish semantics without the deprecation.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | the audit is reproducible from the lockfile and its config, and the dependency bump is a `cargo update` with the suite re-run |
| Application Security Engineer | 8/10 | vulnerabilities went from seven to zero by upgrading, and the two remaining advisories are justified in one file with the reasoning attached |
| QA & Test Engineer | 8/10 | the audit gates (`--deny warnings`), the compiler warning was proven on the Windows job, and the whole run is re-checked for warnings after the change |

**Mean 8.0/10** — at the bar.

## Known limits

- The lockfile bump is a maintenance change: it is exercised by the full suite
  and the PDF export job, not by a new contract.
- `glib 0.18` stays until Tauri moves to GTK 0.20; the ignore must be removed
  when it does.
