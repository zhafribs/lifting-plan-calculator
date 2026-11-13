# Release Notes

## 2.5.0

### Added

- **Booked crane rating is checked.** The crane's own rating is now compared
  with the capacity the lift needs at the 75% rule, and the result governs when
  it fails. A crane too small for the lift can no longer pass because the chart
  reads high; the plan shows both figures, and "Not checked" when no rating has
  been entered.
- **The plan carries its own details.** The Summary tab has a *Plan details*
  card: plan reference, Prepared by, Checked by and Approved by. They print at
  the top of the plan together with the issue date, blank lines included, so a
  printed copy can be signed.

### Changed

- **A radius outside the load chart is refused.** The chart lookup used to
  carry the nearest charted capacity in from either end; now a working radius
  (or a jib boom angle) outside the charted range is refused with the range the
  chart rates. The plan states the refusal in its crane section instead of
  leaving it out.
- The Load example charts the radius it reads, so its figures are a rated lift
  rather than a clamped one.

### Fixed

- A chart table lists only the radii the selected boom column charts, instead
  of repeating the last charted capacity down every remaining workbook radius.
- A boom switch to a column with no charted capacities no longer publishes the
  previous boom's capacity under the new boom's label.
- A chart imported for a lift no charted radius carries starts at the shortest
  charted radius and reports the overload, instead of leaving the tab blank.
- A workbook that names its own unit ("Unit: ton") is read from that text; the
  magnitude guess is kept only for charts that declare nothing. Small charts
  written in kg were being read as tonnes.
- Two example load charts ship in `docs/examples/` (35 t TR-350M-1 and 20 t
  TL-200M), each parsed end to end by the test suite.

### Artifacts

- `Lifting-Plan-Calculator-2.5.0-linux-x86_64.AppImage` (plus `.zsync`) —
  Ubuntu 22.04+, Fedora 36+ and Arch (glibc 2.35 floor)
- `Lifting-Plan-Calculator-2.5.0-windows-x86_64.msi` — Windows 10/11, x64
- `Lifting-Plan-Calculator-2.5.0-windows-x86_64-setup.exe` — NSIS installer
- `SHA256SUMS.txt` — verify any asset with `sha256sum -c SHA256SUMS.txt`

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
