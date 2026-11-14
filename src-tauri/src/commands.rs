//! The Tauri command surface: everything the UI can ask the engine to do.

use std::path::Path;
use std::sync::Mutex;
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::crane::{solve_crane, CraneInputs};
use crate::dto::{build_catalog, table_for, AppState, Catalog, ChartImport, ChartTable, PlanSignoff, SolveAll};
use crate::excel::{
    self, boom_changed, forget_chart, imported_inputs, radius_changed, LoadChartData,
};use crate::nonuniform::{solve_nonuniform, NonuniformResult};
use crate::report::{build_report_html, wrap_katex, ReportInputs};
use crate::sling::solve_one_leg;

/// The parsed workbook behind the current import, held for the boom and radius
/// updates that re-read it.
pub struct ChartHolder(pub Mutex<Option<LoadChartData>>);

// A tiny envelope so the UI can read the app's identity without a second
// round trip. `start_screen` lets a launcher open a named screen directly
// (`--screen=crane`), which is also how the UI is smoke-tested. `start_hitch`
// lands the uniform tab on a chosen sling arrangement (`--sling-hitch=round_choke`).
// `start_chart` imports a load-chart workbook at launch (`--chart=path.xlsx`),
// which is how the workbook-driven screens are smoke-tested; `chart_radius`
// lands that workbook on a chosen working radius (`--chart-radius=12`). The
// update banner has its own QA hook (`--update-check-version=1.0.0`) read by
// `update::check_update`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppInfo {
    pub app_name: String,
    pub version: String,
    /// `std::env::consts::OS`: "windows", "linux", "macos".
    pub platform: String,
    pub start_screen: Option<String>,
    pub start_hitch: Option<String>,
    pub start_chart: Option<String>,
    pub chart_radius: Option<String>,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    let start_screen = std::env::args()
        .find_map(|argument| argument.strip_prefix("--screen=").map(str::to_string));
    let start_hitch = std::env::args()
        .find_map(|argument| argument.strip_prefix("--sling-hitch=").map(str::to_string));
    let start_chart = std::env::args()
        .find_map(|argument| argument.strip_prefix("--chart=").map(str::to_string));
    let chart_radius = std::env::args()
        .find_map(|argument| argument.strip_prefix("--chart-radius=").map(str::to_string));
    AppInfo {
        app_name: "Lifting Plan Calculator".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        start_screen,
        start_hitch,
        start_chart,
        chart_radius,
    }
}

#[tauri::command]
pub fn catalog() -> Catalog {
    build_catalog()
}

/// A tiny breadcrumb channel for the frontend: anything it logs lands in
/// `~/.cache/lifting-plan-calculator/ui.log`, which is how startup problems are
/// diagnosed when the window itself cannot say what went wrong.
#[tauri::command]
pub fn log_ui_event(line: String) {
    let path = std::env::var("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache")
        })
        .join("lifting-plan-calculator")
        .join("ui.log");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write;
        // Keep the breadcrumb log small: start over once it passes 64 kB.
        if file.metadata().map(|meta| meta.len() > 64 * 1024).unwrap_or(false) {
            let _ = std::fs::write(&path, b"");
            let _ = writeln!(file, "-- log rotated --");
        }
        let _ = writeln!(file, "{line}");
    }
    eprintln!("ui: {line}");
}

fn solve_all_internal(state: &AppState) -> SolveAll {
    let totals = state.totals();
    let line_weights: Vec<f64> = state.load_items.iter().map(|item| item.weight()).collect();
    let tackle_subtotals: Vec<f64> = state
        .tackle_items
        .iter()
        .map(|item| item.subtotal())
        .collect();

    let crane = match solve_crane(&state.crane, totals.gross) {
        Ok(result) => crate::dto::Section::solved(result),
        Err(error) => crate::dto::Section::failed(error.0),
    };

    let sling = match solve_one_leg(&state.sling, totals.gross) {
        Ok(result) => crate::dto::Section::solved(result),
        Err(error) => crate::dto::Section::failed(error.0),
    };

    let nonuniform = match solve_nonuniform(&state.nonuniform, totals.gross) {
        Ok(result) => crate::dto::Section::solved(result),
        Err(error) => crate::dto::Section::failed(error.0),
    };

    let tandem = match solve_nonuniform(&state.tandem, totals.gross) {
        Ok(result) => crate::dto::Section::solved(result),
        Err(error) => crate::dto::Section::failed(error.0),
    };

    SolveAll {
        totals,
        line_weights,
        tackle_subtotals,
        crane,
        sling,
        nonuniform,
        tandem,
    }
}

