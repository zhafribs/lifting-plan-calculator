# 0001 — One release-notes file per tag

**Status:** accepted
**Date:** 2026-11-13
**Release:** v2.5.2
**Change:** the uncommitted working tree on `main` (README, CONTRIBUTING,
CHANGELOG, both workflows, `docs/releases/`, `docs/council/`, `packaging/*.sh`,
the printed report's footer, the About screen's closing paragraph)
**Seats:** all eight sat, each as a separate pass in a fresh context against its
own rubric; the implementer did not score. The owner authorised the passes on
`deepseek/deepseek-flash` after the default review provider refused them.

| Seat | Pass |
| --- | --- |
| Release Manager | independent agent pass, fresh context, 2026-11-13 |
| Build & Platform Engineer | independent agent pass, fresh context, 2026-11-13 |
| Application Security Engineer | independent agent pass, fresh context, 2026-11-13 |
| QA & Test Engineer | independent agent pass, fresh context, 2026-11-13 |
| Technical Writer | independent agent pass, fresh context, 2026-11-13 |
| Domain Safety Engineer | independent agent pass, fresh context, 2026-11-13 |
| Product Manager | independent agent pass, fresh context, 2026-11-13 |
| Maintainer / Steward | independent agent pass, fresh context, 2026-11-13 |

The release this record authorises is **v2.5.2**, the patch that carries the
printed footer and the About sentence. The 13 pages through v2.5.1 were
republished on 13 November 2026 as an operational step of the same change; this
record does not claim they were built from this tag.

## Proposal

On GitHub, the v2.5.1 release page repeated the notes of 2.5.0 and 2.4.0, so a
person downloading a version could not tell what had changed in it. The cause
was structural: `RELEASE_NOTES.md` held the last three versions and the publish
job used the whole file as the body. This change replaces it with one notes file
per tag, makes the release job publish exactly that file and refuse a tag whose
notes, version or council record disagree, backfills the 13 existing tags,
republishes the 13 live pages once, and records the process — plus the scope
statement the Safety seat required on the printed plan itself.

## What the council saw

- `git status --short` and `git diff`: 7 tracked files changed, `RELEASE_NOTES.md`
  deleted (staged), 4 new paths (`docs/council/`, `docs/releases/`,
  `packaging/check-release-notes.sh`, `packaging/publish-release-notes.sh`).
- The original defect, on the live page: `gh release view v2.5.1 --json body`
  contained `## 2.5.0` and `## 2.4.0` below its own notes.
- `cargo test --all-targets` → `test result: ok. 176 passed; 0 failed`.
- `cargo clippy --all-targets -- -D warnings` → finished with no warnings.
- `node --check` on every `ui/js/**/*.js` → `checked 15 modules`.
- `./packaging/check-release-notes.sh --self-test` → `self-test: all cases held`
  (10 negative fixtures, including the original cumulative body).
- `./packaging/check-release-notes.sh` → `checked 13 notes file(s) against 13
  tag(s)` / `release notes check: OK`.
- `./packaging/check-release-notes.sh --tag v9.9.9` → refused: tag version
  disagrees with `src-tauri/tauri.conf.json` (2.5.1).
- `./packaging/publish-release-notes.sh` (dry run) → `13 release page(s) would
  change; skipped 0`.
- Both workflows parse as YAML; the job graph is
  `verify-notes → {linux-appimage, windows-installers} → publish`.
- Relative-link scan over 27 Markdown files → 0 broken.

## Scores — first pass

| Seat | Score | Basis |
| --- | --- | --- |
| Release Manager | 6/10 | tag was never compared with the built version; 13 live pages unfixed; rerun overwrote assets |
| Build & Platform Engineer | 7/10 | `windows-installers` not gated; shallow clone printed "0 tags … OK"; `python3` missing from requirements |
| Application Security Engineer | 6/10 | workflow-level write permission; `sha256sum -c` fails for a single-file download; no sink validation; floating action pins |
| QA & Test Engineer | 6/10 | the checker passed the original defect; release gate weaker than CI; no negative tests |
| Technical Writer | 6/10 | network/file claims false; "can never repeat" overstated; stale screenshots; "a third shorter" unmeasured |
| Domain Safety Engineer | 4/10 | README described the 75% band as a cap and "two checks"; scope statement absent from the printed plan; veto did not cover presentation; tags not bound to a record |
| Product Manager | 6/10 | the 13 live pages were not repaired; governance in a release-notes fix; absences unstated; two headlines buried the point |
| Maintainer / Steward | 6/10 | second record convention beside `docs/adr/`; gate inoperable for a solo maintainer; typo exemption contradicted the charter; checker certified false asset names |
| **Council mean** | **5.875/10** | all eight seats below the bar |

