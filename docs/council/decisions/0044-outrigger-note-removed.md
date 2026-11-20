# 0044 — Removing the repeated outrigger stiffener note

**Status:** accepted
**Date:** 2026-11-21
**Release:** next tag
**Change:** `src-tauri/src/outrigger.rs`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

The printed outrigger section already has a `Stiffeners` row — "required, or use
a 40–50 mm plate" — and the governing check names the steel plate width. Two
lines below, a note repeated the same rule in prose. Remove the note; keep the
flag and the row.

## What changed

`ThicknessResult.notes` no longer receives "A steel plate wider than 1.2 m ×
1.2 m needs at least 40–50 mm, or welded stiffening ribs." The `stiffeners`
flag is unchanged, and the report's own row still states the requirement. The
timber guidance is untouched.

## Evidence

- The reported PDF shows the row at line 46 and the duplicate note at 47–48;
  the row carries the same instruction.
- `cargo test --all-targets` 268 passed, including
  `a_wide_steel_pad_raises_the_minimum_and_flags_stiffeners`, which checks the
  flag and the 40 mm minimum, not the prose.
- `cargo clippy --all-targets -- -D warnings` clean.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | no figure, limit or verdict changed; the requirement is still printed in its own row |
| Security & Build | 8/10 | one string removed; the flag and the engine path are unchanged |
| Design & Steward | 8/10 | the section loses a duplicate instead of an instruction |

**Mean 8.0/10** — at the bar.

## Known limits

- A reader who only scans the notes paragraph will no longer see the rule there;
  it remains in the Stiffeners row two lines above.
