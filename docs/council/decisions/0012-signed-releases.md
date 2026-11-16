# 0012 — Releases are signed, and the app will not run an update it cannot attribute

**Status:** accepted
**Date:** 2026-11-16
**Release:** v2.10.0
**Change:** Ed25519 signing of the checksum manifest, verification in the update
path, the signer tool, and the release job that signs

## Problem

2.9.4 made the updater verify the installer's SHA-256 against the release's
`SHA256SUMS.txt`, which closed corruption and asset mismatch. It did not close
authorship: the manifest and the installer come from the same place, so anyone
who can publish to the repository can replace both and every check passes.

## Proposal

- **Sign the manifest.** The release publishes `SHA256SUMS.txt.sig`, an Ed25519
  signature over the manifest's bytes. The public key is
  `src-tauri/update-public-key.txt`, compiled into the app; the private key is the
  `UPDATE_SIGNING_KEY` repository secret and a 0600 file on the maintainer's
  machine, never in the repository.
- **Verify before trusting anything.** `install()` fetches the manifest and its
  signature as bytes, verifies the signature with the embedded key, then looks up
  the installer's digest and compares it. An unsigned, wrongly signed, altered or
  unlisted manifest deletes the download and refuses.
- **One small signer.** `tools/sign-manifest` is a standalone crate (only
  `ed25519-compact`) so the publish job, which has no GTK toolchain, can build it
  in seconds. It refuses to overwrite an existing key, refuses a secret that does
  not match the committed public key, and can verify what it produced.

## What was reviewed

- `cargo test --all-targets` → 216 passed; the signer's own suite → 2 passed.
  New tests: a committed signature fixture verifies under the committed key;
  wrong key, changed manifest, truncated and non-ASCII signatures are refused;
  the update path refuses an unsigned, altered or unlisted manifest.
- `cargo clippy --all-targets -- -D warnings` → clean.
- The ceremony was run: private key at
  `~/.config/lifting-plan-calculator/update-signing.key` (0600), public key
  `c2393429431fc55446c93650d725f89a8af16bc29bdb20d5860a4477b53370ab` committed,
  `UPDATE_SIGNING_KEY` set. Signing with the real key and verifying under the
  committed key both succeed; a random key is refused.
- Two review rounds. The first pass scored 7 and 5 and found a panic on non-ASCII
  signature input, a `generate` that could truncate the only key, a publish job
  that could not build the signer, and no check that the secret matches the
  committed key. All are fixed and re-verified; the second pass scored 8.5 and 9.

## Scores

| Seat | Score | Position |
| --- | --- | --- |
| Application Security Engineer | 8.5/10 | Panic and key-truncation fixed, verification fails closed, fixture proves the key pair matches |
| Release engineering (build) | 9/10 | Standalone signer, key liveness checked, early failure, `target/` untracked, secret confined to the sign step |

## Decision

**Accepted.** A release is signed by the project's key, and an update that is not
is refused.

## Known limits, stated rather than hidden

- **The manifest alone is signed.** It carries no tag or version, so an older
  signed manifest could be replayed as a genuine downgrade. The installer names
  are versioned, which makes that deliberate rather than accidental; binding the
  tag into the signed payload is the next step if downgrade resistance matters.
- **No key means no enforcement.** A build whose key file is empty accepts an
  unsigned manifest (the bootstrap state). Once a key is committed, every later
  build enforces it, so the keyless state cannot be reintroduced silently by a
  normal release.
- **Losing the key ends signing.** Recovery is a new build with a new key that
  users install manually once; the release notes say so and the ceremony script
  refuses to overwrite an existing key.
- Windows installers still carry no Authenticode certificate; the signature here
  is the project's own, not the operating system's.