## Required changes

1. **RM-1 — tag ↔ version.** `--tag` compares the tag with
   `src-tauri/tauri.conf.json` and fails on a mismatch; `verify-notes` and
   `publish` both run it. Done.
2. **RM-2 — live pages.** `packaging/publish-release-notes.sh` republishes the
   13 bodies from `docs/releases/`; dry-run by default, `--yes` to write;
   documented in `docs/releases/README.md`, CONTRIBUTING and README. Done; not
   run against GitHub (owner approval required).
3. **RM-3 — rerun.** Publish creates a **draft**, asserts the four named assets,
   uploads with `overwrite_files: false`, then flips `--draft=false`. A failure
   leaves a hidden draft. Done.
4. **BUILD-1 — fail fast.** `windows-installers` has `needs: [verify-notes]`.
   Done.
5. **BUILD-2 — shallow clone.** The checker exits non-zero when
   `git rev-parse --is-shallow-repository` is true, and outside a git
   repository. Done.
6. **BUILD-3 — requirements.** README names `python3` and why. Done.
7. **SEC-1 — permissions.** Workflow level is `contents: read`; only `publish`
   grants `contents: write`. Done.
8. **SEC-2 — single-file verification.** README uses
   `sha256sum -c --ignore-missing SHA256SUMS.txt`, says the file must sit beside
   the download, and that checksums detect corruption, not compromise. Done.
9. **SEC-3 — sink validation.** `publish` re-runs `--tag` before downloading
   artifacts or touching the release action. Done.
10. **SEC-4 — pins.** Every action in both workflows is pinned by commit SHA.
    Done.
11. **QA-1 — reject the defect.** The checker now rejects a foreign version
    heading, a foreign version token in `## Download`, a table with no asset, a
    substring version, an empty file, and a tag with no file. Done.
12. **QA-2 — one gate.** `verify-notes` runs the same script CI runs, with
    `--tag`; `publish` repeats it. Done.
13. **QA-3 — negative tests.** `--self-test` runs 10 fixtures; CI runs it before
    the repository check. Done.
14. **TW-1 — network and files.** README states the launch-time update check,
    the report save (HTML + KaTeX), the installer download, and `ui.log`. Done.
15. **TW-2 — "never repeat".** README and `docs/releases/README.md` say pages
    cut from this process cannot repeat notes, and that the 13 older pages still
    carry the old body until republished. Done.
16. **TW-3 — screenshots.** Labelled v2.0.0; the report capture is marked as
    predating 2.5.0 and 2.5.1. Done.
17. **TW-4 — unmeasured claim.** "A third shorter" is replaced by "packed into
    the top third"; v2.5.1 leads with the TL-200M reference-data correction.
    Done.
18. **SAFE-1 — describe the check truthfully.** README gives the band
    (≤75% WITHIN LIMIT, 75–100% CAUTION, >100% WARNING), the booked rating
    ("Not checked" when absent), and the out-of-chart refusal. "Two checks" and
    "max 75%" are gone. The band and the refusal rest on
    [ADR-0001](../../adr/0001-rated-capacity-envelope.md); the booked check on
    [ADR-0002](../../adr/0002-booked-rating-check.md). Done.
19. **SAFE-2 — scope reaches the plan.** `src-tauri/src/report.rs` prints
    "Calculation aid only… does not certify the lift, the crane, the equipment
    or its operation… check every figure against the manufacturer's load chart
    and your site's lift plan." Every notes file carries a scope line; the About
    screen no longer assigns responsibility for the figures to the developer.
    Done.
20. **SAFE-3 — summon and veto.** The charter summons the seat for `calc.rs`,
    `crane.rs`, `excel.rs`, `sling.rs`, `model.rs`, `nonuniform.rs`, the report
    builders, and any prose stating a figure, limit, check or verdict; the veto
    now covers presentation. Done.
21. **SAFE-4 — tag ↔ record.** `--tag` refuses a tag with no accepted record
    naming it exactly; CONTRIBUTING makes the record step 1 of a release. Done.
