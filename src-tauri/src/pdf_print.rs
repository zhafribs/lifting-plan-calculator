//! Printing the plan with the app's own web view.
//!
//! The report is printed by the WebKitGTK view the app already runs on, straight
//! to a PDF file and without a dialog, so the AppImage does not have to carry a
//! 100 MB browser and needs nothing installed on the machine. The footer is
//! drawn onto the finished PDF afterwards — see [`crate::pdf`] — because
//! WebKitGTK does not render CSS `@page` margin boxes.
//!
//! Three details matter here. WebKit prints asynchronously, so the caller waits
//! on a channel and the GTK main loop drives the print; running a nested main
//! loop inside a Tauri command would fight the app's own event loop. Printing
//! starts on `load-finished`, which is after the KaTeX auto-render script at the
//! end of the report has run, so the equations are typeset by then. And the
//! print objects are owned by a registry rather than by their own signal
//! handlers, which would otherwise keep every export's web view alive.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use webkit2gtk::{LoadEvent, PrintOperation, PrintOperationExt, WebView, WebViewExt};

/// How long the print may take. A seven-page report prints in under a second;
/// the allowance covers a long plan and a slow machine. It is deliberately short
/// because a desktop with no print backend never reports anything at all, and
/// the release job asserts the AppImage carries GTK's file backend.
const TIMEOUT: Duration = Duration::from_secs(20);

/// Print `uri` (the report's `file://` URL) to `target`, with A4 pages and the
/// report's own one-inch margin. Returns once the file has been written.
pub fn print_to_pdf(
    app: &tauri::AppHandle,
    uri: String,
    target: std::path::PathBuf,
) -> Result<(), String> {
    let (sender, receiver) = mpsc::channel();
    // The web view has to be built and printed from the GTK main thread; the
    // caller is a blocking task, so that thread is free to drive the print.
    app.run_on_main_thread(move || start(&uri, &target, sender))
        .map_err(|error| format!("The report could not be handed to the printer: {error}"))?;
    match receiver.recv_timeout(TIMEOUT) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => {
            // Nothing reported back, so the job is abandoned: drop its view on
            // the main thread rather than hold it for the life of the process.
            let _ = app.run_on_main_thread(cancel_all);
            Err("The PDF did not finish within 20 seconds. If this system has no \
                 print backend, save the plan as HTML and print it from a browser instead."
                .to_string())
        }
        Err(RecvTimeoutError::Disconnected) => {
            Err("The report could not be printed: the print stopped before the PDF was written."
                .to_string())
        }
    }
}

/// Start the print on the calling thread, which must be the GTK main thread,
/// and return the channel it reports through. The app's command path goes
/// through [`print_to_pdf`]; this seam is what the `export_pdf` example and the
/// integration test drive with their own main loop.
pub fn begin_print(uri: &str, target: &Path) -> Receiver<Result<(), String>> {
    let (sender, receiver) = mpsc::channel();
    start(uri, target, sender);
    receiver
}

/// The view and print operation, kept alive until the print finishes.
struct Job {
    id: u64,
    view: WebView,
    operation: PrintOperation,
    sender: RefCell<Option<Sender<Result<(), String>>>>,
}

thread_local! {
    /// Owns every print in flight. Dropping these from inside their own signal
    /// handlers would be re-entrant, so removal is deferred to an idle callback.
    static JOBS: RefCell<Vec<Rc<Job>>> = const { RefCell::new(Vec::new()) };
}

fn start(uri: &str, target: &Path, sender: Sender<Result<(), String>>) {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    let view = WebView::new();
    let operation = PrintOperation::new(&view);

    let setup = gtk::PageSetup::new();
    setup.set_paper_size(&gtk::PaperSize::new(Some("iso_a4")));
    for margin in [
        gtk::PageSetup::set_top_margin,
        gtk::PageSetup::set_bottom_margin,
        gtk::PageSetup::set_left_margin,
        gtk::PageSetup::set_right_margin,
    ] {
        margin(&setup, crate::pdf::MARGIN_PT as f64 / 72.0 * 25.4, gtk::Unit::Mm);
    }
    operation.set_page_setup(&setup);

    let settings = gtk::PrintSettings::new();
    // "Print to File" is GTK's own file backend: with an output URI it writes
    // the PDF without ever opening the print dialog.
    settings.set_printer("Print to File");
    settings.set("output-file-format", Some("pdf"));
    let output = gtk::glib::filename_to_uri(target, None)
        .map(|uri| uri.to_string())
        .unwrap_or_else(|_| String::from("file:///dev/null"));
    settings.set("output-uri", Some(&output));
    operation.set_print_settings(&settings);

    let job = Rc::new(Job {
        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        view,
        operation,
        sender: RefCell::new(Some(sender)),
    });
    JOBS.with(|jobs| jobs.borrow_mut().push(Rc::clone(&job)));

    let finished = Rc::downgrade(&job);
    job.operation.connect_finished(move |_| {
        if let Some(job) = finished.upgrade() {
            finish(&job, Ok(()));
        }
    });
    let failed = Rc::downgrade(&job);
    job.operation.connect_failed(move |_, error| {
        if let Some(job) = failed.upgrade() {
            finish(&job, Err(format!("The PDF renderer failed: {error}")));
        }
    });

    let loaded = Rc::downgrade(&job);
    job.view.connect_load_changed(move |_, event| {
        if event == LoadEvent::Finished {
            if let Some(job) = loaded.upgrade() {
                job.operation.print();
            }
        }
    });
    let load_failed = Rc::downgrade(&job);
    job.view.connect_load_failed(move |_, _, _, error| {
        if let Some(job) = load_failed.upgrade() {
            finish(
                &job,
                Err(format!(
                    "The report could not be prepared for printing: {error}"
                )),
            );
        }
        false
    });

    job.view.load_uri(uri);
}

/// Drop every print in flight. Only used when a print stopped reporting, so
/// its view would otherwise be kept for the life of the process.
fn cancel_all() {
    JOBS.with(|jobs| jobs.borrow_mut().clear());
}

fn finish(job: &Rc<Job>, result: Result<(), String>) {
    if let Some(sender) = job.sender.borrow_mut().take() {
        let _ = sender.send(result);
    }
    let id = job.id;
    // The view and print operation are dropped here, out of their own signal
    // handlers, so an export does not leak a web view per document.
    gtk::glib::idle_add_local_once(move || {
        JOBS.with(|jobs| jobs.borrow_mut().retain(|job| job.id != id));
    });
}
