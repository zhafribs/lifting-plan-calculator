# Architecture

A map of the code for the next person. The rule behind it is simple: **one file,
one job.** If you cannot say in one sentence what a file is for, it is either two
files or it belongs elsewhere. Files are deliberately not merged for the sake of
having fewer files.

## The shape of a run

```
ui/index.html          shell markup, early theme paint, startup-error banner
ui/js/main.js?         (no: app.js is the entry point, imported by index.html)
ui/js/app.js           rail, topbar, screens, toasts, the publish cycle
   -> screens/*.js     one module per tab
   -> api.js           every Tauri command the UI may call
   -> store.js         the single solved state every screen reads
Rust (src-tauri/src)
   lib.rs              Tauri setup, command registration, the QA probe hook
   commands.rs         the command surface: solve, chart import, report, files
   calc/…              the engine (see below)
   report*.rs          the plan document
   pdf*.rs             the export paths
```

## Rust modules

| File | Responsibility |
| --- | --- |
| `calc.rs` | Load at the hook, usage bands, the arithmetic every screen shares. |
| `crane.rs` | Cranes, load charts, the capacity check. |
| `sling.rs` | Sling geometry and tension, hitch specs. |
| `nonuniform.rs` | Off-centre and tandem cases. |
| `outrigger.rs` | Outrigger pad sizing (BS 8004). |
| `figures.rs` | Rounding and the figures shown on the report. |
| `excel.rs` | Reading a load chart out of `.xlsx`. |
| `model.rs`, `dto.rs` | The wire types between the UI and the engine. |
| `report.rs` | Assembles the plan document from the results. |
| `report_style.rs` | The report's stylesheet, one string, shared by every printer. |
| `report_capacity.rs`, `report_uniform.rs`, `report_nonuniform.rs`, `report_outrigger.rs` | One section of the document each. |
| `pdf.rs` | Page margins, footer stamping, the PDF bytes. |
| `pdf_print.rs`, `pdf_windows.rs` | The Linux and Windows print paths. |
| `commands.rs` | The Tauri commands and the file dialogs behind them. |
| `update.rs` | Update check, verified download, per-platform install. |
| `appearance.rs` | The stored scheme preference, the wallpaper palette (pywal/wallust) and the desktop accent. |
| `theme_windows.rs` | Windows-only accent lookup. |

## Frontend modules

| File | Responsibility |
| --- | --- |
| `app.js` | The shell: navigation, topbar, toasts, appearance, update banner. |
| `screens/overall.js`, `crane.js`, `outrigger.js`, `uniform.js`, `nonuniform.js`, `tandem.js`, `report.js`, `about.js` | One tab each. |
| `screens/graph.js` | The working-range diagram: drawing, interaction, capture. |
| `graph/geometry.js` | Diagram geometry, no DOM. |
| `graph/print-palette.js` | Turns the dark palette into print colours in the bitmap. |
| `graph/snapshot.js` | The registration point the report uses to ask for a fresh diagram. |
| `components.js` | The shared widgets (card, field, select, badges, meters). |
| `dom.js`, `format.js`, `store.js` | Element helper, number formatting, the solved state. |
| `api.js` | One function per Tauri command; nothing else talks to Rust. |
| `theme.js` | Scheme resolution and accent tokens, no DOM. |
| `windows-theme.js` | Platform scope, rail behaviour, accent maths. |
| `pdf-math.js`, `pdf-viewer.js` | Page fitting and the PDF page view. |

## Stylesheets

| File | Responsibility |
| --- | --- |
| `tokens.css` | The type face, the measurements, and both palettes. |
| `app.css` | Layout and components, reading only token names. |
| `windows.css` | The Fluent layer, every rule scoped to `[data-platform="windows"]`. |

## Where to add something

- **A screen**: new `screens/<name>.js` exporting `create<Name>Screen(ctx)`, one
  entry in `NAV` in `app.js`, one `create…Screen(ctx)` beside the others. It gets
  state through `ctx.store`, never from another screen.
- **A report section**: new `report_<section>.rs` with one `pub fn …_section`,
  called from `report.rs`. Styling goes in `report_style.rs`.
- **A command**: the function in `commands.rs` (or the module that owns the
  behaviour), then one line in `generate_handler!` in `lib.rs`, then one wrapper
  in `api.js`. The UI may not call `window.__TAURI__` directly.
- **A palette rule**: a token in `tokens.css`, in both blocks, and a rule in
  `app.css` that reads it. A per-theme rule in `app.css` is a smell.
- **An update path**: `update.rs` already has three (Windows installer, AppImage,
  Arch package). Each is a `#[cfg]`-gated `install*` function, detected by what
  the running copy is; add the detection first, then the install.

## Tests

| Where | What |
| --- | --- |
| `src-tauri/src/*.rs` `mod tests` | The engine, the report HTML, the update policy, the accent mapping. |
| `ui/js/*.test.mjs` | Pure frontend rules: page maths, jib rules, print palette, appearance. |
| `src-tauri/tests/` | End-to-end fixtures (a WebKit-printed plan, an update manifest). |
| CI | `cargo test`, `clippy -D warnings`, `cargo audit`, the node tests, the vendored-asset hashes, and a PDF probe on Windows. |

## Known seams (next restructure, in order)

1. `screens/graph.js` is 1 200 lines: drawing, interaction and capture in one.
   Split is `graph/draw.js` (canvas in, state out) and the screen module keeping
   DOM and events. The capture is already isolated behind `graph/snapshot.js`.
2. `commands.rs` carries the export probe. Move `export_probe`, `probe_one` and
   `probe_refusals` to `probe.rs`; `check_pdf` stays shared.
3. `excel.rs` reads charts, and is big by nature. Split only if a second reader
   (`.csv`, a chart API) is ever added.
4. The engine files (`crane.rs`, `sling.rs`, `nonuniform.rs`) are cohesive and
   heavily tested; splitting them would add indirection without a reader.
