## [2.17.7] - 2026-11-21

### Changed

- **The outrigger report no longer repeats the stiffener rule.** The section
  already prints "Stiffeners: required, or use a 40–50 mm plate"; the note two
  lines below said the same thing in prose. The flag and the row stay.
- **Load and tackle lines are ledger rows.** Each line leads with its identity
  and its result — description on the left, subtotal on the right, sharing one
  edge down the card — and carries the working underneath: the measured-by
  choice, the quantity, and the measurement (one box for fixed weight, two side
  by side for per-metre and per-volume). Each box is sized to the figure it
  holds: `00.0` for metres, `0000.0` for the per-unit figures. Rows are
  separated by hairlines instead of nested cards; the description keeps a soft
  field edge so it reads as typeable, and the add row is dashed in the accent
  colour.
- **Every result list uses the same figure scale.** Values are 600 weight with
  a small muted unit, so the overall weight stays the one coloured hero figure
  and the crane, sling, outrigger and tandem figures read the same as the item
  rows.

### Fixed

- **A two-box measurement never stacks.** When the working line is too narrow,
  the pair moves to a line of its own and stays side by side. The summary's
  one-line values also stop running into their labels at narrow widths.

## [2.17.6] - 2026-11-20

### Fixed

- **The fixed-weight dropdown stays between the quantity and the value box.**
  With the rail expanded in a small window, the card shrank below the row's
  width; the 62px quantity slid under the 156px dropdown and clipped "Fixed"
  to "ed". Below a 485px card the quantity has its own column and the dropdown
  takes the space left between it and the value box, so the row stays on two
  lines without overlapping.

## [2.17.5] - 2026-11-20

### Fixed

- **Dropdowns are drawn by the app, not left to the platform.** A native
  dropdown opens as its own window; on some Linux compositors that window is
  oversized and covers the control, which made the selected text look clipped
  and the list enormous. Every dropdown now opens a compact app-drawn list —
  grouped where the options are grouped — while the underlying `<select>` keeps
  the value, focus and keyboard behaviour.

## [2.17.4] - 2026-11-20

### Fixed

- **A per-metre or per-volume line now reads in the order it is used.** 2.17.3
  put "Measured by" on a full-width line with the two value boxes below it. The
  row is "Weight by Metre" beside "kg/m", then the quantity beside "m"; per
  volume, "Weight by Volume" beside "m3", then the quantity beside "kg/m3". The
  subtotal closes the row.

## [2.17.3] - 2026-11-20

### Added

- **The rail collapses to icons on Linux too.** The header button that Windows
  already had is now on every platform: it folds the navigation to a 64px strip
  and back, keeps the name and version in the footer, and leaves the choice
  alone when the window is resized. Until it is clicked, the rail still follows
  the window width across the 1100px breakpoint.

### Fixed

- **A per-metre or per-volume load line reads its two boxes on separate
  lines.** "kg/m" and its number used to share a 122px cell, so the unit took
  the width the figure needed. The pair stacks at every window size; below
  620px the "Measured by" dropdown takes a full-width line of its own and the
  quantity moves beneath it, so "Weight by Volume" is never clipped.

## [2.17.2] - 2026-11-20

### Added

- `SKILLS.md` gains a `humanizer` mode, adapted from `blader/humanizer` (MIT):
  the prose rules for docs, release notes and replies.

### Changed

- **The Windows install is a normal per-machine install.** It registers the
  publisher in Add/Remove Programs and puts the licence and the third-party
  notices beside the executable. The frontend, KaTeX and the PDF viewer are
  inside the executable and the C runtime is linked statically, so a small
  folder is the complete app, not a partial one. The installer now carries the
  WebView2 bootstrapper, so a machine without the runtime can still install.

### Fixed

- **The page-margin unit no longer covers the number spinner.** On Linux the
  native arrows were drawn under the "mm" suffix in the Summary tab's margins
  panel; the spinner is hidden on every platform now, as it already was on
  Windows.
- **Tooltips are small and app-drawn.** Hovering a rail tab opened a large
  black window with the label against one edge. The app draws its own bubble,
  on hover and on keyboard focus; the report preview keeps its accessible name
  without a hover hint.
- **The rail footer fits the minimum window.** Below 1100px the contact line
  broke the address mid-word, and at 720×540 it ran past the bottom edge. It is
  hidden there; the About screen prints it in full.
- **An update download is bounded.** The installer or AppImage is capped at
  512 MiB and hashed in chunks, so a broken or hostile release endpoint can no
  longer exhaust memory by sending an endless body.
- **Saving the HTML report is atomic.** The page is written beside the chosen
  file and moved into place, so a failed save leaves the previous report intact
  instead of a truncated one.

## [2.17.1] - 2026-11-20

### Added

