# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.3.0] - 2026-10-11

### Added

- The new workbook format in the Crane and Graph tabs: the crane's position,
  the longest main boom, a jib configuration card (the configuration picks
  its jib; only the offset angles it allows are offered), the jib capacity
  checked at the boom angle, the boom angle auto-chosen to pass the 75% rule
  at the farthest reach, the manual entry mirroring the jib check, and the
  Graph tab following the workbook with a "Reset to Crane Tab" button after
  manual edits.

### Fixed

- Jib offset angles rotate clockwise (below the boom axis).
- Working radii are measured from the x = 0 axis, not from the crane.
- Choosing a dropdown no longer scrolls the page back to its top.
- The workbook table folds behind a toggle.

## [2.2.1] - 2026-10-10

### Fixed

- Opening the Graph tab no longer shows "ResizeObserver loop completed with
  undelivered notifications." as an error banner: the diagram's ResizeObserver
  defers its redraw to the next animation frame, and the global error handler
  treats that specific recoverable message as the warning it is.

## [2.2.0] - 2026-10-09

### Added

- **Windows installers.** The release workflow now builds a Windows MSI and an
  NSIS `setup.exe` from the same Tauri source and attaches them to each release.
  The saved-report opener now uses `start` on Windows instead of `xdg-open`.
- **AppImage delta updates.** Every AppImage release ships a matching
  `.AppImage.zsync` sidecar (built with `zsyncmake`), so `appimageupdatetool`
  can download only the blocks that changed between versions.

### Changed

- **Standardised release asset names.** Linux is now
  `Lifting-Plan-Calculator-<version>-linux-x86_64.AppImage` (plus `.zsync`) and
  Windows is `Lifting-Plan-Calculator-<version>-windows-x86_64.msi` (plus
  `-setup.exe`). Assets from earlier releases keep their old names.

## [2.1.5] - 2026-10-09

### Fixed

- **Double scroll now engages the moment a tab can show two columns**, at the
  same 981px cut the layouts use (previously 1001px). Between 981 and 1000px
  the Overall Weight, Crane, Graph, Uniform Load, Nonuniform Load and Tandem
  tabs showed their cards side by side with a single page scroll; now every
  tab splits into independent scroll panes whenever its cards are side by
  side (981px+), and falls back to one stacked scroll below that. Summary
  gets the same treatment.

## [2.1.4] - 2026-10-09

### Changed

- **One consistent layout on every tab**: all wide windows now use the same
  60/40 two-column split (Overall Weight, Crane, Graph, Uniform Load,
  Nonuniform Load, Tandem and Summary), and every tab's content fills the same
  width instead of each tab shrinking to fit its own content. On the Summary
  tab the split is mirrored (40/60) so the report preview gets the wider
  column. Stacked (narrow) windows are unchanged.

## [2.1.3] - 2026-10-09

### Added

- **Independent scroll panes on the Crane and Summary tabs**: on wide windows
  the Crane form column and its capacity-check column scroll on their own, and
  on the Summary tab the sections/export column scrolls on its own while the
  report preview keeps its own scroll. Stacked (narrow) windows keep the single
  scroll they had before.

## [2.1.2] - 2026-10-09

### Added

- **Independent scroll panes on the Overall Weight tab**: on wide windows the
  Load weight and Lifting tackle cards and the Weight summary and Next step
  cards each scroll on their own, so the summary keeps its place while the form
  moves. Stacked (narrow) windows keep the single scroll they had before.

## [2.1.1] - 2026-10-09

### Fixed

- **Rail footer in short, wide windows**: the sidebar footer (app name,
  version, contact) is no longer pushed below the window when the labelled
  navigation is taller than the window. The navigation list scrolls on its own
  and the footer stays pinned, at every window size.

## [2.1.0] - 2026-10-09

A usability release that keeps the important things in view at any window
size, with no change to the calculations.

### Added

- **Independent scroll panes on wide windows**: Uniform Load, Nonuniform Load
  and Tandem each split into a form column and a results column that scroll on
  their own, so one side keeps its place while the other moves. Stacked
  (narrow) windows keep the single scroll they had before.
- **Always-visible working range diagram**: on the Graph tab the diagram fills
  its column and rescales, so it stays in view while the boom configuration,
  boom angle and envelope, and readout scroll beside it.

### Fixed

- **Rail footer**: the app name, version and contact are no longer hidden when
  the window is at its smallest size. The footer stays pinned beneath the icon
  rail and the navigation list scrolls on its own, so the product identity is
  visible at every window size.

## [2.0.0] - 2026-10-09

The first public release: a Linux desktop build of the Lifting Plan Calculator,
written in Rust with a Tauri 2 shell and delivered as a portable AppImage.

### Added

- **Overall Weight** form: load lines and tackle, summed to the gross load at
  the hook.
- **Crane** form: capacity check against a load chart imported from Excel, or a
  single point typed by hand under the 75% rule.
- **Uniform Load** form: the nineteen 1-leg / 2-leg / nested / bridle
  arrangements, each checked against the gross load.
- **Nonuniform Load** form: the 2-leg asymmetric bridle with a shortening grab
  hook.
- **Tandem** form: two cranes sharing one load, in both declared lug
  configurations.
- **Summary** tab: an HTML report with KaTeX-typeset formulas that can be saved
  as a portable file or printed.
- Rust calculation engine ported from the Android Kotlin source, with 134 tests
  ported from the Kotlin unit suite.
- Two AppImage build paths: a fast host build and a portable Ubuntu 22.04
  userspace build (glibc 2.35 floor) that runs on Ubuntu 22.04+, Fedora 36+ and
  Arch.

### Fixed

- **Summary tab scrolling**: on narrow (stacked) windows the cards pinned over
  the report and the page had two scrollbars. The cards now leave the sticky
  layer below the breakpoint and the report is sized to its content, so the
  page owns a single scrollbar and the whole plan scrolls as one. The wide
  side-by-side layout keeps its independent report scroll.