#[tauri::command]
pub fn solve_all(state: AppState) -> SolveAll {
    solve_all_internal(&state)
}

/// Open the workbook picker. Returns the chosen path, or None when cancelled.
#[tauri::command]
pub async fn pick_chart_file() -> Result<Option<String>, String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Open Excel load chart")
        .add_filter("Excel workbook", &["xlsx"])
        .add_filter("All files", &["*"])
        .pick_file()
        .await;
    Ok(handle.map(|file| file.path().to_string_lossy().to_string()))
}

/// Read a workbook, apply it to the crane form, and remember it for the boom
/// and radius controls.
#[tauri::command]
pub fn import_chart(
    holder: tauri::State<'_, ChartHolder>,
    state: AppState,
    path: String,
) -> Result<ChartImport, String> {
    let bytes = std::fs::read(&path).map_err(|_| "That file could not be opened.".to_string())?;
    let chart = excel::read_chart(&bytes).map_err(|error| error.0)?;
    let name = Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "workbook.xlsx".to_string());
    let gross = state.totals().gross;
    let crane = imported_inputs(&state.crane, &chart, &name, gross);
    let table = table_for(&chart, crane.chart_boom);
    *holder.0.lock().expect("chart holder poisoned") = Some(chart);
    Ok(ChartImport { crane, table })
}

/// The actions the crane tab can take against the held workbook.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ChartAction {
    Boom { boom: f64, gross: f64 },
    Radius { radius: f64 },
    Jib {
        config: String,
        length: Option<f64>,
        offset: Option<f64>,
        #[serde(default)]
        gross: f64,
    },
    Forget,
    Manual { row: crate::crane::ChartRow },
}

/// The crane form after a chart control changed, plus the display table when
/// there is still a workbook behind it.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChartUpdate {
    pub crane: CraneInputs,
    pub table: Option<ChartTable>,
}

#[tauri::command]
pub fn chart_update(
    holder: tauri::State<'_, ChartHolder>,
    crane: CraneInputs,
    action: ChartAction,
) -> Result<ChartUpdate, String> {
    match action {
        ChartAction::Forget => Ok(ChartUpdate {
            crane: forget_chart(&crane),
            table: None,
        }),
        ChartAction::Manual { row } => Ok(ChartUpdate {
            crane: excel::manual_edit(&crane, row),
            table: None,
        }),
        ChartAction::Jib {
            config,
            length,
            offset,
            gross,
        } => {
            let guard = holder.0.lock().expect("chart holder poisoned");
            let chart = guard.as_ref().ok_or("No workbook is loaded.")?;
            Ok(ChartUpdate {
                crane: excel::jib_changed(&crane, chart, &config, length, offset, gross),
                table: None,
            })
        }
        ChartAction::Boom { boom, gross } => {
            let guard = holder.0.lock().expect("chart holder poisoned");
            let chart = guard.as_ref().ok_or("No workbook is loaded.")?;
            let updated = boom_changed(&crane, chart, boom, gross);
            let table = table_for(chart, updated.chart_boom);
            Ok(ChartUpdate {
                crane: updated,
                table: Some(table),
            })
        }
        ChartAction::Radius { radius } => {
            let guard = holder.0.lock().expect("chart holder poisoned");
            let chart = guard.as_ref().ok_or("No workbook is loaded.")?;
            let updated = radius_changed(&crane, chart, radius);
            let table = table_for(chart, updated.chart_boom);
            Ok(ChartUpdate {
                crane: updated,
                table: Some(table),
            })
        }
    }
}