- `AGENTS.md`, guidance for coding agents: the build and test commands, the
  project layout, and the rules the build enforces. No application change — the
  installed app behaves exactly as 2.17.0.

### Changed

- `AGENTS.md` names all three files that carry the version: `Cargo.toml`,
  `tauri.conf.json` and `Cargo.lock`.

## [2.17.0] - 2026-11-19

### Added

- **Three curated palettes in Settings: Clay, Lavender and Pine.** Each has a
  light and a dark accent, and the shell's surfaces are tinted to match. `Match
  the desktop` remains the default and keeps following the system accent and
  palette. The set is deliberately small; there is no free colour picker, so the
  contrast and the shell's identity stay under control.
- The palette choice is stored beside the scheme in
  `~/.config/lifting-plan-calculator/appearance.json`; an older file without the
  setting means `Match the desktop`.

### Fixed

- The text colour on a filled accent button now chooses whichever of black or
  white contrasts more. Mid-tone accents such as Clay previously received white
  text at about 2.9:1; they now get black at about 7:1.

## [2.16.13] - 2026-11-19

### Changed

- **The Add buttons follow the palette.** They were a fixed green that had no
  relation to the desktop accent; they now use the accent-tinted fill and border
  with the normal text colour. Success and warning bands keep their semantic
  green.
- **Explicit Light and Dark surfaces carry the palette accent.** The tint was so
  faint that the result read as flat white or black. The app's own ladder is now
  visibly tinted from the wallpaper accent, giving cream/light brown and dark
  brown for a brown accent, and the matching lavender pair. `Match the
  desktop` still uses the desktop's exact palette.

## [2.16.12] - 2026-11-19

### Fixed

- **An explicit Light or Dark choice now overrides the desktop's surfaces.** If
  the desktop published a palette for the other scheme — a dark Noctalia palette
  while choosing Light — its tinted surfaces were applied anyway, so only part
  of the shell changed. The desktop palette is now used only when it matches the
  resolved scheme; an override uses our own palette for that scheme, tinted from
  the wallpaper accent when there is one.
- **The Windows title bar follows the in-app scheme.** Choosing Light or Dark now
  sets the native window theme as well, so the title bar and Mica no longer stay
  on the system scheme while the page changes.

## [2.16.11] - 2026-11-19

### Changed

- The first section of every sling arrangement is named for it:
  `LIFT INPUT PARAMETERS - UNIFORM LOAD`, `... - NONUNIFORM LOAD`,
  `... - TANDEM`. Previously only the uniform case carried the suffix, and the
  nonuniform and tandem inputs sat under a bare sub-heading.
- The nonuniform and tandem inputs use the same three-column figure cards as the
  uniform inputs, instead of a long key/value table.

## [2.16.10] - 2026-11-19

### Fixed

- **Section numbers follow what is actually printed.** The builders carried
  their own numbers, so adding the outrigger pad section produced two "2."
  headings and left the rest out of step. The report is renumbered in print
  order at the end: with crane, diagram, outrigger and uniform ticked, the cards
  run 1 to 6 and the last is FINAL CAPACITY & EQUIPMENT SELECTION.
- **The uniform load inputs are named.** The card reads
  `3. LIFT INPUT PARAMETERS - UNIFORM LOAD` instead of leaving it ambiguous.
- **Only one sling arrangement at a time.** Ticking Uniform, Nonuniform or
  Tandem clears the other two, with their page-break ticks.
- **Windows: the update no longer fails with "requires elevation" (os error
  740).** The per-machine setup cannot be started with CreateProcess from a
  non-elevated app; it now goes through the shell's `runas` verb, which shows
  the UAC prompt the installer expects.

## [2.16.9] - 2026-11-19

### Added

- **A page break per section.** Every row in Report sections has a New page tick:
  tick it and that section starts on a fresh page, in the page view and in the
  saved PDF. The report still finds its own breaks when nothing is ticked.

### Fixed

- **Windows: the appearance choice now works.** The Fluent layer followed the
  operating system's light/dark setting through a media query, so Light and Dark
  in Settings changed the stored value but not the window. It now follows the
  app's own choice, like the rest of the shell.
- **Windows: the choice is remembered.** The setting was written under `$HOME`,
  which a normal Windows session does not have; it now goes to `%APPDATA%`.
- **A failed installer says where the file is**, so an update that cannot start
  can be finished by hand instead of dead-ending.

## [2.16.8] - 2026-11-19

### Fixed

- **The app follows the wallpaper again, and now exactly.** Noctalia writes its
  colours into the generated GTK stylesheet (`gtk-3.0/gtk.css` importing
  `noctalia.css`), which is what the terminals and the file manager paint with.
  The app read only the terminal's 16 colours and picked the most vivid one — a
  yellow on a pink wallpaper. It now reads the GTK roles first (`accent_color`,
  `window_bg_color`, `window_fg_color`, `headerbar_bg_color`) and paints its
  surfaces from them, so it matches the rest of the desktop instead of
  approximating it.

- **A wallpaper change no longer needs a restart or an alt-tab.** While the
  window is in front the palette is re-read every 20 seconds, in addition to the
  refresh on focus.

- **A section title can no longer end a page, in any section.** The
  keep-with-next rule was scoped to `section-card`, but "Lift Input Parameters"
  lives in `input-card` — the case in your plan. The rule is now stated on every
  report heading and the block that follows it; measured cost: no extra page
  (4 sections, 4 pages), and the orphaned heading moves with its rows.

## [2.16.7] - 2026-11-19

### Changed

- A new plan's Summary starts with the crane check ticked and every other
  section clear, including the working-range diagram. The sections a plan needs
  beyond that are opt-in.

### Fixed

- The report preview's two modes share one frame size: the page list and the
  live view now fill the same pane instead of one being shorter than the other.
- The live preview is laid out at the print text width and zoomed to the pane,
  so it reads at the size the page view shows instead of at 7 pt in a column.
- A section title can no longer end a page. Keep-with-next is now stated on both
  sides — the heading never ends a page, and the block after it never starts
  one — which Chrome honours without costing a page.

## [2.16.6] - 2026-11-19

### Changed

- The Updates card moved from About to Settings, beside the appearance: one page
  holds the controls. About keeps what the app does and who built it.
- The launch reminder no longer carries its own installer buttons. It says a
  release exists, links to its notes, and opens Settings, so the update paths
  live in one place instead of two copies that could drift.

## [2.16.5] - 2026-11-19

### Added

- **A Settings tab**, above About, holding the appearance: Match the desktop,
  Light or Dark, and a line saying where the accent came from. It left the About
  page, which keeps the update path and the credits.

- **Help for the Arch update when polkit is missing.** The in-app update needs
  pkexec and a running polkit agent. When either is absent, About now says which
  one and offers the command that installs it — `polkit` itself, or
  `hyprpolkitagent` on Hyprland (`polkit-gnome` elsewhere) — then you press the
  button again. A cancelled password prompt installs nothing and suggests
  nothing.

### Changed

- The manual `sudo pacman -U` command is always offered next to the packaging
  help, for an operator who would rather not use the button at all.

## [2.16.4] - 2026-11-19

### Fixed

- A Noctalia desktop was not recognised, although it recolours the terminals and
  the GTK applications from the wallpaper. The app now reads Noctalia's own
  palette — the kitty theme it writes — and its light/dark choice, so a
  wallpaper-themed Hyprland session needs no configuration at all. The generated
  `gtk.css` (`@define-color accent_color`) is read as a fallback, which also
  covers matugen-style setups.
- The desktop's published scheme now wins over the webview's guess, so the
  palette follows Noctalia's mode even when GTK does not advertise it.

## [2.16.3] - 2026-11-19

### Added

- The app follows the wallpaper on setups that publish a palette. When pywal or
  wallust has written `~/.cache/wal/colors.json`, `~/.cache/wallust/colors.json`
  or `~/.cache/wal/colors`, the most usable colour of that palette becomes the
  accent and the surfaces are tinted with its hue — the same way the rest of a
  wallpaper-themed Hyprland desktop follows it. The tint is re-read when the
  window is focused, so changing the wallpaper does not need a restart.
- About now says where the accent came from: the wallpaper palette, the
  desktop's accent setting, or the app's own blue.

## [2.16.2] - 2026-11-19

### Changed

- The code is split by responsibility, with a written map in
  [`docs/architecture.md`](docs/architecture.md): the report's stylesheet moved
  to `report_style.rs`, and the palettes and measurements moved out of `app.css`
  to `tokens.css`. No behaviour changed.

## [2.16.1] - 2026-11-19

### Added

- The Arch package updates from the app's own button. When the running copy was
  installed by pacman, About offers Download & install: the release's
  `.pkg.tar.zst` is downloaded, checked against the signed `SHA256SUMS.txt`, and
  handed to `pkexec pacman -U`. A desktop with no polkit agent is not a dead
  end — the app shows the exact `sudo pacman -U` command, with a copy button,
  for the file it already verified.

## [2.16.0] - 2026-11-19

### Added

- A neutral shell of our own for Linux, in a light and a dark palette, with the
  same structure on every distribution and desktop. It follows the desktop's
  light/dark preference and borrows its accent colour when one is published
  (GNOME and KDE today), and About offers Match the desktop, Light or Dark.
  The choice is remembered in `~/.config/lifting-plan-calculator/appearance.json`.
- Windows keeps its Fluent layer, now sharing the same accent plumbed from the
  system.

### Changed

- Filled buttons carry their own text colour, so the primary and add/danger
  buttons stay readable in both palettes.

## [2.15.1] - 2026-11-19

### Fixed

- Dropdowns read as dropdowns again. The Windows Fluent layer had reset the
  base select chevron and its right padding, so a dropdown looked exactly like
  a text field. The chevron is restored in the Windows layer.
- The Qty field no longer spills under the Measured-by select in the narrow
  two-line load row when the rail is expanded in a small window. The Qty column
  keeps its 62 px width and the select takes the remaining space.

## [2.15.0] - 2026-11-18

### Added

- A Windows 11 Fluent shell: a native title bar over Mica, a NavigationView rail
  that folds from 240 px to a 56 px icon strip, Segoe UI Variable, Fluent cards,
  controls, badges and list rows, and the system light/dark setting and accent
  colour. Windows 10 gets the same layout on solid surfaces.
- Fluent light and dark token sets in `ui/css/windows.css`, loaded after the
  unchanged base theme and scoped to the Windows platform marker.
- `--platform-preview=windows` to preview the Windows layout on Linux; it swaps
  the tokens without Mica.

### Fixed

- Unit suffixes no longer overlap a right-aligned number in a field.
- An expanded Windows rail keeps its labels below 1100 px instead of inheriting
  the base stylesheet's narrow-window collapse.

### Changed

- The README and tutorial document the Windows 11 shell, the Windows 10 fallback
  and the unchanged report, with light, dark and compact screenshots.

## [2.14.0] - 2026-11-18

### Added

- Linux packages for every common distribution, built from the same release
  binary: `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), `x86_64.pkg.tar.zst`
  (Arch) and a single-file `.flatpak` on the GNOME 50 runtime. Each depends on
  the distribution's own GTK 3 and WebKitGTK 4.1; none carries its own copy.

