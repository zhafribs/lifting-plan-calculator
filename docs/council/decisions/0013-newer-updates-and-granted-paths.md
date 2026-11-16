# 0013 — An update must be newer, and nothing is read from a path the app did not grant

**Status:** accepted
**Date:** 2026-11-16
**Release:** v2.11.0
**Change:** downgrade refusal, URL parsing, workbook and report path grants, and
build-time digest verification

## Problem

Three audit records were still open after 2.10.0:

- the signed manifest carried no version, so an older correctly signed release
  could be replayed as a downgrade (`install()` never compared versions);
- `install_update` joined a caller-supplied name onto a directory and accepted
  any URL with the right prefix; `import_chart` read any path; `save_report`
  passed a caller string into the dialog as a path;
- `SHA256SUMS.txt` was generated from the same downloaded bytes the release
  published, so it could not detect a swapped artifact.

## Proposal

- **Require a newer release.** The tag is parsed from the asset URL
  (`/releases/download/<tag>/…`) and compared with `CARGO_PKG_VERSION`; an equal
  or older tag is refused. This closes replay without changing the signed payload,
  so 2.10.0 installations keep updating normally.
- **Parse the URL.** `url::Url` checks scheme `https`, host `github.com` and the
  repository's release path. The installer name must survive `Path::file_name`,
  so a name containing separators is refused rather than written.
- **Grant the path.** `ChartHolder` keeps one path — set by the file picker or by
  the `--chart=` launch hook — and `import_chart` accepts only that path, once.
  A failed attempt does not spend the grant.
- **Keep the dialog a dialog.** The suggested report name is a leaf name, and the
  written file is forced to `.html`.
- **Verify at the sink.** Each build job records `sha256sum` of its own artifacts;
  the publish job downloads those records and fails if any byte differs before it
  signs and publishes.

## What was reviewed

- `cargo test --all-targets` → 220 passed. New tests: only this project's release
  URLs are accepted (other host, other repo, plain http, `..`); the tag is parsed
  from the URL; a suggested report name cannot choose a folder; a workbook can
  only be imported from the granted path; a mismatch on the wrong path does not
  spend the grant.
- `cargo clippy --all-targets -- -D warnings` → clean.
- The workflow gained `digests-linux` and `digests-windows` artifacts and a
  publish step that compares them; `yaml` parses and the step order is:
  download artifacts → four-asset check → download digests → compare → generate
  checksums → sign → upload.

## Decision

**Accepted.** An older release is refused, the app reads only what it was
granted, and the published checksum list is verified against what the build jobs
produced.

## Consequences

- Re-importing the same workbook requires picking it again; the launch hook is
  unaffected.
- A save dialog where the operator types a non-HTML extension writes `.html`.
- A publish run whose artifacts were altered between the build and the publish job
  now fails instead of publishing a manifest that matches the alteration.
- Still open, unchanged: Windows Authenticode (needs a certificate or SignPath's
  free OSS programme), CSP and a sandboxed preview frame (needs a UI pass), the
  empty zsync signature sections, the unverified portable rootfs, and the GitHub
  settings around tag protection.
