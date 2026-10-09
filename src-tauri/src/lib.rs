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
pub mod report;
pub mod report_capacity;
pub mod report_nonuniform;
pub mod report_uniform;
pub mod sling;

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
            commands::open_report_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Lifting Plan Calculator");
}
