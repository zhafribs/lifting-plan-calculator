# 0017 — Every common Linux package, and a per-machine Windows install

**Status:** accepted
**Date:** 2026-11-18
**Release:** v2.14.0
**Change:** `build-appimage.sh`, `packaging/arch/`, `packaging/flatpak/`,
`packaging/make-arch-package.sh`, `packaging/make-flatpak.sh`,
`.github/workflows/release.yml`, `src-tauri/tauri.conf.json`,
`src-tauri/src/update.rs`, `README.md`
**Seats:** Build & Platform Engineer, Application Security Engineer, QA & Test
Engineer, Product Manager. The Release Manager chaired; the implementer did not
score.

## Proposal

One release now carries every format a user is likely to want:

- **Windows:** the MSI and the NSIS setup install **per-machine** into
  `C:\Program Files`. The setup defaulted to per-user, which put the app in one
  profile; the MSI was already per-machine, so the two disagreed.
- **Linux:** in addition to the AppImage and its `.zsync`, the release ships
  `.deb`, `.rpm`, `x86_64.pkg.tar.zst` and a single-file `.flatpak`. All four
  are built from the same release binary and depend on the distribution's own
  GTK 3 and WebKitGTK 4.1 (the Flatpak uses the GNOME 48 runtime), so a package
  manager updates the libraries with the system.

The README's per-release "What's new" block is removed: the Releases page
already carries each tag's notes, and the file per tag is the single source.

## What the council saw

- `.deb` and `.rpm` are produced by Tauri's own bundlers, locally verified as
  real archives: the deb's `data.tar.gz` carries `/usr/bin/lifting-plan-calculator`
  and the hicolor icons.
- The Arch package is built by `makepkg` from the release binary; the local run
  produced `lifting-plan-calculator 2.13.5-1` (4.5 MB) with `.PKGINFO`,
  `.BUILDINFO` and `.MTREE`.
- The Flatpak is built by `flatpak-builder` against `org.gnome.Platform//48`,
  which provides `libwebkit2gtk-4.1.so.0` and GTK 3; the bundle installs and
  reports its version. It builds without warnings after the icon, desktop entry
  and metainfo were named after the app id.
- The release job now requires all eight assets before publishing, and the notes
  checker accepts the new file extensions.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | each format is built from one binary; the Arch and Flatpak steps are scripts that run the same way locally and in CI |
| Application Security Engineer | 8/10 | packages link the system libraries rather than bundling copies, so security updates come from the distribution; the Flatpak keeps its sandbox network/webkit permissions to what the app uses |
| QA & Test Engineer | 8/10 | deb/rpm/Arch/Flatpak were built and inspected locally; the workflow gates on all eight assets |
| Product Manager | 8/10 | users no longer have to unpack an AppImage to install the app the way their distribution expects |

**Mean 8.0/10** — at the bar.

## Known limits

- The Flatpak was installed locally but could not be *run* headlessly: WebKit
  killed itself under Xvfb without DRI3 on this Wayland host. It needs one
  manual launch on a real desktop.
- The Arch package is built inside an `archlinux:base-devel` container in CI; it
  is not signed with a packager key.
- The distributions' own repositories are not updated; these packages are
  release assets, installed by path.