/// The full report page, with the sections the caller asked for.
#[tauri::command]
pub fn build_report(
    state: AppState,
    include: Vec<String>,
    issued_at: String,
    signoff: PlanSignoff,
    working_diagram: Option<String>,
    footer_center: Option<String>,
) -> String {
    let solved = solve_all_internal(&state);
    let tandem = match solved.tandem.result {
        Some(NonuniformResult::Tandem(result)) => Some(result),
        _ => None,
    };
    let inputs = ReportInputs {
        totals: solved.totals,
        crane: solved.crane.result,
        sling: solved.sling.result,
        nonuniform: solved.nonuniform.result,
        nonuniform_error: solved.nonuniform.error,
        tandem,
        tandem_error: solved.tandem.error,
        crane_error: solved.crane.error,
        signoff,
        issued_at: Some(issued_at).filter(|value| !value.is_empty()),
        working_diagram,
        footer_center,
    };
    wrap_katex(&build_report_html(&inputs, &include))
}

/// Save the report beside a copy of its KaTeX assets, so the file typesets its
/// equations wherever it is opened. Returns the chosen path, or None when the
/// dialog was cancelled.
#[tauri::command]
pub async fn save_report(
    app: tauri::AppHandle,
    html: String,
    suggested_name: String,
) -> Result<Option<String>, String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Save lifting plan report")
        .set_file_name(&suggested_name)
        .add_filter("HTML page", &["html"])
        .save_file()
        .await;
    let Some(handle) = handle else {
        return Ok(None);
    };
    let target = handle.path().to_path_buf();
    let Some(parent) = target.parent() else {
        return Err("That destination cannot be written to.".to_string());
    };
    let stem = target
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "lifting-plan".to_string());
    let asset_folder = format!("{stem}_assets");

    // Rewrite the wrapper's asset references to the sibling folder, then write
    // the known KaTeX files there from the assets embedded in the binary.
    let rewritten = html.replace(
        "vendor/katex/",
        &format!("{asset_folder}/katex/"),
    );
    std::fs::write(&target, rewritten)
        .map_err(|error| format!("The report could not be saved: {error}"))?;

    let resolver = app.asset_resolver();
    let katex_dir = parent.join(&asset_folder).join("katex");
    std::fs::create_dir_all(katex_dir.join("fonts"))
        .map_err(|error| format!("The report assets could not be saved: {error}"))?;
    for file in crate::dto::KATEX_FILES {
        // The asset resolver may key embedded files with or without a leading
        // slash depending on the bundle; try both before giving up.
        let bare = format!("vendor/katex/{file}");
        let rooted = format!("/{bare}");
        let asset = resolver
            .get(bare.clone())
            .or_else(|| resolver.get(rooted));
        if let Some(asset) = asset {
            let destination = katex_dir.join(file);
            std::fs::write(&destination, asset.bytes)
                .map_err(|error| format!("The report assets could not be saved: {error}"))?;
        }
    }
    Ok(Some(target.to_string_lossy().to_string()))
}

/// The environment an AppImage injects for its own bundled GTK and glibc.
///
/// A system program started with these loads the AppImage's libraries and dies
/// on a symbol mismatch — system Chromium reported "undefined symbol:
/// BrotliDecoderAttachDictionary", which looked exactly like "not installed".
/// Every external program this module starts is handed a clean environment.
const APPIMAGE_ENV: [&str; 13] = [
    "APPDIR",
    "ARGV0",
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "GTK_DATA_PREFIX",
    "GTK_THEME",
    "GTK_EXE_PREFIX",
    "GTK_PATH",
    "GTK_IM_MODULE_FILE",
    "GDK_PIXBUF_MODULE_FILE",
    "GI_TYPELIB_PATH",
    "GIO_MODULE_DIR",
    "GSETTINGS_SCHEMA_DIR",
];

/// Strip the AppImage's own paths so a system program runs as the system built it.
fn outside_the_appimage(command: &mut std::process::Command) -> &mut std::process::Command {
    for key in APPIMAGE_ENV {
        command.env_remove(key);
    }
    command.env_remove("XDG_DATA_DIRS");
    command
}

/// Windows' PDF renderer: the Edge that ships with the system, or Chrome.
///
/// Linux does not come through here: the app prints the plan with its own web
/// view, so no browser has to be installed and none is carried in the AppImage.
#[cfg(target_os = "windows")]
fn pdf_browser_candidates() -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();
    // Program Files first: a per-user install is the least privileged place to
    // find an executable, so it is the last resort.
    for variable in ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"] {
        if let Ok(base) = std::env::var(variable) {
            let base = std::path::Path::new(&base);
            candidates.push(base.join("Microsoft/Edge/Application/msedge.exe"));
            candidates.push(base.join("Google/Chrome/Application/chrome.exe"));
        }
    }
    candidates
}