### Fixed

- The printed working-range diagram keeps its original height (78 mm) and is
  sharp: the report no longer filters the image, which made the print pipeline
  rasterise the capture again at 300 dpi. The colours are turned over in the
  bitmap instead, so the diagram is embedded at its full 2700 px (about
  684 ppi at this size).
- The Windows installers are per-machine: the NSIS setup installed per-user,
  which put the app in one profile instead of `C:\Program Files`. Both the MSI
  and the setup now install for all users, with the administrator prompt that
  implies.
- The Windows build no longer warns that the AppImage update helper is unused;
  the helper is compiled only on Linux, where it is written.

### Changed

- The README's per-release "What's new" block is gone. Those summaries belong on
  the Releases page, which already carries them; the README keeps the download
  table and the scope of use.

## [2.13.5] - 2026-11-18

### Fixed

- The working-range diagram prints legibly: it now spans the report's full text
  width instead of being capped at 78 mm of height, its labels are drawn 25 %
  larger for the page, and the capture is three times the screen size so the
  lines and figures stay sharp at print resolution.

## [2.13.4] - 2026-11-18

### Changed

- **The printed bearing-area formula is one line.** The report now prints
  `A = 0.65 × (CM + L) / V = ...` as a single equation instead of two.
- **Outrigger labels mark the variable.** `Design (V)`, `Design load (P)` and
  `Required pad area (A)` replace the unmarked labels.
