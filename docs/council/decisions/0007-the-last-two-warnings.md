# 0007 — The last two warnings in the release log

**Status:** accepted
**Date:** 2026-11-16
**Release:** v2.9.2
**Change:** `.github/workflows/release.yml`, `src-tauri/Cargo.toml`,
`src-tauri/tauri.conf.json`, `CHANGELOG.md`
**Seats:** Build & Platform Engineer, QA & Test Engineer. The Release Manager
chaired; the implementer did not score.

## Proposal

v2.9.1's CI was clean, but its release log still carried two warnings that the
CI workflow does not produce, because only the release runs those steps:

1. **A dbind accessibility warning** from the release's own PDF job. v2.9.1 set
   `NO_AT_BRIDGE=1` in the CI copy of that job and not in the release copy.
2. **A Node `Buffer()` deprecation** from `actions/download-artifact` in the
   publish job. The action is GitHub's; its code still calls the deprecated
   constructor, and the warning prints on every publish.

This release fixes both: the release PDF job gets the same environment as the
CI one, and the artifacts are fetched with `gh run download` instead of the
action, which keeps the draft-then-publish flow without the deprecation.

## What the council saw

- `gh run download <run> --name linux --dir dist` was exercised against the
  2.9.1 run: the AppImage and its `.zsync` land directly in `dist/`, which is
  what the asset check expects. The same call for `--name windows` covers the
  installers.
- The PDF job's environment now matches CI line for line.
- No application code changed: the same 206 tests, the same `cargo audit` gate
  (zero vulnerabilities), the same artifacts.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | both warnings come from steps only the release runs; the fix was verified where it could be (the download) and mirrored where it could not (the PDF env) |
| QA & Test Engineer | 8/10 | the release is itself the test: the run after the tag is checked for zero warning lines, and a regression would reappear in the same place |

**Mean 8.0/10** — at the bar.

## Known limits

- This is a pipeline-only release; the app is identical to 2.9.1.
- Third-party actions can print their own deprecations at any time. Where that
  happens, the council prefers replacing the action with `gh` over suppressing
  the message.