#[cfg(target_os = "windows")]
fn find_pdf_browser() -> Option<std::path::PathBuf> {
    pdf_browser_candidates()
        .into_iter()
        .find(|candidate| candidate.is_file())
}

/// `file:///…` with spaces escaped, for Windows paths as well as Unix ones.
fn file_url(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    let path = path.trim_start_matches('/');
    format!("file:///{}", path.replace(' ', "%20"))
}

fn write_katex_assets(app: &tauri::AppHandle, root: &Path) -> Result<(), String> {
    let resolver = app.asset_resolver();
    let katex_dir = root.join("vendor/katex");
    std::fs::create_dir_all(katex_dir.join("fonts"))
        .map_err(|error| format!("The PDF report assets could not be written: {error}"))?;
    for file in crate::dto::KATEX_FILES {
        let bare = format!("vendor/katex/{file}");
        let rooted = format!("/{bare}");
        if let Some(asset) = resolver.get(bare).or_else(|| resolver.get(rooted)) {
            std::fs::write(katex_dir.join(file), asset.bytes)
                .map_err(|error| format!("The PDF report assets could not be written: {error}"))?;
        }
    }
    Ok(())
}

/// What the report screen sends so the PDF's footer can be written on Linux,
/// where the app's own web view prints the pages and does not draw `@page`
/// margin boxes. Absent values fall back to the report's own defaults.
#[derive(Debug, Clone, Deserialize)]
pub struct PdfFooter {
    pub reference: Option<String>,
    pub center: Option<String>,
}

/// The footer a report is printed with: the plan reference, and the credit line
/// or the report's default.
fn printed_footer(footer: Option<&PdfFooter>) -> crate::pdf::Footer {
    let (reference, center) = crate::report::footer_texts(
        footer
            .and_then(|footer| footer.reference.as_deref())
            .unwrap_or(""),
        footer.and_then(|footer| footer.center.as_deref()),
    );
    crate::pdf::Footer { reference, center }
}

/// The last preview, kept so *Save as PDF* writes exactly the pages the
/// operator was looking at instead of printing them a second time.
struct CachedPreview {
    key: u64,
    bytes: Vec<u8>,
}

static PREVIEW_CACHE: Mutex<Option<CachedPreview>> = Mutex::new(None);
static PREVIEW_GENERATION: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
/// One print at a time: the GTK main thread drives every print through the same
/// web view, and a preview must not queue behind a stale one.
static PREVIEW_LOCK: Mutex<()> = Mutex::new(());

const SUPERSEDED: &str = "The preview was superseded by a newer change.";

