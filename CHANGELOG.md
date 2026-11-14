## [Unreleased]

Nothing yet.

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
