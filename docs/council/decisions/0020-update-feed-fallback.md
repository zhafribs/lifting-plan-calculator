# 0020 — Update check falls back to the release feed

**Status:** accepted
**Date:** 2026-11-18
**Release:** v2.13.3
**Change:** `src-tauri/src/update.rs`
**Seats:** Release Manager, Application Security Engineer, QA & Test Engineer,
Product Manager, Maintainer / Steward. The Release Manager chaired; the
implementer did not score.

## Proposal

The update check used the GitHub API without a token. A user on 2.13.1 received
`status code 403` from
`https://api.github.com/repos/zhafribs/lifting-plan-calculator/releases/latest`,
which is the unauthenticated rate-limit response. The update button then had no
latest release to work from.

The API remains the first choice. When it fails, the app reads the repository's
public releases Atom feed (`releases.atom`), takes the first stable `vX.Y.Z`
tag, and builds the same asset names the release workflow publishes. The
download still verifies the signed `SHA256SUMS.txt` and the asset digest before
it is used.

## What the council saw

- **Application Security Engineer:** the fallback uses the same HTTPS GitHub host
  and the same fixed repository path. It does not accept a tag, file name or URL
  from the response beyond the stable tag token; asset names are constructed
  locally. The feed does not bypass the signed checksum or digest check, and the
  existing update URL and downgrade guards still run on download.
- **QA & Test Engineer:** `cargo test` → **246 passed**, including tests that the
  Atom parser skips a prerelease and that the fallback builds the expected
  release asset URLs. The live feed currently lists v2.13.2, v2.13.1 and older
  stable tags, and the parser selects the first stable tag.
- **Product Manager:** a 403 from a rate-limited API should not look like a
  broken app. The fallback keeps the update button useful without adding a
  GitHub token or a new service.
- **Maintainer / Steward:** no dependency or configuration was added; the
  fallback is a small parser over GitHub's public feed.
- **Release Manager:** the app version does not change the artifact format or
  the signed release process. The release notes tell users on older builds to
  install 2.13.3 once, because 2.13.1 does not carry this fallback.

## Decision

**Accepted.** The update check falls back to the releases Atom feed when the
GitHub API is rate-limited.

## Consequences

- The API rate limit no longer disables the update button.
- If both the API and the feed are unreachable, the error names both failures
  and suggests the release page.
- The download path is unchanged: signed manifest first, then digest, then the
  AppImage replacement.