/// A cheap, stable key for a report: its HTML and the footer it is printed with.
fn report_key(html: &str, footer: &crate::pdf::Footer) -> u64 {
    // FNV-1a, 64-bit: comparing the whole report would cost as much as the
    // render it is avoiding.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for bytes in [html.as_bytes(), footer.reference.as_bytes(), footer.center.as_bytes()] {
        for byte in bytes {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

/// Whether a newer preview has been asked for since `generation` was taken. A
/// superseded request must neither print nor store: its pages are already out
/// of date by the time they would be ready.
fn superseded(generation: u64) -> bool {
    PREVIEW_GENERATION.load(std::sync::atomic::Ordering::Relaxed) != generation
}

fn cached_preview(key: u64) -> Option<Vec<u8>> {
    PREVIEW_CACHE
        .lock()
        .ok()
        .and_then(|cache| cache.as_ref().filter(|entry| entry.key == key).map(|entry| entry.bytes.clone()))
}

/// Render a report into memory, through the same print and footer as the
/// export. The workspace is removed on both paths.
fn render_plan_bytes(
    app: &tauri::AppHandle,
    html: &str,
    footer: &crate::pdf::Footer,
) -> Result<Vec<u8>, String> {
    let workspace = private_workspace()?;
    let path = workspace.join("plan.pdf");
    let rendered = render_pdf_into(app, html, &path, footer);
    let bytes = match rendered {
        Ok(()) => std::fs::read(&path)
            .map_err(|error| format!("The preview could not be read back: {error}")),
        Err(error) => Err(error),
    };
    let _ = std::fs::remove_dir_all(&workspace);
    bytes
}

/// Render the report for the Summary preview. The UI shows these pages, and
/// *Save as PDF* writes these same bytes when the plan has not changed since.
#[tauri::command]
pub async fn preview_pdf(
    app: tauri::AppHandle,
    html: String,
    footer: Option<PdfFooter>,
) -> Result<tauri::ipc::Response, String> {
    let footer = printed_footer(footer.as_ref());
    let key = report_key(&html, &footer);
    if let Some(bytes) = cached_preview(key) {
        return Ok(tauri::ipc::Response::new(bytes));
    }

    let generation = PREVIEW_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let _guard = PREVIEW_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // A newer request arrived while this one waited: printing this report
        // now would waste a web view print and race the cache.
        if superseded(generation) {
            return Err(SUPERSEDED.to_string());
        }
        let bytes = render_plan_bytes(&app, &html, &footer)?;
        if superseded(generation) {
            return Err(SUPERSEDED.to_string());
        }
        if let Ok(mut cache) = PREVIEW_CACHE.lock() {
            *cache = Some(CachedPreview { key, bytes: bytes.clone() });
        }
        Ok(bytes)
    })
    .await
    .map_err(|error| format!("The preview could not be rendered: {error}"))??;
    Ok(tauri::ipc::Response::new(bytes))
}

fn render_pdf(app: &tauri::AppHandle, html: &str, target: &Path, footer: &crate::pdf::Footer) -> Result<(), String> {
    // The pages are printed, and then footered, into a staging file beside the
    // target. Only a finished PDF ever reaches the path the operator chose, so a
    // failed footer cannot leave a plan without its reference and page numbers.
    let staging = staged_path(target);
    let _ = std::fs::remove_file(&staging);
    let rendered = render_pdf_into(app, html, &staging, footer);
    match rendered {
        Ok(()) => move_into_place(&staging, target),
        Err(error) => {
            let _ = std::fs::remove_file(&staging);
            Err(error)
        }
    }
}

fn render_pdf_into(
    app: &tauri::AppHandle,
    html: &str,
    target: &Path,
    footer: &crate::pdf::Footer,
) -> Result<(), String> {
    let temp = private_workspace()?;
    let html_path = temp.join("report.html");
    // The footer is stamped onto the finished pages on every platform, so the
    // engines are handed the report without its `@page` margin boxes: no engine
    // version can decide whether a plan carries its reference and page numbers.
    std::fs::write(&html_path, crate::report::print_html(html))
        .map_err(|error| format!("The PDF report could not be prepared: {error}"))?;
    write_katex_assets(app, &temp)?;

    // Linux prints with the app's own web view, so no browser has to be
    // installed and none is carried in the AppImage. Windows keeps Edge.
    #[cfg(not(target_os = "windows"))]
    let printed = crate::pdf_print::print_to_pdf(
        app,
        file_url(&html_path),
        target.to_path_buf(),
    );
    // Windows prints with the WebView2 the app already runs on, so the export
    // works when Edge has been removed; Edge is the fallback. Each attempt
    // prints to its own file, so a print that is still finishing when the
    // fallback starts cannot overwrite the winner.
    #[cfg(target_os = "windows")]
    let printed = {
        let in_app = temp.join("webview2.pdf");
        let via_edge = temp.join("edge.pdf");
        let in_app_result = crate::pdf_windows::print_to_pdf(app, &html_path, &in_app);
        match in_app_result {
            Ok(()) => std::fs::rename(&in_app, target)
                .map_err(|error| format!("The PDF could not be written: {error}")),
            Err(first) => {
                let _ = std::fs::remove_file(&in_app);
                match render_with_browser(&html_path, &temp, &via_edge) {
                    Ok(()) => std::fs::rename(&via_edge, target)
                        .map_err(|error| format!("The PDF could not be written: {error}")),
                    Err(second) => Err(format!(
                        "The app's own printer failed ({first}); Edge then failed too ({second})."
                    )),
                }
            }
        }
    };

    let _ = std::fs::remove_dir_all(&temp);
    printed?;

    check_pdf(target)?;
    crate::pdf::stamp(target, footer)?;
    Ok(())
}

