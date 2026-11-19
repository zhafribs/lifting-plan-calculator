# 0020 — The Arch package updates itself, with a way out

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.16.1
**Change:** `src-tauri/src/update.rs`, `src-tauri/src/lib.rs`, `ui/js/api.js`,
`ui/js/screens/about.js`, `docs/releases/v2.16.1.md`
**Seats:** Build & Platform Engineer, Application Security Engineer, QA & Test
Engineer, Product Manager. The Release Manager chaired; the implementer did not
score.

## Proposal

The AppImage and the Windows installer could update from About; an Arch install
could only follow a link. The operator asked for the same button on the package.

- The app decides how it was installed once, from the facts it can see: an
  `APPIMAGE` or `FLATPAK_ID` marker means not a pacman install; otherwise
  `pacman -Q lifting-plan-calculator` must succeed.
- A pacman copy is offered the release's `pkg.tar.zst` in the update check, the
  same way an AppImage copy is offered its own file.
- Installing reuses the existing verified download: the file's SHA-256 must
  match the digest recorded in the release's **signed** `SHA256SUMS.txt`, the
  URL must belong to this repository's releases, and the name must be a bare
  file name.
- The verified file is then handed to `pkexec pacman -U --noconfirm`. No shell
  is involved and no string from the network is passed to a shell.
- When `pkexec` is missing or no polkit agent answers, nothing is installed. The
  app shows `sudo pacman -U '<file>'` for the file already on disk, with a copy
  button. The operator stays in control of the privilege decision.

## What the council saw

- The policy is the same one the other two platforms already use: HTTPS to this
  project's release downloads, a signed checksum, then the platform's own
  installer. `pkexec` is a fixed binary with fixed arguments and our verified
  path as its only variable.
- The detection is conservative: a Flatpak would otherwise also find `pacman` on
  an Arch host, so `FLATPAK_ID` and `APPIMAGE` are checked first.
- A unit test pins the refusal: an AppImage asset handed to the package
  installer is rejected before any download.
- The Rust suite is 250 tests, `clippy -D warnings` clean; the frontend suites
  and the vendored-asset checks are unchanged.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | one detection point, one verified-download path shared with the AppImage, no new dependency |
| Application Security Engineer | 8/10 | signed checksum before install; fixed argv to `pkexec`/`pacman`; no shell; the manual fallback changes nothing by itself |
| QA & Test Engineer | 8/10 | refusal tested; the download, digest and package build are exercised on every release; the polkit path needs a desktop to see the prompt |
| Product Manager | 8/10 | the Arch package stops being the only install that cannot update itself, and a minimal desktop is not left stuck |

**Mean 8.0/10** — at the bar.

## Known limits

- The polkit path was not exercised on a real desktop in this session: no
  passwordless polkit agent was available to the build machine. The no-agent
  path is the one shown here.
- The manual command uses the verified file in
  `$TMPDIR/lifting-plan-calculator-update/`; a reboot or a panicked
  `systemd-tmpfiles` run can remove it, in which case the operator downloads the
  package again.
- Deb and rpm copies still update through their own package managers; the same
  design would extend to them, but was not asked for.
