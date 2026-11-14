## [Unreleased]

Nothing yet.

## [2.8.2] - 2026-11-14

### Changed

- Dependency refresh: tauri 2.12.5, lopdf 0.47, calamine 0.38, rfd 0.18,
  serde_json 1.0.154, zip 9.5.1, tauri-build 2.7.4, and
  `actions/download-artifact` 8.0.2 in the release workflow.
- No behaviour, calculation or report change. The PDF writer, the workbook
  reader and the save dialog were all exercised by the release's test suite and
  the export probe before this went out.

## [2.8.1] - 2026-11-14

### Fixed

- Saving a PDF can no longer delete the plan already at the chosen name. The
  export deleted the target before moving the new file in, on the belief that
  Windows cannot replace on rename; it can, so a failed save used to leave
  neither file. The move is one replacing rename, retried while a just-written
  file is still being closed.
- The footer is written into a file beside the print rather than over it, so a
  PDF engine still holding its output cannot block the save.
- Dependabot no longer moves `webview2-com` or `windows-core` on their own: a
  version apart makes the WebView2 COM interfaces stop type-checking, which broke
  the Windows build in the 2.8.0 dependency update.

## [2.8.0] - 2026-11-14

### Changed

- On Windows, *Save as PDF* prints with the WebView2 runtime the app already
  needs in order to start, so the export works when Microsoft Edge has been
  removed or disabled. Edge remains the fallback if the runtime is too old to
  print or the print fails.
- The Summary page view redraws itself about a second after typing stops, so it
  keeps up while the plan is edited instead of waiting for a field to be left.
  The Live view is unchanged and still instant.

## [2.7.0] - 2026-11-14

### Added

- Summary's report preview opens on **Pages**: the report as the A4 pages that
  will be saved, with zoom in and out, fit-to-width and a `Page N of M` counter.
  Page breaks are now visible before printing, and *Save as PDF* writes exactly
  the bytes the page view showed when the plan has not changed since.
- The **Live** view keeps the fast HTML preview for typing; the page view is
  rendered on demand — when it opens, when a section is toggled, or when a field
  is left — never on every keystroke.

### Changed

- The footer's embedded font is read directly, so the export no longer depends
  on the unmaintained `ttf-parser` crate. The dependency audit in CI now reports
  advisories instead of gating: every one it finds is inherited from the
  Tauri/GTK stack and has no fixed upgrade.

## [2.6.0] - 2026-11-14

### Changed

- *Save as PDF* prints the report with the app's own WebKitGTK view instead of
  a bundled browser. The AppImage no longer carries a headless Chromium: it drops
  from about 182 MB to roughly the 2.5.3 size, and the export still needs nothing
  installed on the machine, downloads nothing, and works offline.
- The printed footer is written onto the finished pages by the app on Linux, so
  it no longer depends on the renderer's version. Chromium builds older than 131
  drop CSS `@page` margin boxes silently, which used to take the plan reference
  and the page counter off a printed plan. Windows keeps printing with Edge and
  keeps its own margin boxes.
- The PDF is printed into a hidden file beside the chosen one and only replaces
  it once the footer is on every page, so a failed export no longer leaves a
  plan without its reference and page numbers.
- The default credit line's envelope is now an em dash, so the field in Summary
  and the printed footer read the same.
- The footer's typeface is embedded in the export (Liberation Serif, the face
  the Chromium build used), so a plan reference or credit line in Latin with
  diacritics, Greek, Cyrillic or Vietnamese prints as it was typed instead of
  falling back to a placeholder.

### Removed

- The bundled Chrome for Testing payload (`src-tauri/pdf-renderer/**`), the
  build-time fetch (`scripts/fetch-pdf-renderer.sh`), the `BUNDLE_PDF_RENDERER`
  build switch and the release workflow's renderer cache step. There is one
  AppImage variant, and it carries no browser.

## [2.5.4] - 2026-11-14
