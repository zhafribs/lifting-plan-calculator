# 0034 — AGENTS.md for coding agents

**Status:** accepted
**Date:** 2026-11-20
**Release:** v2.17.1
**Change:** `AGENTS.md` (new), `CHANGELOG.md`, version strings in
`src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.lock`
**Seats:** Technical Writer, Product Manager, Maintainer (chair). The implementer
did not score.

## Proposal

A repository with this much process — ported arithmetic tests, council records,
per-tag release notes, a hash-pinned vendored parser — loses that context when a
coding agent starts cold. `AGENTS.md` at the root states the commands, the
layout and the rules in one file an agent reads before editing.

The change touches no source file. It adds the guidance file, records it in
`CHANGELOG.md`, and bumps the version so the tag carries an accepted record of
the addition.

## What the council saw

- **Technical Writer:** every command in `AGENTS.md` was run before it was
  written down. `node --test` on the five webview-free files → **38 passed**;
  the `node --check` loop → **24 modules** clean; `cargo run`, `cargo test
  --all-targets` and `cargo clippy --all-targets -- -D warnings` are copied
  verbatim from `CONTRIBUTING.md` and `.github/workflows/ci.yml`. The release
  notes quote only changes that exist in the diff.
- **Product Manager:** the file changes no user-visible behaviour, so the
  release note says so in its first paragraph. Shipping a patch that a user can
  skip is honest; dressing a docs change as a feature would not be.
- **Maintainer:** one new root file, no new dependency, no build step, no
  renamed path. The version bump is mechanical and the release gate reads
  `tauri.conf.json`, which agrees with the tag.

## Evidence

- `./packaging/check-release-notes.sh --self-test` — passes.
- `./packaging/check-release-notes.sh` — every tag has a notes file.
- `./packaging/check-release-notes.sh --tag v2.17.1` — notes, version and this
  record agree.
- No file under `src-tauri/src/` or `ui/` is touched; `cargo test` was not run
  for this change and does not need to be.

## Decision

**Accepted.** Add `AGENTS.md` at the repository root and cut v2.17.1 as a
documentation-only patch.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Technical Writer | 9/10 | every documented command was executed; claims match the diff |
| Product Manager | 8/10 | scope is honest and the release note says "no application change" |
| Maintainer | 8/10 | one file, no new dependency, no path renamed |

**Mean 8.3/10** — at the bar.

## Known limits

- `AGENTS.md` mirrors CI. When a gate changes, the file must change with it; it
  is not generated from the workflow.
- The file is written for this repository's conventions and is not a general
  coding-agent policy.
- A documentation-only tag still builds both platforms. Nothing in the artifacts
  changed but the version string.
