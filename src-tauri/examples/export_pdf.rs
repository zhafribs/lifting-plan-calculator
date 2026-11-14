//! Dev helper: print a report the way *Save as PDF* does, without the GUI.
//!
//! Run with a session (it needs a display), or under `xvfb-run` on the CI job
//! that proves the export works:
//!
//! ```text
//! cargo run --example export_pdf -- /tmp/opencode/fixtures/report.html /tmp/opencode/plan.pdf
//! ```
//!
//! This is the real export path: the app's own WebKit view prints the report to
//! A4, then the footer is written onto the finished pages. Layout assertions
//! live in the PDF tests; this example exists so the print step itself can be
//! run and inspected.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("export_pdf is a Linux helper: Windows exports with Edge.");
    std::process::exit(2);
}

#[cfg(target_os = "linux")]
fn main() {
    use std::path::PathBuf;

    use gtk::glib;
    use lifting_plan_calculator_lib::pdf::{stamp, Footer};
    use lifting_plan_calculator_lib::pdf_print::begin_print;

    let mut args = std::env::args().skip(1);
    let Some(html) = args.next() else {
        eprintln!("usage: export_pdf <report.html> <out.pdf> [reference] [credit]");
        std::process::exit(2);
    };
    let out = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/opencode/plan.pdf"));
    let reference = args.next().unwrap_or_else(|| "No reference".to_string());
    let center = args
        .next()
        .unwrap_or_else(|| lifting_plan_calculator_lib::report::DEFAULT_FOOTER.to_string());

    // The report resolves `vendor/katex/…` beside itself, exactly as the app's
    // export workspace does.
    let workspace = std::env::temp_dir().join("lpc-export-pdf");
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(workspace.join("vendor")).unwrap();
    std::fs::copy(&html, workspace.join("report.html")).unwrap();
    copy_tree(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/vendor/katex"),
        &workspace.join("vendor/katex"),
    )
    .unwrap();

    gtk::init().unwrap();
    let url = glib::filename_to_uri(workspace.join("report.html"), None).unwrap();
    let receiver = begin_print(url.as_ref(), &out);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let result = loop {
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
        match receiver.try_recv() {
            Ok(result) => break result,
            Err(std::sync::mpsc::TryRecvError::Empty) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(error) => break Err(format!("the print did not report back: {error}")),
        }
    };
    if let Err(error) = result {
        eprintln!("print failed: {error}");
        std::process::exit(1);
    }

    // `EXPORT_PDF_RAW=1` stops after the print, which is how the committed
    // WebKit fixture was captured (the stamping test needs an unstamped PDF).
    if std::env::var_os("EXPORT_PDF_RAW").is_some() {
        println!("wrote {}", out.display());
        return;
    }
    if let Err(error) = stamp(&out, &Footer { reference, center }) {
        eprintln!("footer failed: {error}");
        std::process::exit(1);
    }
    println!("wrote {}", out.display());
}

#[cfg(target_os = "linux")]
fn copy_tree(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
