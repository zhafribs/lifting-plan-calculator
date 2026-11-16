# 0011 — The update download is verified, and the build tools are pinned

**Status:** accepted
**Date:** 2026-11-16
**Release:** v2.9.4
**Change:** the Windows installer download, the Tauri CLI acquisition in the
release workflow, and the vendored-code check

## Problem

The audit (run-1, findings `release/updater/no-signature-verification`,
`release/build/tauri-cli-unverified`, `repo/vendor/pdfjs-hash-self-referential`)
found that:

- the Windows updater downloaded an installer and executed it with no hash or
  signature check, although the release publishes `SHA256SUMS.txt`;
- the release workflow piped an unverified `cargo-tauri` archive into `tar` and
  executed the binary that builds the shipped artifact;
- the vendored-code check covered pdf.js only, and its manifest lives in the same
  commit it verifies.

## Proposal

- **Verify the installer against the release's manifest.** Before running it, the
  updater fetches `SHA256SUMS.txt` from the asset's own release directory, finds
  the installer by name, and compares the SHA-256 of the downloaded bytes. A
  missing manifest, a missing entry, a failed read or a mismatched digest
  deletes the file and refuses to run it.
- **Pin the build tools.** The Linux and Windows jobs download the pinned
  `cargo-tauri` archive to a file, verify a hard-coded SHA-256, and only then
  extract. The cache key now includes the digest.
- **Cover every vendored tree.** `check-vendored.sh` checks pdf.js, KaTeX and the
  bundled fonts against a manifest in each directory, and fails if any directory
  has none.

## What was reviewed

- `cargo test --all-targets` → 212 passed, including three new tests: a release
  manifest names its installer by digest, a malformed or absent entry is not a
  checksum, and the manifest URL is derived next to the asset.
- `cargo clippy --all-targets -- -D warnings` → clean.
- `packaging/check-vendored.sh` → all three trees OK.
- The pinned digests were computed from the archives the workflow fetches
  (`820ae3c7…` Linux, `3e8687fb…` Windows, Tauri CLI 2.12.4).
- The Windows download path is compile-checked in CI; the network fetch itself
  was not exercised here.

## Honest limit

This verifies **integrity**, not **authorship**. The manifest is served from the
same release as the asset, so a compromised release can replace both — exactly
the limitation recorded in `docs/releases/README.md`. Closing that requires
signing the manifest with a key whose public half is embedded in the app, which
needs a key ceremony the maintainer must perform; it is not claimed here.

## Decision

**Accepted.** A corrupt or substituted download no longer runs, the build tools
are pinned, and every vendored tree is checked.

## Consequences

- An offline or rate-limited machine cannot install the update; the app reports
  that the release checksums could not be read rather than running anything.
- The two pinned digests must be updated deliberately when `TAURI_CLI_VERSION`
  changes; the workflow fails loudly if the archive does not match.
- The vendored manifests still live in the repository, so they detect drift and
  make a replacement visible in review, not tampering by someone who can edit
  both. External pinning needs the signing key above.