- **The pad card separates its extra fields.** The rectangular pad width and the
  timber 45° foot fields sit under their own labelled sections, and the
  **Use crane tab** / **Use overall weight** buttons sit directly under their
  matching load fields.

## [2.13.3] - 2026-11-18

### Added

- **Outrigger pad thickness guidance.** The Outrigger Pad Size tab now selects
  steel plate or hardwood timber and shows the recommended thickness by crane
  class, the 45° timber overhang check, cross-layering for pads wider than
  1.2 m, steel stiffener limits and the materials to avoid. The same guidance
  is structured on the **Thickness** sheet of
  `docs/examples/Presumed Allowable Bearing Values (BS 8004 Table 1).xlsx`.

### Changed

- The outrigger report drops the introductory and closing paragraphs, moves the
  ground note below the result and removes the “at 50 mm increments” wording.
  The minimum-pad sentence keeps its normal size.

### Fixed

- **A GitHub API rate limit no longer disables the update check.** The app falls
  back to the repository's releases Atom feed when the API returns 403, then
  builds the same release asset names for the download.

## [2.13.2] - 2026-11-18

### Changed

- **The outrigger result is the minimum pad, not a usage band.** The formula
  already decides the minimum bearing area, so the selected-pad applied
  pressure, utilisation and 90/100% bands are removed. A square pad reports the
  minimum square side; a rectangular pad reports the minimum length for the
  chosen width. A larger pad is acceptable.
- The outrigger form is reflowed: the bearing value and the published table
  value share a row, and the crane/lift actions sit on their own line.

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
