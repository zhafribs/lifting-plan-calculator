# 0065 — Toast announcements, diagnostics and the signer audit

**Status:** accepted
**Date:** 2026-11-26
**Release:** v2.17.18
**Change:** `ui/js/app.js`, `ui/js/dom.js`, `ui/js/screens/report.js`,
`.github/workflows/ci.yml`, `AGENTS.md`, `ui/fonts/`, `CHANGELOG.md`
**Seats:** three seats, each covering charter roles: Safety & QA (QA & Test
Engineer, Accessibility Reviewer), Security & Build (Application Security
Engineer, Build & Platform Engineer), Design & Steward (Technical Writer,
Product Manager, Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

Close the three upkeep limits recorded in 0064 and the signer audit gap:

- Toast messages carry `role="status"` for success/information and `role="alert"`
  for failures, with `aria-atomic="true"`.
- Save as HTML gets its own code icon; the PDF keeps the file icon.
- A screen factory that throws is logged as `screen <key> failed` before the
  error reaches the global handler.
- CI audits `tools/sign-manifest/Cargo.lock` as well as the app lockfile.
- The unused Inter italic asset and its checksum line are removed.

## What the council saw

- **Safety & QA (8.5):** the roles are applied; the failed-screen log rethrows
  the original error and does not replace the global banner; the icon is visual
  only; no face referenced the italic; both audits pass. The polite half of the
  announcement is not proven on every AT, so the changelog says "can announce",
  not "will be heard".
- **Security & Build (9.0):** the audit step is a single `bash -e` block, fails
  on either lockfile, and the signer's 18-crate tree is clean; the font manifest
  matches and no dependency, network, CSP or permission changed.
- **Design & Steward (8.5):** the code glyph matches the icon system and the
  HTML/PDF distinction is clear; the changelog is split into Changed and
  Security; Inter stays because the fixed value boxes depend on its metrics.
- **Release Manager:** notes name 2.17.18 and this record; the report is
  unchanged.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8.5/10 | announcements present; rethrow intact; no report change |
| Security & Build | 9.0/10 | both lockfiles gated; manifest matches; no new surface |
| Design & Steward | 8.5/10 | clear icon distinction; honest, split changelog |

**Mean 8.7/10** — at the bar.

## Decision

**Accepted.** Toasts carry a live-region role, the HTML export is visually
distinct, a failed screen is named in the log, and the signer lockfile is
audited.

## Consequences

- Assistive tech can announce success and failure messages; the exact behaviour
  depends on the AT.
- The release signer's dependency tree is held to the same advisory gate as the
  app.
- The printed report, its PDF and all arithmetic are unchanged.

## Known limits

- `role="status"` announcements are not verified with a screen reader on
  WebKitGTK or WebView2.
- Error toasts still disappear after six seconds; a long failure has no second
  read.
- Both audits share `.cargo/audit.toml`; an ignore added for one tree applies to
  the other.
- `check-vendored.sh` verifies listed files but cannot see an unlisted asset;
  the italic orphan was removed, not the hole.