/// A fresh workspace nobody else can read or guess: the report's HTML and KaTeX
/// assets are written there before printing, and they carry the plan's data.
fn private_workspace() -> Result<std::path::PathBuf, String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let temp = std::env::temp_dir().join(format!(
        "lifting-plan-pdf-{}-{stamp}",
        std::process::id()
    ));
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(&temp)
        .map_err(|error| format!("The PDF workspace could not be created: {error}"))?;
    Ok(temp)
}

/// Where the PDF is written before it is known to be complete.
fn staged_path(target: &Path) -> std::path::PathBuf {
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "lifting-plan.pdf".to_string());
    target.with_file_name(format!(".{name}.part-{}", std::process::id()))
}

/// Write finished PDF bytes to the path the operator chose, through the same
/// staging file as a render, so a failure never leaves a partial plan.
fn write_bytes(bytes: &[u8], target: &Path) -> Result<(), String> {
    let staging = staged_path(target);
    std::fs::write(&staging, bytes)
        .map_err(|error| {
            let _ = std::fs::remove_file(&staging);
            format!("The PDF could not be written to that folder: {error}")
        })?;
    move_into_place(&staging, target)
}

/// Move the finished PDF to the path the operator chose.
fn move_into_place(staging: &Path, target: &Path) -> Result<(), String> {
    if !staging.is_file() {
        return Err("The PDF was not written, so the chosen file was left as it was.".to_string());
    }
    // On Unix this replaces the target in one step. Windows refuses to rename
    // over an existing file, so it is removed first — and only then, never
    // before the finished PDF is known to exist.
    match std::fs::rename(staging, target) {
        Ok(()) => Ok(()),
        Err(_first) if target.exists() => {
            std::fs::remove_file(target).map_err(|error| {
                format!("The existing PDF could not be replaced: {error}")
            })?;
            std::fs::rename(staging, target).map_err(|error| {
                let _ = std::fs::remove_file(staging);
                format!("The PDF could not be written to that folder: {error}")
            })
        }
        Err(error) => {
            let _ = std::fs::remove_file(staging);
            Err(format!("The PDF could not be written to that folder: {error}"))
        }
    }
}

/// Reject a print that wrote nothing, or something that is not a PDF.
fn check_pdf(target: &Path) -> Result<(), String> {
    use std::io::Read;
    let mut file = std::fs::File::open(target)
        .map_err(|error| format!("The PDF was not written: {error}"))?;
    let mut header = [0u8; 5];
    file.read_exact(&mut header)
        .map_err(|_| "The PDF renderer wrote an empty file.".to_string())?;
    if &header != b"%PDF-" {
        let _ = std::fs::remove_file(target);
        return Err("The PDF renderer did not produce a PDF. The report was not saved.".to_string());
    }
    Ok(())
}

/// Windows: print with the Edge the system already carries.
#[cfg(target_os = "windows")]
fn render_with_browser(
    html_path: &Path,
    temp: &Path,
    target: &Path,
) -> Result<(), String> {
    let browser = find_pdf_browser().ok_or_else(|| {
        "No PDF renderer was found. Microsoft Edge is part of Windows 10 and 11; \
         if it has been removed, reinstall it to save plans as PDF."
            .to_string()
    })?;
    let profile = temp.join("profile");
    std::fs::create_dir_all(&profile)
        .map_err(|error| format!("The PDF workspace could not be created: {error}"))?;

    let mut child = outside_the_appimage(&mut std::process::Command::new(&browser))
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--no-pdf-header-footer")
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg(format!("--print-to-pdf={}", target.display()))
        .arg(file_url(html_path))
        .spawn()
        .map_err(|error| format!("The PDF renderer could not start: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(90);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = std::fs::remove_file(target);
                return Err("The PDF renderer did not finish within 90 seconds.".to_string());
            }
            Err(error) => {
                let _ = std::fs::remove_file(target);
                return Err(format!("The PDF renderer could not be followed: {error}"));
            }
        }
    };

    if !status.success() {
        let _ = std::fs::remove_file(target);
        return Err(format!(
            "The PDF renderer exited with {status}. The report was not saved."
        ));
    }
    Ok(())
}

