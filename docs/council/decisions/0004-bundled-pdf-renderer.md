# 0004 — The AppImage carries its own PDF renderer

**Status:** accepted
**Date:** 2026-11-14
**Release:** v2.5.4
**Change:** `src-tauri/src/commands.rs`, `src-tauri/tauri.conf.json`,
`scripts/fetch-pdf-renderer.sh`, `build-appimage.sh`,
`build-appimage-portable.sh`, `.github/workflows/release.yml`,
`THIRD_PARTY_NOTICES.md`
**Seats:** Build & Platform Engineer, Application Security Engineer, QA & Test
Engineer, Product Manager. The Release Manager chaired; the implementer did not
score.

## Proposal

*Save as PDF* searched the system for Chromium. Two things followed from that:

1. On a machine with no browser installed — the normal case for a site laptop —
   the feature was dead. The app said so, but a feature the AppImage ships must
   work from the AppImage alone.
2. Inside the AppImage the search *found* Chromium and then failed to start it:
   the child inherited the AppImage's `LD_LIBRARY_PATH`, loaded the bundled
   Ubuntu 22.04 libraries and died with
   `undefined symbol: BrotliDecoderAttachDictionary`. The app reported "no PDF
   renderer was found", which was false and sent the operator looking for a
   browser that was already installed.

The change does three things:

- the build fetches Chrome for Testing's headless shell (pinned version and
  archive checksum, `scripts/fetch-pdf-renderer.sh`, ~128 MB, not in git) and
  the AppImage bundles it;
- the app prefers the bundled renderer and falls back to a system Chromium,
  Edge on Windows;
- every external program the app starts — the renderer and `xdg-open` — is
  handed an environment with the AppImage's own paths removed.

## What the council saw

- Reproduced on the released 2.5.3 AppImage: with its `LD_LIBRARY_PATH`,
  `chromium --version` exits 127 with the symbol error; with the variable
  removed it prints `Chromium 157.0.8103.38`.
- The bundled headless shell prints the report to A4 (594.96 × 841.92 pt) with
  the one-inch content margin, six pages, no missing shared libraries.
- `cargo test --all-targets` → **187 passed** (186 before, plus the
  environment-stripping test); `cargo clippy --all-targets -- -D warnings`
  clean.
- The resource glob has a committed placeholder, so `cargo test` and CI do not
  download 128 MB to compile; only the build scripts and the release job fetch
  it.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | pinned version and checksum, 12 s fetch, cached in CI under the pin; the size cost is stated rather than hidden |
| Application Security Engineer | 8/10 | the bundled browser renders a local `file://` page only, in a throwaway profile; the checksum pins the artifact; the license text travels in the payload |
| QA & Test Engineer | 8/10 | the failure was reproduced before the fix, the environment stripping is unit-tested, and the system-browser fallback is unchanged |
| Product Manager | 8/10 | "works on a machine with nothing installed" is the promise the single-file download makes; the download grows by the renderer |

**Mean 8.0/10** — at the bar.

## Required changes before merge

1. **BP-1 — say the cost.** The README and the release notes state the larger
   download and why; no size claim is left stale.
2. **AS-1 — carry the license.** `LICENSE.headless_shell` ships beside the
   binary and `THIRD_PARTY_NOTICES.md` names the component and its license.
3. **QA-1 — keep the gate cheap.** The placeholder keeps `cargo test` free of
   the download; the release job caches it under the pinned version.
4. **QA-2 — prove the fix, not the theory.** The environment stripping is
   tested, and the system-browser path is still exercised when no bundle is
   present.

## Known limits

- The AppImage grows by roughly the renderer's compressed size (~100 MB).
- The renderer is pinned and must be bumped deliberately; the cache key carries
  the version, so a bump refetches.
- Windows keeps using the installed Edge; no Chromium is bundled there.
