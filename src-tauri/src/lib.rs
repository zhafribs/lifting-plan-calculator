//! Lifting Plan Calculator — desktop entry point.
//!
//! The calculation engine lives in the sibling modules (ported from the Android
//! Kotlin source); this file only wires it to the Tauri shell.

pub mod calc;
pub mod commands;
pub mod crane;
pub mod dto;
pub mod excel;
pub mod figures;
pub mod model;
pub mod nonuniform;
pub mod pdf;
#[cfg(target_os = "linux")]
pub mod pdf_print;
#[cfg(target_os = "windows")]
pub mod pdf_windows;
pub mod report;
pub mod report_capacity;
pub mod report_nonuniform;
pub mod report_uniform;
pub mod sling;
pub mod update;

pub fn run() {
    tauri::Builder::default()
        .manage(commands::ChartHolder(std::sync::Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::catalog,
            commands::log_ui_event,
            commands::solve_all,
            commands::pick_chart_file,
            commands::import_chart,
            commands::chart_update,
            commands::build_report,
            commands::save_report,
            commands::save_pdf,
            commands::preview_pdf,
            commands::open_report_file,
            update::check_update,
            update::install_update,
            update::open_external,
            update::quit_app,
        ])
        .setup(|app| {
            // A QA hook: export a probe report and exit with the result, so a CI
            // job can prove the PDF path on a platform instead of only compiling
            // it. Nothing in normal use passes this argument.
            let probe = std::env::args()
                .find_map(|argument| argument.strip_prefix("--export-pdf-test=").map(str::to_string));
            if let Some(path) = probe {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    let code = match commands::export_probe(&handle, std::path::Path::new(&path)) {
                        Ok(()) => {
                            eprintln!("export probe: ok");
                            0
                        }
                        Err(error) => {
                            eprintln!("export probe: failed: {error}");
                            1
                        }
                    };
                    handle.exit(code);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Lifting Plan Calculator");
}