22. **PM-1 — repair the live pages.** Same as RM-2. Done in the repository;
    GitHub write still requires the owner.
23. **PM-2 — cut the council.** **Rejected by the owner.** The council is a
    standing instruction, not padding; the per-release cost is reduced to one
    notes file and one record, trivial changes take one seat, and the charter
    has a solo-maintainer path. The seat is asked to re-score whether that
    process is proportionate, not whether it should exist.
24. **PM-3 — state the absences.** No screenshots, no in-app changelog, no code
    signing — stated in `docs/releases/README.md` and README. Done.
25. **PM-4 — headlines.** v2.5.1 leads with the TL-200M correction; v2.1.5 no
    longer headlines a pixel breakpoint. Done.
26. **MAINT-1 — record stores.** Charter §"The record and the ADR" splits
    product decisions (`docs/adr/`) from review records (`decisions/`); the
    naming is `NNNN-<slug>.md` in both. Done.
27. **MAINT-2 — operable gate.** Chair named; independent pass defined;
    implementer never scores their own work; no independent pass, no landing;
    figure changes never accepted on one pass. Done.
28. **MAINT-3 — trivial changes.** One rule, stated identically in CHARTER and
    CONTRIBUTING: Maintainer seat alone, noted in the pull request, no numbered
    record. Done.
29. **MAINT-4 — checker asset names.** The Download section needs a real asset
    row and may not name another version; the self-test covers a copied table.
    Done.

## Re-score

The blocking seats were sent the revised tree with their own blockers and asked
to score independently against their rubric. The default review provider refused
two rounds ("Free usage limit reached"); the owner then authorised the passes on
`deepseek/deepseek-flash`, each in a fresh context.

| Seat | Second score | What moved it |
| --- | --- | --- |
| Release Manager | 8/10 | tag↔version gate, draft with `overwrite_files: false`, republish verified by dry run |
| Build & Platform Engineer | 8/10 | reproduced the shallow-clone and non-git failures returning 1; graph and pins re-verified |
| Application Security Engineer | 8/10 | job-scoped write permission, sink re-validation, `--ignore-missing`, 40-hex pins |
| QA & Test Engineer | 8/10 | 20 self-test cases including the original defect; its pre-release and heading holes then closed |
| Technical Writer | 8/10 | network claim, measured page count and section order all verified; four follow-up facts corrected |
| Domain Safety Engineer | 8/10 | scope prints on the plan; band matches `calc.rs`; ADR-0001 and ADR-0002 cited |
| Product Manager | 8/10 | republish mechanism, stated absences, corrected headlines |
| Maintainer / Steward | 8/10 | trivial-change rule identical; record names v2.5.2, links the ADRs and lists the passes |
| **Council mean** | **8.0** | every seat at or above the bar |

The Technical Writer's three blockers were fixed before its second pass: the
network paragraph names the installer download, the "old body" claim is narrowed
to v2.5.0 and v2.5.1, and the six v2.1.x files put Compatibility before Download.
The Maintainer's two: CONTRIBUTING uses the charter's exact trivial-change rule,
and this record names the release, links ADR-0001 and ADR-0002, and lists the
passes. Both seats re-scored the fixed tree at 8.

## Decision

**Accepted.** Every participating seat scored 8/10 after the required changes,
and the council mean is 8.0 — the bar in the charter is met. The change keeps one
notes file per tag, a release gate that compares tag, version, notes and record
before either platform build, a one-command republish for the 13 older pages,
and the scope line on the printed plan. Out of scope and recorded as such: code
signing, an in-app changelog, and the version bump — v2.5.2 is cut separately,
after `tauri.conf.json` and `docs/releases/v2.5.2.md` are written.

## Consequences

This commits the project to: one notes file per tag; a release gate that compares
tag, version, notes and record before building; a one-command republish for older
pages; and a printed plan that states its own scope. It would be wrong to keep if
the per-release writing cost proves unpayable at this cadence — the correct
response then is to cut the release cadence or the prose, not the checks.

`packaging/publish-release-notes.sh` is a one-time repair script; retire it once
the 13 pages are republished, or it becomes a permanent script for a finished
job.

The two claims that would need revisiting if the code changes: the four asset
names asserted by `release.yml`, and the band wording in README, which mirrors
`calc.rs`.
