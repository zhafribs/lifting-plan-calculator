# Release Notes

## 2.3.0

### Added

- **The new workbook format, wired into the Crane and Graph tabs.** Importing
  a load chart with the `Crane location on axis (X, Y)` cell and the jib
  sheet now:
  - reads the crane's position and keeps the **longest main boom**;
  - offers a **Jib configuration** card — Boom length / + jib / + extended
    jib — where the configuration picks its jib (7.2 m or 12.8 m) and the
    operator chooses one of the offset angles that jib allows, plus the boom
    angle to read the chart at;
  - **auto-chooses the boom angle that passes the 75% rule** at the farthest
    reach, exactly as the working radius is suggested for the main boom;
  - checks the jib table at that angle, with the working radius derived from
    the assembly geometry (measured from the X axis);
  - mirrors the check into the manual entry: derived radius, jib capacity at
    the angle, and the boom written as `34+7.2` / `34+12.8`.

- **The Graph tab follows the workbook.** The crane position comes from the
  workbook, the boom length and working radius from the Crane tab, and the
  boom angle drawn there is published back (with the radius the jib reaches).
  A manual change on the diagram swaps the card's caption for a
  **Reset to Crane Tab** button; the next Crane tab edit re-syncs.

### Fixed

- A jib offset now tilts the jib **clockwise** (below the boom axis), as the
  charts are drawn.
- Working radii are measured from the **x = 0 axis**, not from the crane's own
  position.
- Choosing a dropdown no longer yanks the page back to the top: a re-solve
  leaves every scroll position exactly where the operator left it.
- The workbook table folds behind a **Workbook table** toggle.

### Artifacts

- `Lifting-Plan-Calculator-2.3.0-linux-x86_64.AppImage` (plus `.zsync`) —
  Ubuntu 22.04+, Fedora 36+ and Arch (glibc 2.35 floor)
- `Lifting-Plan-Calculator-2.3.0-windows-x86_64.msi` — Windows 10/11, x64
- `Lifting-Plan-Calculator-2.3.0-windows-x86_64-setup.exe` — NSIS installer
- `SHA256SUMS.txt` — verify any asset with `sha256sum -c SHA256SUMS.txt`