/// Write the report as a PDF with the report's own 1-inch page margin.
#[tauri::command]
pub async fn save_pdf(
    app: tauri::AppHandle,
    html: String,
    suggested_name: String,
    footer: Option<PdfFooter>,
) -> Result<Option<String>, String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Save lifting plan as PDF")
        .set_file_name(&suggested_name)
        .add_filter("PDF document", &["pdf"])
        .save_file()
        .await;
    let Some(handle) = handle else {
        return Ok(None);
    };
    let mut target = handle.path().to_path_buf();
    if !target
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
    {
        target.set_extension("pdf");
    }

    let footer = printed_footer(footer.as_ref());
    let key = report_key(&html, &footer);
    let cached = cached_preview(key);

    let target_for_render = target.clone();
    tauri::async_runtime::spawn_blocking(move || match cached {
        // The operator is saving what the preview showed: write those bytes,
        // so the file cannot differ from the pages on screen.
        Some(bytes) => write_bytes(&bytes, &target_for_render),
        // One print at a time, so saving and previewing cannot fight over the
        // GTK main thread.
        None => {
            let _guard = PREVIEW_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            render_pdf(&app, &html, &target_for_render, &footer)
        }
    })
    .await
    .map_err(|error| format!("The PDF export failed: {error}"))??;
    Ok(Some(target.to_string_lossy().to_string()))
}

/// Render a one-page probe report through the real export path and check the
/// result. This is the hook the Windows CI job runs: it proves that the PDF
/// export works on the platform, which a compile check cannot.
pub fn export_probe(app: &tauri::AppHandle, target: &Path) -> Result<(), String> {
    let footer = crate::pdf::Footer {
        reference: "PROBE-0001".to_string(),
        center: crate::report::DEFAULT_FOOTER.to_string(),
    };
    let html = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\">{}</head>\
         <body><div class=\"report\"><h1>Export probe</h1>\
         <p>PROBE-MARKER</p></div></body></html>",
        crate::report::REPORT_CSS
    );
    render_pdf(app, &html, target, &footer)?;
    check_pdf(target)?;

    // The footer must be on the page, and the page must be a real one.
    let bytes = std::fs::read(target)
        .map_err(|error| format!("The probe PDF could not be read: {error}"))?;
    if !bytes.windows(9).any(|window| window == b"LPCFooter") {
        return Err("The probe PDF has no footer drawn on it.".to_string());
    }
    let document = lopdf::Document::load(target)
        .map_err(|error| format!("The probe PDF could not be parsed: {error}"))?;
    if document.get_pages().is_empty() {
        return Err("The probe PDF has no pages.".to_string());
    }
    Ok(())
}

/// Open a path or URL with the platform's own opener.
pub fn open_with_desktop(target: &str) -> Result<(), String> {
    // The desktop opener differs per platform: `xdg-open` on Linux, `start` on
    // Windows. `start` needs an explicit window-title argument (the empty
    // string) or it treats the first quoted token of the target as the title.
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = std::process::Command::new("cmd");
        command.args(["/C", "start", "", target]);
        command
    };
    #[cfg(not(target_os = "windows"))]
    let mut command = {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(target);
        command
    };
    outside_the_appimage(&mut command)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("No application could open that: {error}"))
}

/// Open a saved report in the desktop's own browser.
#[tauri::command]
pub fn open_report_file(path: String) -> Result<(), String> {
    open_with_desktop(&path)
}

#[allow(dead_code)]
fn ensure_nonuniform_variant_marker(result: &NonuniformResult) -> &'static str {
    match result {
        NonuniformResult::Asym(_) => "asym",
        NonuniformResult::Tandem(_) => "tandem",
    }
}

#[cfg(test)]
mod pdf_export_tests {
    use super::*;

    #[test]
    fn file_urls_escape_spaces_and_use_forward_slashes() {
        assert_eq!(
            file_url(Path::new("/tmp/a b.html")),
            "file:///tmp/a%20b.html"
        );
        assert_eq!(
            file_url(Path::new("C:\\Users\\A B\\report.html")),
            "file:///C:/Users/A%20B/report.html"
        );
    }

