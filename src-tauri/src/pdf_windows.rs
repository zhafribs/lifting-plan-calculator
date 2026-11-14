//! Windows: print the report with the WebView2 the app already runs on.
//!
//! The app cannot start on Windows without the WebView2 runtime, so using it to
//! print means *Save as PDF* works even when the Microsoft Edge browser has been
//! removed or disabled — the case that used to fail. Edge remains the fallback
//! for a runtime too old to expose `PrintToPdf`, and for the print failing.
//!
//! WebView2 is Chromium, so the report's own `@page` margin boxes draw the
//! footer: nothing is stamped here, unlike Linux.

use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use tauri::webview::PageLoadEvent;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_7, ICoreWebView2Environment6,
};
use webview2_com::PrintToPdfCompletedHandler;
use windows_core::{Interface, PCWSTR};

/// How long the report may take to load before the window is given up on.
const LOAD_TIMEOUT: Duration = Duration::from_secs(20);
/// How long the print may take once the page is up.
const PRINT_TIMEOUT: Duration = Duration::from_secs(60);

/// Print `html_path` to `target` with the app's own WebView2.
pub fn print_to_pdf(
    app: &tauri::AppHandle,
    html_path: &Path,
    target: &Path,
) -> Result<(), String> {
    let url = tauri::Url::from_file_path(html_path)
        .map_err(|_| "The report file could not be opened for printing.".to_string())?;
    // A distinct label per export: two saves must not collide in the manager.
    let label = format!(
        "pdf-print-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    );

    let (loaded_tx, loaded_rx) = mpsc::channel();
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        // Off-screen rather than hidden: a hidden WebView2 may stop painting,
        // and a page that never paints prints blank.
        .position(-32_000.0, -32_000.0)
        .inner_size(1060.0, 1500.0)
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .focused(false)
        .on_page_load(move |_window, payload| {
            if payload.event() == PageLoadEvent::Finished {
                let _ = loaded_tx.send(());
            }
        })
        .build()
        .map_err(|error| format!("The print view could not be opened: {error}"))?;

    let result = (|| -> Result<(), String> {
        loaded_rx
            .recv_timeout(LOAD_TIMEOUT)
            .map_err(|_| "The report did not finish loading for printing.".to_string())?;

        let (done_tx, done_rx) = mpsc::channel();
        let target = target.to_path_buf();
        window
            .with_webview(move |platform| {
                let controller = platform.controller();
                let environment = platform.environment();
                // The closure cannot return a value, so every failure is sent
                // through the same channel the success uses.
                let outcome = print(&controller, &environment, &target, done_tx.clone());
                if let Err(error) = outcome {
                    let _ = done_tx.send(Err(error));
                }
            })
            .map_err(|error| format!("The print could not be started: {error}"))?;

        done_rx
            .recv_timeout(PRINT_TIMEOUT)
            .map_err(|_| "The PDF did not finish within 60 seconds.".to_string())?
    })();

    let _ = window.close();
    result
}

/// Hand the page to WebView2's own PDF printer.
fn print(
    controller: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller,
    environment: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
    target: &Path,
    done: Sender<Result<(), String>>,
) -> Result<(), String> {
    // SAFETY: every call is on the WebView2 UI thread, inside `with_webview`,
    // and each interface is the same runtime object the app is running on.
    unsafe {
        let core = controller
            .CoreWebView2()
            .map_err(|error| format!("The print view is not ready: {error}"))?;
        let core: ICoreWebView2_7 = core.cast().map_err(|_| {
            "This WebView2 runtime is too old to print. Update the WebView2 Runtime."
                .to_string()
        })?;
        let environment: ICoreWebView2Environment6 = environment.cast().map_err(|_| {
            "This WebView2 runtime is too old to print. Update the WebView2 Runtime."
                .to_string()
        })?;
        let settings = environment
            .CreatePrintSettings()
            .map_err(|error| format!("The print settings could not be created: {error}"))?;
        // A4 with the report's one-inch margin, in inches.
        settings.SetPageWidth(8.27).ok();
        settings.SetPageHeight(11.69).ok();
        settings.SetMarginTop(1.0).ok();
        settings.SetMarginBottom(1.0).ok();
        settings.SetMarginLeft(1.0).ok();
        settings.SetMarginRight(1.0).ok();
        settings.SetShouldPrintBackgrounds(true.into()).ok();
        // The footer comes from the report's `@page` margin boxes, not the
        // browser's own header and footer.
        settings.SetShouldPrintHeaderAndFooter(false.into()).ok();

        let wide: Vec<u16> = target
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let path = PCWSTR::from_raw(wide.as_ptr());
        let handler = PrintToPdfCompletedHandler::create(Box::new(move |result, finished| {
            let _ = done.send(if finished && result.is_ok() {
                Ok(())
            } else {
                Err(format!("The PDF was not written: {result:?}"))
            });
            Ok(())
        }));
        core.PrintToPdf(path, &settings, &handler)
            .map_err(|error| format!("The PDF could not be printed: {error}"))?;
    }
    Ok(())
}
