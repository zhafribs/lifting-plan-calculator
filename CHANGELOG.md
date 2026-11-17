## [Unreleased]

Nothing yet.

## [2.13.1] - 2026-11-17

### Fixed

- **Outrigger Pad Size has a shortcut.** The tab is `Ctrl+3`; Graph, Uniform
  Load, Nonuniform Load, Tandem and Summary move to `Ctrl+4` through `Ctrl+8`.
- **The outrigger form reads clearly.** The bearing-value row is separated from
  the ground selectors, the `V` label names its unit, and the value field no
  longer stretches across the card.
- **The two load fields follow the rest of the sheet.** `Crane mass (CM)`
  defaults to the Crane tab's rated capacity, and `Lift load (L)` defaults to
  the Overall Weight gross load. Editing either field breaks the link;
  **Use crane tab** and **Use overall weight** reconnect it.

## [2.13.0] - 2026-11-17

### Added

- **Outrigger Pad Size.** A new tab sizes the bearing area under one outrigger
  from the BS 8004 Table 1 presumed allowable bearing values adapted for
  Malaysian practice. It carries the ground category, description, bearing
  value, crane mass, lift load, square or rectangular pad, utilisation check and
  a printable report section. The same ground table ships as
  `docs/examples/Presumed Allowable Bearing Values (BS 8004 Table 1).xlsx`.

### Changed

- The Excel workbook table starts collapsed after an import, so the chart
  controls stay in view. Click the table toggle to read the rows.

### Fixed

- The Linux AppImage update button now downloads the new AppImage, verifies its
  signed checksum, replaces the running file and restarts the app. The old file
  is removed after the new one starts, and restored if the new one does not.

## [2.12.1] - 2026-11-17

### Fixed

- On Windows the report view no longer flashes a white print window on every
  refresh: the plan is printed by headless Edge, which needs no window at all.
  The app's own WebView2 printer remains as the fallback, and it now stays
  hidden until the report has loaded.

## [2.12.0] - 2026-11-17

### Security

- **The window runs under a Content-Security-Policy.** Scripts load only from
  the app's own origin, so an inline script or an inline event handler no longer
  runs; nothing may be fetched from another origin; the IPC endpoints are the
  only connection target; plugins and forms are off. The report's KaTeX pass
  moved from an inline script to `vendor/katex/lpc-autorender.js`, and the
  preview reports its result by message instead of reading across frames. A
  sandboxed preview frame was tried and rejected: an opaque origin makes
  `script-src 'self'` match nothing, so the equations would not typeset.

### Fixed

- **The jib configuration returns to the main boom properly.** Choosing "Boom
  length" again restores the import's radius basis — the farthest charted radius
  within the 75% rule, its row mirrored, the jib's table and angle cleared —
  instead of keeping the jib's reach and a zeroed angle.
- **A crane without an extended jib no longer offers one.** "Boom length +
  extended jib" is disabled and greyed when the workbook carries a single jib
  length, on both the Crane and Graph tabs, and the store refuses it as well.
- **The Graph tab's boom angle reaches the Crane tab.** It no longer needs a
  workbook or a live radius: with a workbook an angle edit also publishes the
  reach it produces, so the chart check and the printed diagram agree with the
  printed angle. Dragging the crane itself leaves the working radius alone.
- **A jib boom angle between chart rows is read at the lower row.** Capacity
  rises with the boom angle, so a value between two rows must not borrow the
  higher one; the Crane tab's angle select shows the lower row and the check
  uses that row's capacity. Below the first row the angle is refused as outside
  the chart.

## [2.11.0] - 2026-11-16

### Security

- An update must be newer than the running build. The release tag is read from
  the asset URL and compared with the app's version, so a replay of an older
  signed release is refused as a downgrade.
- Release URLs are parsed, not prefix-matched: scheme, host and repository path
  are checked, so a URL that merely starts with the release prefix no longer
  passes.
- The installer's file name must be a plain name, not a path; one that resolves
  outside the update folder is refused instead of written.
- A workbook can only be imported from the path the file picker (or the
  `--chart=` hook) just returned, and only once. The webview can no longer name
  an arbitrary file.
- The report save dialog is pre-filled with a leaf name only, and the file is
  written with an `.html` extension.
- The preview's error message is escaped before it is written into the frame, so
  an error carrying markup can no longer become markup.
- The release job records each artifact's SHA-256 where it was built and the
  publish job refuses a download that does not match, so `SHA256SUMS.txt` is no
  longer a restatement of whatever the artifact store held.
- The portable build pins the Ubuntu base rootfs by SHA-256 and fetches it over
  https only (redirects included), so the compiler and the bundled libraries it
  produces a release with are the ones that were reviewed.

## [2.10.0] - 2026-11-16

### Added