    #[test]
    fn an_external_program_is_started_outside_the_appimage() {
        let mut command = std::process::Command::new("true");
        command.env("LD_LIBRARY_PATH", "/tmp/appimage/usr/lib");
        command.env_remove("APPDIR");
        outside_the_appimage(&mut command);
        let removed: Vec<String> = command
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .map(|(key, _)| key.to_string_lossy().to_string())
            .collect();
        for key in ["LD_LIBRARY_PATH", "APPDIR", "GTK_PATH", "XDG_DATA_DIRS"] {
            assert!(removed.contains(&key.to_string()), "kept {key}");
        }
    }

    #[test]
    fn a_finished_pdf_replaces_the_chosen_file() {
        let dir = std::env::temp_dir().join(format!("lpc-staging-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("plan.pdf");
        let staging = staged_path(&target);
        // The staging file is a sibling, so the move is a rename, not a copy
        // across filesystems — and it is hidden and unique to the process.
        assert_eq!(staging.parent(), target.parent());
        assert!(staging
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with(&format!(".plan.pdf.part-{}", std::process::id())));

        std::fs::write(&target, b"previous plan").unwrap();
        std::fs::write(&staging, b"new plan").unwrap();
        move_into_place(&staging, &target).unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"new plan");
        assert!(!staging.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_export_leaves_the_chosen_file_alone() {
        // The footer is written after printing; if that step fails the operator
        // keeps the plan they had, rather than an unfootered replacement.
        let dir = std::env::temp_dir().join(format!("lpc-staging-fail-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("plan.pdf");
        std::fs::write(&target, b"previous plan").unwrap();

        let error = move_into_place(&staged_path(&target), &target)
            .expect_err("a missing staging file must not replace the target");
        assert!(error.contains("left as it was"), "{error}");
        assert_eq!(std::fs::read(&target).unwrap(), b"previous plan");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_newer_preview_supersedes_an_older_one() {
        // The generation gate is what stops a superseded request from printing
        // or storing. Only this test touches the counter, so it can drive it.
        let mine = PREVIEW_GENERATION
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        assert!(!superseded(mine));
        PREVIEW_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert!(superseded(mine), "a newer request must supersede this one");
    }

    #[test]
    fn a_report_key_changes_with_the_report_and_its_footer() {
        let footer = crate::pdf::Footer {
            reference: "LP-1".to_string(),
            center: "Credit".to_string(),
        };
        let base = report_key("<p>plan</p>", &footer);
        assert_eq!(base, report_key("<p>plan</p>", &footer));

        let other_footer = crate::pdf::Footer {
            reference: "LP-2".to_string(),
            center: "Credit".to_string(),
        };
        assert_ne!(base, report_key("<p>plan</p>", &other_footer));
        assert_ne!(base, report_key("<p>plan 2</p>", &footer));
        // A credit-line edit is a different printed footer, so a different key.
        let other_credit = crate::pdf::Footer {
            reference: "LP-1".to_string(),
            center: "Someone else".to_string(),
        };
        assert_ne!(base, report_key("<p>plan</p>", &other_credit));
    }

    #[test]
    fn finishing_a_pdf_from_memory_replaces_the_file_in_one_step() {
        let dir = std::env::temp_dir().join(format!("lpc-bytes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("plan.pdf");
        std::fs::write(&target, b"old").unwrap();

        write_bytes(b"%PDF-1.7 preview", &target).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"%PDF-1.7 preview");
        // Nothing is left beside it.
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".part-"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_pdf_footer_matches_the_report_defaults() {
        // The PDF export writes the footer onto the pages on Linux, so it has
        // to resolve exactly as the report's own margin boxes do.
        let (reference, center) = crate::report::footer_texts("", None);
        assert_eq!(reference, "No reference");
        assert_eq!(center, crate::report::DEFAULT_FOOTER);

        let (reference, center) =
            crate::report::footer_texts("  LP-2026-014  ", Some("Checked by A. Rigger"));
        assert_eq!(reference, "LP-2026-014");
        assert_eq!(center, "Checked by A. Rigger");
    }
}
