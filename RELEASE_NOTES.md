# Release Notes

## 2.4.0

### Added

- **Update reminders.** The app asks GitHub for the newest release at launch,
  and on demand from the About tab. When one exists, a reminder strip appears
  under the topbar with the release notes, the repository, and a dismiss
  button — both links open in the desktop's browser.
- **Windows updates from this repository.** On Windows the reminder and the
  About card also offer **Download & install**: the release's installer is
  fetched from the GitHub release, started, and the app closes so the
  installer can replace the running files. The AppImage is only reminded —
  its release page carries the new file, and `appimageupdatetool` still
  delta-updates it.

### Fixed

- The reminder strip folds without moving the page: the screen keeps the
  flexible grid row, so every tab's layout is untouched whether the strip is
  showing or not.

### Artifacts

- `Lifting-Plan-Calculator-2.4.0-linux-x86_64.AppImage` (plus `.zsync`) —
  Ubuntu 22.04+, Fedora 36+ and Arch (glibc 2.35 floor)
- `Lifting-Plan-Calculator-2.4.0-windows-x86_64.msi` — Windows 10/11, x64
- `Lifting-Plan-Calculator-2.4.0-windows-x86_64-setup.exe` — NSIS installer
- `SHA256SUMS.txt` — verify any asset with `sha256sum -c SHA256SUMS.txt`