- Releases are signed. `SHA256SUMS.txt` is signed with an Ed25519 key into
  `SHA256SUMS.txt.sig`; the public half is compiled into the app
  (`src-tauri/update-public-key.txt`), and the Windows updater verifies the
  signature before it trusts any digest. An unsigned, wrongly signed or
  mismatched update is deleted and never run.
- `scripts/update-signing-key.sh` creates the key and prints the `gh secret set`
  command; the private key stays on the maintainer's machine and in the
  `UPDATE_SIGNING_KEY` repository secret.

### Changed

- The release workflow signs the checksum manifest in the publish job and fails
  if the tree carries a key but the secret is missing, so a release cannot go out
  unsigned by accident.

## [2.9.4] - 2026-11-16

### Fixed

- The Windows updater now verifies the installer's SHA-256 against the checksum
  manifest published with that release before it runs it. A download that is not
  listed, does not match, or cannot be read is deleted and never executed.
- The release build pins the Tauri CLI archive by SHA-256 for both platforms and
  verifies it before extraction; the cache key includes the digest, so a restored
  cache is the verified archive.
- The vendored-code check now covers KaTeX and the bundled fonts, not only
  pdf.js.

### Security

- This verifies integrity, not authorship: the checksum manifest is served from
  the same release. Signing the manifest remains the next step and is not claimed
  here.

## [2.9.3] - 2026-11-16

### Fixed

- A workbook that declares a huge used area (cells at `A1` and `XFD1048576`, for
  example) made the spreadsheet parser allocate the whole rectangle — hundreds
  of gigabytes — before any limit applied, aborting the app as soon as the file
  was opened. Sheets are now read cell by cell under a row ceiling, a column
  ceiling and a workbook-wide cell budget, and an over-large sheet is refused
  with a message instead of being materialised.
- The 24 MB workbook limit is now applied before the file is read, not after it
  has been loaded into memory.
- On Windows, opening a report or a link went through `cmd /C start`, where an
  `&` in a file name or URL started a second command. The opener now uses
  `ShellExecuteW`, so the target is data and no shell parses it.

## [2.9.2] - 2026-11-16

### Fixed

- The release workflow's PDF job no longer logs the accessibility-bus warning.
- The artifacts are downloaded with `gh` instead of a Node action, so publishing
  no longer prints the `Buffer()` deprecation. Nothing in the app changed.

## [2.9.1] - 2026-11-16

### Changed

- Dependency refresh: rustls 0.23.46, rustls-webpki 0.103.17, webpki-roots
  1.0.12, ctutils 0.4.8, rustix 1.1.6, tokio 1.54.1, and the rest of the
  lockfile. `cargo audit` reports no vulnerabilities.
- The dependency audit now gates the build (`--deny warnings`). The two
  advisories this project cannot act on are listed, with their reasons, in
  `.cargo/audit.toml`.

### Fixed

- The Windows build no longer warns about an unused `mut` in the PDF workspace.
- The release is published with `gh`, so the Node deprecation from the release
  action is gone; the git default-branch hint and the accessibility-bus warnings
  no longer appear in CI.

## [2.9.0] - 2026-11-16

### Added

- The plan's **page margins** are set in the app: **Margins** beside *Refresh*
  opens four fields (5–50 mm, presets for 1 inch, 20 mm, 0.5 inch and 10 mm) and
  reports the text area on A4. The choice drives the page view, *Save as PDF*
  (footer included) and the exported HTML's `@page`, so all three cannot
  disagree. One inch remains the default.
- Importing a load chart fills the crane name and its rated capacity from the
  workbook's own title line ("Tadano TL200M (20 ton)"). A workbook without that
  line leaves what the operator typed alone.

### Changed

- The Crane tab labels its machine field **Crane name**, matching the value it
  holds.

- In the uniform report, every factor in **Rigging factors & tension
  calculations** is laid out two to a row, the way the crane's required
  capacity and usage are: the angle factor beside the base reeve factor, and a
  choke's reduction beside the combined reeve factor. The two-leg basket's
  three factors and a nested hitch's longer chain follow the same grid, with a
  lone last factor spanning the row.

- The uniform report's **Note** section is one row: the choke notes and the
  tag-rating rule sit side by side, under one-line headings whose rules line up
  ("Choke Notes" / "Sling Legs per Tag Rating"). The choker note no longer
  prints the "(1) Percent of sling rated capacity…" paragraph, and its
  reduction table is split in two side by side (120° to 60°, then the rest).
- The divider in a two-column row is set by the content, not fixed at 42/58: it
  sits at the true centre when both sides fit, and moves to give room to
  whichever side needs it.

### Fixed

- A two-leg choke said "the main lifting leg stands perfectly vertical (0°)"
  while printing an angle factor below 1.0. Only the single-leg chokes stand
  vertical; the note now appears for those alone.

### Removed

- The crane's booked rating is no longer checked against the 75% rule. The load
  chart is the only rating used: it is the one that applies at the boom and
  radius in use. The rated capacity is still printed for the record, and the
  Crane tab no longer shows a "Booked rating check" row.

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
