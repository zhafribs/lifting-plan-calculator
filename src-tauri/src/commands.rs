//! The Tauri command surface: everything the UI can ask the engine to do.

use std::path::Path;
use std::sync::Mutex;
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

/// A PDF renderer: Chromium on Linux, Edge or Chrome on Windows. Both are
/// headless Chromium builds, so the same flags and the report's own
/// `@page { margin: 25.4mm }` produce the 1-inch margin.
fn pdf_browser_candidates() -> Vec<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::new();
        for variable in ["PROGRAMFILES(X86)", "PROGRAMFILES", "LOCALAPPDATA"] {
            if let Ok(base) = std::env::var(variable) {
                let base = std::path::Path::new(&base);
                candidates.push(base.join("Microsoft/Edge/Application/msedge.exe"));
                candidates.push(base.join("Google/Chrome/Application/chrome.exe"));
            }
        }
        candidates
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Absolute paths first: an AppImage launched from a desktop does not
        // always carry the shell's PATH, and a bare name is resolved through it.
        let mut candidates = Vec::new();
        for name in [
            "chromium",
            "chromium-browser",
            "google-chrome",
            "google-chrome-stable",
            "brave-browser",
        ] {
            candidates.push(std::path::PathBuf::from("/usr/bin").join(name));
            candidates.push(std::path::PathBuf::from("/usr/local/bin").join(name));
        }
        candidates.push(std::path::PathBuf::from("/snap/bin/chromium"));
        for name in [
            "chromium",
            "chromium-browser",
            "google-chrome",
            "google-chrome-stable",
            "brave-browser",
        ] {
            candidates.push(std::path::PathBuf::from(name));
        }
        candidates
    }
}

fn find_pdf_browser(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    // The renderer the bundle carries comes first: an AppImage has to work on a
    // machine with no browser installed.
    if let Some(bundled) = bundled_pdf_browser(app) {
        return Some(bundled);
    }
    pdf_browser_candidates().into_iter().find(|candidate| {
        if candidate.components().count() > 1 {
            candidate.is_file()
        } else {
            outside_the_appimage(&mut std::process::Command::new(candidate))
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    })
}

/// The headless Chromium shipped inside the bundle, when this build carries one.
fn bundled_pdf_browser(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    use tauri::Manager;
    let relative = "pdf-renderer/chrome-headless-shell-linux64/chrome-headless-shell";
    app.path()
        .resolve(relative, tauri::path::BaseDirectory::Resource)
        .ok()
        .filter(|path| path.is_file())
        .filter(|path| {
            outside_the_appimage(&mut std::process::Command::new(path))
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        })
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

fn render_pdf(app: &tauri::AppHandle, html: &str, target: &Path) -> Result<(), String> {
    let browser = find_pdf_browser(app).ok_or_else(|| {
        "No PDF renderer was found. This build carries none and the system has no \
         Chromium; save the HTML and print it with the margins set to Default."
            .to_string()
    })?;

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let temp = std::env::temp_dir().join(format!(
        "lifting-plan-pdf-{}-{stamp}",
        std::process::id()
    ));
    std::fs::create_dir_all(&temp)
        .map_err(|error| format!("The PDF workspace could not be created: {error}"))?;
    let html_path = temp.join("report.html");
    std::fs::write(&html_path, html)
        .map_err(|error| format!("The PDF report could not be prepared: {error}"))?;
    write_katex_assets(app, &temp)?;
    let profile = temp.join("profile");
    std::fs::create_dir_all(&profile)
        .map_err(|error| format!("The PDF workspace could not be created: {error}"))?;

    let mut child = outside_the_appimage(&mut std::process::Command::new(&browser))
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--no-pdf-header-footer")
        .arg(format!("--user-data-dir={}", profile.display()))
        .arg(format!("--print-to-pdf={}", target.display()))
        .arg(file_url(&html_path))
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
                let _ = std::fs::remove_dir_all(&temp);
                let _ = std::fs::remove_file(target);
                return Err("The PDF renderer did not finish within 90 seconds.".to_string());
            }
            Err(error) => {
                let _ = std::fs::remove_dir_all(&temp);
                let _ = std::fs::remove_file(target);
                return Err(format!("The PDF renderer could not be followed: {error}"));
            }
        }
    };
    let _ = std::fs::remove_dir_all(&temp);

    if !status.success() {
        let _ = std::fs::remove_file(target);
        return Err(format!(
            "The PDF renderer exited with {status}. The report was not saved."
        ));
    }
    let size = std::fs::metadata(target)
        .map_err(|error| format!("The PDF was not written: {error}"))?
        .len();
    if size == 0 {
        let _ = std::fs::remove_file(target);
        return Err("The PDF renderer wrote an empty file.".to_string());
    }
    Ok(())
}

/// Write the report as a PDF with the report's own 1-inch page margin, using a
/// headless Chromium build rather than the system print dialog.
#[tauri::command]
pub async fn save_pdf(
    app: tauri::AppHandle,
    html: String,
    suggested_name: String,
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

    let target_for_render = target.clone();
    tauri::async_runtime::spawn_blocking(move || render_pdf(&app, &html, &target_for_render))
        .await
        .map_err(|error| format!("The PDF export failed: {error}"))??;
    Ok(Some(target.to_string_lossy().to_string()))
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
    fn a_pdf_browser_candidate_list_is_always_present() {
        assert!(!pdf_browser_candidates().is_empty());
    }
}
