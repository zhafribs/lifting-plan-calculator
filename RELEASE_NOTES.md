# Release Notes

## 2.2.0

### Added

- **Windows installers.** Windows 10/11 is now supported. Each release carries a
  WiX `.msi` and an NSIS `-setup.exe` built from the same Rust/Tauri source on
  GitHub's Windows runners. The saved-report button now uses the platform's own
  opener (`start` on Windows, `xdg-open` on Linux).
- **AppImage delta updates.** A `.AppImage.zsync` sidecar now ships with every
  AppImage, so `appimageupdatetool` can fetch only the changed blocks when
  updating to a newer version instead of the whole ~78 MB file.

### Changed

- **Standardised release asset names:**
  - `Lifting-Plan-Calculator-2.2.0-linux-x86_64.AppImage` (plus `.zsync`)
  - `Lifting-Plan-Calculator-2.2.0-windows-x86_64.msi` / `-windows-x86_64-setup.exe`
  - Assets from releases before 2.2.0 keep their old names; this only affects
    new downloads.

### Artifacts

- `Lifting-Plan-Calculator-2.2.0-linux-x86_64.AppImage` — Ubuntu 22.04+,
  Fedora 36+ and Arch (glibc 2.35 floor)
- `Lifting-Plan-Calculator-2.2.0-windows-x86_64.msi` — Windows 10/11, x64
- `Lifting-Plan-Calculator-2.2.0-windows-x86_64-setup.exe` — NSIS installer
- `SHA256SUMS.txt` — verify any asset with `sha256sum -c SHA256SUMS.txt`

### Housekeeping

- GitHub Actions artifact actions were bumped off the deprecated Node.js 20
  runtime; Release runs are now warning-free and the shared cargo cache makes
  the Linux and Windows builds ~4x faster.