//! The HTML report builder, ported from `Report.kt`.
//!
//! The report is deliberately independent of the UI: [`build_report_html`]
//! takes the solved state and returns a self-contained body that a webview, a
//! browser, or a test can render. The structure follows the numbered sections
//! in the reference documents, while keeping the formulas readable when KaTeX
//! is unavailable.

use crate::calc::{crane_usage_band, UsageState};
use crate::crane::{CraneResult, DESIGN_LOAD_RATIO};
use crate::dto::PlanSignoff;
use crate::model::LiftTotals;
use crate::nonuniform::{NonuniformResult, TandemResult};
use crate::report_capacity::{capacity_section, notes_section, shows_notes, factors_section};
use crate::report_nonuniform::{nonuniform_sections};
use crate::report_uniform::{geometry_section, section_inputs};
use crate::sling::{hitch_spec, SlingResult};

/// The centre text of every printed page's footer, unless the report screen
/// supplies its own.
pub const DEFAULT_FOOTER: &str =
    "Prepared with Lifting Plan Calculator by Zhafri Syazwi — zhafribs@gmail.com";

pub const REPORT_CSS: &str = r#"
<style>
@page {
    @bottom-left { content: var(--plan-reference, "No reference"); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
    @bottom-center { content: var(--footer-center, "Prepared with Lifting Plan Calculator by Zhafri Syazwi — zhafribs@gmail.com"); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
    @bottom-right { content: counter(page) " / " counter(pages); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
}
.report {
    color: #151515;
    background: #fff;
    font-family: "Liberation Serif", "DejaVu Serif", serif;
    font-size: 7pt;
    line-height: 1.2;
}
.report h1 { font-size: 9pt; margin: 0 0 4pt; color: #111; text-align: center; }
.report h2 { font-size: 6pt; margin: 11pt 0 3pt; padding: 2pt 4pt;
    background: #e8edf3; border-bottom: 1px solid #9aa9ba; }
.report h3 { font-size: 6pt; margin: 7pt 0 1pt; }
.report p { margin: 1.5pt 0; }
.report .meta { color: #555; font-size: 5.5pt; margin-bottom: 5pt; }
.report table { border-collapse: collapse; width: 100%; margin: 3pt 0 6pt; }
.report table.kv { table-layout: fixed; }
.report table.kv th { width: 38%; }
.report table.kv td { width: 62%; }
.report table.kv.light { border-collapse: collapse; margin: 2pt 0 5pt; }
.report table.kv.light th { border: 0; border-bottom: 0.5pt solid #e4eaf1;
    background: none; padding: 2.5pt 8pt 2.5pt 0; font-size: 5.5pt;
    font-weight: normal; letter-spacing: 0.3pt; text-transform: uppercase;
    color: #6d7a88; vertical-align: top; }
.report table.kv.light td { border: 0; border-bottom: 0.5pt solid #e4eaf1;
    padding: 2.5pt 0; font-size: 7.5pt; color: #1b2733; vertical-align: top; }
.report table.kv.light tr:last-child th,
.report table.kv.light tr:last-child td { border-bottom: 0; }
.report td, .report th { border: 1px solid #7c8998; padding: 1pt 2pt;
    vertical-align: top; font-size: 5.5pt; }
.report th { background: #f1f4f7; text-align: left; font-weight: bold; }
.report .equation { margin: 2pt 0 3pt 12pt; font-family: "DejaVu Serif", serif;
    font-size: 5pt; }
.report .equation .fallback { font-family: "Liberation Serif", serif; }
.report .note { border: 1px solid #9aa9ba; background: #f7f7f7; padding: 5pt 7pt;
    margin-top: 5pt; }
.report .critical { border: 2px solid #a33; padding: 5pt 7pt; margin: 5pt 0; }
.report .pass { color: #176b3a; font-weight: bold; }
.report .fail { color: #9b1c1c; font-weight: bold; }
.report .small { font-size: 7pt; } .report .note { font-size: 7pt; } .report .status { font-size: 7pt; } .report .meta { font-size: 7pt; } .report p { font-size: 7pt; } .report td, .report th { font-size: 7pt; } .report .equation { font-size: 6.5pt; } .report h1 { font-size: 12pt; } .report h2 { font-size: 9pt; } .report h3 { font-size: 8pt; }
/* The plan's head: a light card that reads top-down — what the plan is, the
   figures that decide it, then the verdict. No heavy grid; the values carry
   the weight and the labels stay quiet. */
.report { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
.report .plan-head { border: 1px solid #d9e1e9; border-radius: 5pt;
    background: #fbfcfe; padding: 5pt 9pt 5pt; margin: 0 0 6pt; }
.report .plan-title-block { margin: 0 0 5pt; }
.report .plan-title-block h1 { font-size: 16pt; line-height: 1; margin: 0;
    text-align: left; letter-spacing: -0.3pt; color: #101820; }
.report .plan-crane { font-size: 10pt; font-weight: bold; color: #1b2733; }
.report .plan-details { margin-top: 4pt; }
.report .plan-head h2 { margin: 3pt 0 2pt; }
.report table.plan-meta { width: 100%; border-collapse: collapse;
    table-layout: fixed; margin: 0; }
.report table.plan-meta col.ref { width: 30%; }
.report table.plan-meta col.issued { width: 22%; }
.report table.plan-meta col.pb,
.report table.plan-meta col.cb,
.report table.plan-meta col.ab { width: 16%; }
.report table.plan-meta td { border: 0; padding: 0 5pt 0 0; vertical-align: top; }
.report .plan-meta .k { display: block; font-size: 5.5pt; letter-spacing: 0.3pt;
    text-transform: uppercase; color: #6d7a88; }
.report .plan-meta .v { display: block; font-size: 7.5pt; color: #1b2733;
    border-bottom: 0.5pt solid #ccd6e0; padding: 1pt 0 1.5pt; min-height: 8pt; }
.report table.metrics { width: 100%; border-collapse: separate;
    border-spacing: 3pt 2pt; margin: 2pt -3pt 0; table-layout: fixed; }
.report table.metrics td { border: 0; padding: 3pt 5pt; background: #f2f6fa;
    border-radius: 3pt; vertical-align: top; }
.report .input-card { border: 1px solid #d9e1e9; border-radius: 5pt;
    background: #fbfcfe; padding: 5pt 9pt 6pt; margin: 0 0 6pt; }
.report .input-card h2 { margin: 1pt 0 4pt; }
.report .section-card { border: 1px solid #d9e1e9; border-radius: 5pt;
    background: #fbfcfe; padding: 5pt 9pt 6pt; margin: 0 0 6pt; }
.report .section-card h2 { margin: 1pt 0 4pt; }
.report table.input-grid { width: 100%; border-collapse: separate;
    border-spacing: 3pt 2pt; margin: 0 -3pt; table-layout: fixed; }
.report table.input-grid td { border: 0; padding: 3.5pt 5pt; background: #f2f6fa;
    border-radius: 3pt; vertical-align: top; }
.report table.input-grid .input-v { font-size: 8pt; }
.report .working-wrap { padding: 0; margin-top: 6pt; }
.report table.working { width: 100%; border-collapse: collapse; margin: 4pt 0 4pt;
    table-layout: fixed; }
.report table.working td { border: 0; padding: 0; vertical-align: top; }
.report table.working td:first-child { width: 42%; padding-right: 8pt; }
.report table.working td:last-child { width: 58%; padding-left: 8pt;
    border-left: 0.5pt solid #e4eaf1; }
.report .work-label { font-size: 7pt; font-weight: bold; color: #1b2733;
    text-align: center; padding-bottom: 3pt; margin-bottom: 5pt;
    border-bottom: 0.5pt solid #d9e1e9; }
.report table.working .equation { margin: 0; font-size: 6pt; text-align: center; }
.report table.working .equation .katex-display { text-align: center; margin: 0; }
.report table.working .equation .katex-display > .katex { text-align: center; }
.report .work-empty { font-size: 6pt; color: #6d7a88; }
.report .working-diagram { margin-top: 6pt; text-align: center; }
.report .diagram-title { font-size: 6pt; font-weight: bold; letter-spacing: 0.3pt;
    text-transform: uppercase; color: #6d7a88; margin-bottom: 2pt; }
.report .working-diagram img { max-width: 100%; max-height: 78mm;
    border: 0.5pt solid #d9e1e9; border-radius: 3pt; background: #fff;
    filter: invert(1) hue-rotate(180deg) brightness(1.08); }
.report .metric-k { display: block; font-size: 5.5pt; letter-spacing: 0.3pt;
    text-transform: uppercase; color: #6d7a88; }
.report .metric-v { display: block; font-size: 10pt; font-weight: bold;
    line-height: 1.1; color: #0f1720; margin-top: 0.5pt; }
.report .metric-line { display: flex; align-items: baseline; gap: 4pt; }
.report .metric-line .metric-v { margin: 0; }
.report .metric-line .metric-band { margin: 0; }
.report .metric-n { display: block; font-size: 5.5pt; color: #5d6b7a;
    margin-top: 1pt; line-height: 1.2; }
.report .metric-band { display: block; font-size: 8pt; font-weight: bold;
    letter-spacing: 0.3pt; line-height: 1.1; margin-top: 1pt; }
.report .metric-sub { display: block; font-size: 6pt; color: #5d6b7a;
    line-height: 1.2; }
.report .metric-band.within { color: #176b3a; }
.report .metric-band.caution { color: #8a5517; }
.report .metric-band.over { color: #9b1c1c; }
.report .metric-band.neutral { color: #4a5560; }
</style>
"#;

/// The page wrapper: assemble the report into a page that renders its own
/// equations. Ported from `ReportPage.kt`.
///
/// **The KaTeX distribution is bundled rather than fetched**, so the page
/// renders with no network and no remote content: a lift plan read on site must
/// not change because a CDN did.
pub const PRINT_MARGIN_MM: f64 = 25.4;

/// The auto-render pass, as a string so the backslashes reach JavaScript intact.
const KATEX_AUTO_RENDER: &str = r#"
document.addEventListener("DOMContentLoaded", function () {
  try { renderMathInElement(document.body, {
    delimiters: [
      {left: "\\(", right: "\\)", display: false},
      {left: "\\[", right: "\\]", display: true}
    ], throwOnError: false, strict: false
  }); } catch (e) {
    document.body.setAttribute("data-katex-error", e.message);
  }
});
"#;

/// `body` as a complete page that typesets its own equations.
pub fn wrap_katex(body: &str) -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html><html><head><meta charset=\"utf-8\">");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    out.push_str("<style>");
    out.push_str(&format!(
        "@page {{ size: A4; margin: {}mm; }} ",
        PRINT_MARGIN_MM
    ));
    out.push_str("html, body { background: #fff; margin: 0; padding: 0; } ");
    out.push_str("body { background-color: #fff; color: #111; font-size: 7pt; line-height: 1.2; } ");
    out.push_str("table { font-size: inherit; } ");
    out.push_str(".katex-display { text-align: left; margin: 3px 0; } ");
    out.push_str("</style>");
    out.push_str("<link rel=\"stylesheet\" href=\"vendor/katex/katex.min.css\">");
    out.push_str("<script src=\"vendor/katex/katex.min.js\"></script>");
    out.push_str("<script src=\"vendor/katex/auto-render.min.js\"></script>");
    out.push_str("</head><body>");
    out.push_str(body);
    out.push_str("<script>");
    out.push_str(KATEX_AUTO_RENDER);
    out.push_str("</script></body></html>");
    out
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

pub fn number_or(value: Option<f64>, default: f64) -> f64 {
    match value {
        Some(value) if value.is_finite() => value,
        _ => default,
    }
}

/// A figure with a thousands separator, or an em dash for "nothing to show".
///
/// A null is *not* zero. The report prints "—" for an absent figure and "0.00"
/// for a measured zero, and the two mean different things on a lift plan.
pub fn fmt(value: Option<f64>, places: usize, unit: &str) -> String {
    match value {
        None => "\u{2014}".to_string(),
        Some(value) => {
            let text = crate::figures::grouped(number_or(Some(value), 0.0), places);
            if unit.is_empty() {
                text
            } else {
                format!("{text} {unit}")
            }
        }
    }
}

pub fn plain(value: Option<f64>, places: usize) -> String {
    fmt(value, places, "")
}

/// Number formatting that survives being read as TeX: thousands separators are
/// braced.
pub fn tex_num(value: Option<f64>, places: usize) -> String {
    fmt(value, places, "").replace(',', "{,}")
}

pub fn tex_deg(value: Option<f64>, places: usize) -> String {
    crate::figures::grouped(number_or(value, 0.0), places) + "^\\circ"
}

/// One KaTeX expression, in display mode.
pub fn equation(latex: &str) -> String {
    format!("<div class=\"equation\">\\[{latex}\\]</div>")
}

pub fn heading(number: usize, title: &str) -> String {
    format!("<h2>{number}. {}</h2>", escape(&title.to_uppercase()))
}

pub fn subheading(title: &str) -> String {
    format!("<h3>{}</h3>", escape(title))
}

/// A two-column key/value table, the report's only table shape.
pub fn table(rows: &[(String, String)]) -> String {
    let mut out = String::from("<table class=\"kv\"><tbody>");
    for (key, value) in rows {
        out.push_str(&format!(
            "<tr><th>{}</th><td>{}</td></tr>",
            escape(key),
            escape(value)
        ));
    }
    out.push_str("</tbody></table>");
    out
}

/// The same rows as a three-column grid of figure cards, used for the lift
/// inputs so they read like the figures in the plan head.
pub fn card_grid(rows: &[(String, String)]) -> String {
    let mut out = String::from("<table class=\"input-grid\"><tbody>");
    for chunk in rows.chunks(3) {
        out.push_str("<tr>");
        for (key, value) in chunk {
            out.push_str(&format!(
                "<td><span class=\"metric-k\">{}</span><span class=\"metric-v input-v\">{}</span></td>",
                escape(key),
                escape(value)
            ));
        }
        for _ in chunk.len()..3 {
            out.push_str("<td></td>");
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody></table>");
    out
}

/// A paragraph whose text is already HTML.
pub fn p(text: &str, css: &str) -> String {
    if css.is_empty() {
        format!("<p>{text}</p>")
    } else {
        format!("<p style=\"{css}\">{text}</p>")
    }
}

pub fn label_value(label: &str, value: &str) -> String {
    p(
        &format!("<b>{}:</b> {}", escape(label), escape(value)),
        "",
    )
}

pub fn label_or_not_entered(value: Option<&str>) -> String {
    match value {
        None | Some("") => "Not entered".to_string(),
        Some(value) => value.to_string(),
    }
}

/// `html.escape` with quoting on.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// A value safe inside a double-quoted CSS string.
fn css_string(value: &str) -> String {
    // `<` is escaped as a CSS code point so a reference or credit line cannot
    // close the `<style>` element it is written into. The preview is an
    // unsandboxed `srcdoc` frame read by the main window, so a closing tag here
    // would be script execution, not a formatting glitch.
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('<', "\\3c ")
        .replace(['\n', '\r'], " ")
}

// ---------------------------------------------------------------------------
// The crane design load
// ---------------------------------------------------------------------------

/// Crane design load `W = gross load at hook / 0.75`.
pub fn design_load_from_gross(gross: f64) -> f64 {
    if gross <= 0.0 {
        0.0
    } else {
        gross / DESIGN_LOAD_RATIO
    }
}

// ---------------------------------------------------------------------------
// The document
// ---------------------------------------------------------------------------

/// Everything the report can draw on.
#[derive(Debug, Clone, Default)]
pub struct ReportInputs {
    pub totals: LiftTotals,
    pub crane: Option<CraneResult>,
    pub sling: Option<SlingResult>,
    pub nonuniform: Option<NonuniformResult>,
    /// Why the nonuniform lift could not be solved, or None when it could.
    pub nonuniform_error: Option<String>,
    pub tandem: Option<TandemResult>,
    /// Why the tandem lift could not be solved, or None when it could.
    pub tandem_error: Option<String>,
    /// Why the crane check could not be solved, or None when it could.
    pub crane_error: Option<String>,
    /// The plan's own reference and sign-off lines.
    pub signoff: PlanSignoff,
    /// When the plan was produced, as the client reported it.
    pub issued_at: Option<String>,
    /// The Graph tab's working-range diagram, as a PNG data URL, or None.
    pub working_diagram: Option<String>,
    /// The centre text of every printed page's footer; None uses the default.
    pub footer_center: Option<String>,
}

/// The sections a report may carry, in the builder's own source order.
pub const REPORT_SECTIONS: [&str; 4] = ["crane", "uniform", "nonuniform", "tandem"];

/// The sections one tab's report carries, in the report's own order.
pub fn report_sections_for(own: &str, include_crane: bool) -> Vec<String> {
    REPORT_SECTIONS
        .iter()
        .filter(|section| **section == own || (include_crane && **section == "crane"))
        .map(|section| section.to_string())
        .collect()
}

/// The first unused section number for a section about to be built.
pub fn next_section_number(parts: &[String]) -> usize {
    let mut highest = 0usize;
    for part in parts {
        let bytes = part.as_bytes();
        let mut i = 0;
        while i + 4 <= bytes.len() {
            if &bytes[i..i + 4] == b"<h2>" {
                let start = i + 4;
                let mut j = start;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j > start && j < bytes.len() && bytes[j] == b'.' {
                    if let Ok(value) = part[start..j].parse::<usize>() {
                        highest = highest.max(value);
                    }
                }
                i = j.max(i + 4);
            } else {
                i += 1;
            }
        }
    }
    highest + 1
}

/// One key figure: a quiet label, a value that carries the weight, an optional
/// note beneath it.
fn metric_cell(key: &str, value: &str, note: &str) -> String {
    metric_cell_html(key, value, &escape(note))
}

/// As [`metric_cell`], but the note may carry markup (the band word).
fn metric_cell_html(key: &str, value: &str, note_html: &str) -> String {
    let mut out = format!(
        "<td><span class=\"metric-k\">{}</span><span class=\"metric-v\">{}</span>",
        escape(key),
        escape(value)
    );
    if !note_html.is_empty() {
        out.push_str(&format!("<span class=\"metric-n\">{note_html}</span>"));
    }
    out.push_str("</td>");
    out
}

/// `48.00°` rather than `48.00 °` in the head's prose.
fn degrees(angle: f64) -> String {
    fmt(Some(angle), 2, "\u{00b0}").replace(" \u{00b0}", "\u{00b0}")
}

/// The six figures that decide the plan, as two rows of three cards.
fn metrics_table(crane: &CraneResult, gross: f64) -> String {
    let chart_value = crane
        .chart_capacity_kg
        .map(|capacity| fmt(Some(capacity), 2, "kg"))
        .unwrap_or_else(|| "Not evaluated".to_string());
    let usage_value = crane
        .capacity_usage_percent
        .map(|usage| format!("{:.2}%", usage))
        .unwrap_or_else(|| "\u{2014}".to_string());
    let required_value = crane
        .required_capacity_kg
        .map(|required| fmt(Some(required), 2, "kg"))
        .unwrap_or_else(|| "\u{2014}".to_string());
    let booked_value = if crane.crane_capacity_kg != 0.0 {
        fmt(Some(crane.crane_capacity_kg), 2, "kg")
    } else {
        "Not entered".to_string()
    };
    let boom_value = crane
        .boom_m
        .filter(|value| *value != 0.0)
        .map(|boom| fmt(Some(boom), 2, "m"))
        .unwrap_or_else(|| "\u{2014}".to_string());
    let radius_value = if crane.radius_m != 0.0 {
        fmt(Some(crane.radius_m), 2, "m")
    } else {
        "\u{2014}".to_string()
    };
    let angle_value = crane
        .boom_angle_deg
        .filter(|value| *value != 0.0)
        .map(degrees)
        .unwrap_or_else(|| "\u{2014}".to_string());
    let config_value = format!("{boom_value} / {radius_value} / {angle_value}");
    let (usage_band, usage_sub): (String, String) =
        match crane.capacity_usage_percent.map(crane_usage_band) {
            Some(UsageState::Over) => (
                "<span class=\"metric-band over\">WARNING</span>".to_string(),
                "over chart capacity".to_string(),
            ),
            Some(band) if crane.booked_ok == Some(false) => (
                format!(
                    "<span class=\"metric-band neutral\">CHART: {}</span>",
                    band.wording()
                ),
                "crane rating below the 75% rule".to_string(),
            ),
            Some(UsageState::Within) => (
                "<span class=\"metric-band within\">WITHIN LIMIT</span>".to_string(),
                "of chart capacity".to_string(),
            ),
            Some(UsageState::Caution) => (
                "<span class=\"metric-band caution\">CAUTION</span>".to_string(),
                "of chart capacity, past the 75% target".to_string(),
            ),
            None => {
                let message = if crane.chart_capacity_kg.is_some() {
                    "enter the lift weight"
                } else {
                    "enter the load chart"
                };
                (
                    "<span class=\"metric-band neutral\">NOT EVALUATED</span>".to_string(),
                    message.to_string(),
                )
            }
        };
    let usage_cell = format!(
        "<td><span class=\"metric-k\">Usage at working radius</span>\
         <span class=\"metric-line\"><span class=\"metric-v\">{}</span>{}</span>\
         <span class=\"metric-sub\">{}</span></td>",
        escape(&usage_value),
        usage_band,
        escape(&usage_sub)
    );
    let rating_note = match (crane.booked_ok, crane.required_capacity_kg) {
        (Some(false), Some(required)) => format!(
            "<span class=\"metric-band over\">WARNING</span>\
             <span class=\"metric-sub\">{} is below the {} required at the 75% rule</span>",
            fmt(Some(crane.crane_capacity_kg), 2, "kg"),
            fmt(Some(required), 2, "kg")
        ),
        (Some(false), None) => "<span class=\"metric-band over\">WARNING</span>\
            <span class=\"metric-sub\">below the required capacity at 75%</span>"
            .to_string(),
        (Some(true), _) => "<span class=\"metric-sub\">checked against 75%</span>".to_string(),
        (None, _) => "<span class=\"metric-sub\">not checked</span>".to_string(),
    };

    format!(
        "<table class=\"metrics\"><tr>{}{}{}</tr><tr>{}{}{}</tr></table>",
        metric_cell("Gross load at hook", &fmt(Some(gross), 2, "kg"), ""),
        metric_cell(
            "Chart capacity",
            &chart_value,
            "at this radius / boom angle",
        ),
        usage_cell,
        metric_cell("Required capacity at 75%", &required_value, ""),
        metric_cell_html("Crane rating", &booked_value, &rating_note),
        metric_cell("Boom / radius / angle", &config_value, ""),
    )
}

/// The plan's head: title and model, the plan's own lines, then the figures
/// and the verdict. Printed even when no crane section is selected — a plan is
/// signed on paper, and a blank line is what gets a name written on it.
fn plan_head(
    signoff: &PlanSignoff,
    issued_at: Option<&str>,
    gross: f64,
    crane: Option<&CraneResult>,
    working_diagram: Option<&str>,
) -> String {
    let entry = |value: &str| {
        if value.trim().is_empty() {
            crate::figures::NOTHING.to_string()
        } else {
            value.trim().to_string()
        }
    };

    let mut out = String::from("<div class=\"plan-title-block\"><h1>Lifting Plan");
    if let Some(result) = crane {
        if !result.crane_name.is_empty() {
            out.push_str(&format!(
                "<span class=\"plan-crane\"> - {}</span>",
                escape(&result.crane_name)
            ));
        }
    }
    out.push_str("</h1></div>");

    out.push_str("<section class=\"plan-head\">");
    out.push_str("<div class=\"plan-details\">");
    out.push_str(&heading(1, "Plan Details"));
    out.push_str("<table class=\"plan-meta\"><colgroup><col class=\"ref\">\
                  <col class=\"issued\"><col class=\"pb\"><col class=\"cb\">\
                  <col class=\"ab\"></colgroup><tr>");
    for (key, value) in [
        ("Reference", entry(&signoff.reference)),
        ("Issued", entry(issued_at.unwrap_or_default())),
        ("Prepared by", entry(&signoff.prepared_by)),
        ("Checked by", entry(&signoff.checked_by)),
        ("Approved by", entry(&signoff.approved_by)),
    ] {
        out.push_str(&format!(
            "<td><span class=\"k\">{}</span><span class=\"v\">{}</span></td>",
            escape(key),
            escape(&value)
        ));
    }
    out.push_str("</tr></table></div>");

    if let Some(result) = crane {
        out.push_str(&metrics_table(result, gross));
        out.push_str(&crane_section(result, gross));
    }

    if let Some(source) = working_diagram {
        out.push_str("<div class=\"working-diagram\">");
        out.push_str("<div class=\"diagram-title\">Working range diagram</div>");
        out.push_str(&format!(
            "<img src=\"{}\" alt=\"Working range diagram\">",
            escape(source)
        ));
        out.push_str("</div>");
    }

    out.push_str("</section>");
    out
}

/// The reference and credit line every printed page carries, with the defaults
/// the report applies. The PDF export uses the same function, so the footer on
/// the finished PDF is the footer the preview showed.
pub fn footer_texts(reference: &str, center: Option<&str>) -> (String, String) {
    let reference = reference.trim();
    let reference = if reference.is_empty() {
        "No reference".to_string()
    } else {
        reference.to_string()
    };
    let center = center.unwrap_or(DEFAULT_FOOTER);
    (reference, center.to_string())
}

/// The selected report sections as HTML.
///
/// Missing or incomplete state is rendered as an explicit setup message rather
/// than raising while the operator is still entering data.
pub fn build_report_html(inputs: &ReportInputs, include: &[String]) -> String {
    let selected = |name: &str| include.iter().any(|section| section == name);
    let gross = inputs.totals.gross;
    let crane_for_head = if selected("crane") {
        inputs.crane.as_ref()
    } else {
        None
    };
    let reference = if inputs.signoff.reference.trim().is_empty() {
        "No reference".to_string()
    } else {
        inputs.signoff.reference.trim().to_string()
    };
    let working_diagram = inputs
        .working_diagram
        .as_deref()
        .filter(|source| source.starts_with("data:image/png;base64,"));
    let (reference, footer_center) =
        footer_texts(&reference, inputs.footer_center.as_deref());
    let mut parts: Vec<String> = vec![
        REPORT_CSS.to_string(),
        format!(
            "<style>:root {{ --plan-reference: \"{}\"; --footer-center: \"{}\"; }}</style>",
            css_string(&reference),
            css_string(&footer_center)
        ),
        "<div class=\"report\">".to_string(),
        plan_head(
            &inputs.signoff,
            inputs.issued_at.as_deref(),
            gross,
            crane_for_head,
            working_diagram,
        ),
    ];

    if selected("crane") && inputs.crane.is_none() {
        if let Some(error) = &inputs.crane_error {
            // The check could not be made. Saying so keeps a plan from looking
            // like it simply had no crane section.
            parts.push(subheading("Crane capacity check"));
            parts.push(p(
                &format!("{error} The crane check is not part of this plan."),
                "",
            ));
        }
    }

    if selected("uniform") {
        if let Some(sling) = &inputs.sling {
            if let Ok(spec) = hitch_spec(&sling.hitch) {
                parts.push(section_inputs(sling, spec));
                parts.push(format!(
                    "<section class=\"section-card\">{}</section>",
                    geometry_section(sling, spec)
                ));
                parts.push(format!(
                    "<section class=\"section-card\">{}</section>",
                    factors_section(sling, spec)
                ));
                parts.push(format!(
                    "<section class=\"section-card\">{}</section>",
                    capacity_section(sling, spec)
                ));
                if shows_notes(spec) {
                    parts.push(format!(
                        "<section class=\"section-card\">{}</section>",
                        notes_section(sling, spec)
                    ));
                }
            }
        }
    }

    if selected("nonuniform") {
        let first = next_section_number(&parts);
        let section = nonuniform_sections(
            inputs.nonuniform.as_ref(),
            first,
            "Nonuniform Load",
            inputs.nonuniform_error.as_deref(),
        )
        .join("");
        parts.push(format!("<section class=\"section-card\">{section}</section>"));
    }
    if selected("tandem") {
        let first = next_section_number(&parts);
        let section = nonuniform_sections(
            inputs.tandem.as_ref().map(|tandem| {
                NonuniformResult::Tandem(tandem.clone())
            }).as_ref(),
            first,
            "Tandem Lift",
            inputs.tandem_error.as_deref(),
        )
        .join("");
        parts.push(format!("<section class=\"section-card\">{section}</section>"));
    }

    if gross <= 0.0 {
        parts.push(p(
            "<b>Setup note:</b> Add the load and complete the Overall Weight \
             tab before issuing this plan.",
            "border:2px solid #a33; padding:5pt 7pt; margin:5pt 0;",
        ));
    }

    parts.push("</div>".to_string());
    parts.join("")
}

/// The crane, its boom and the capacity check against the load chart.
fn crane_section(crane: &CraneResult, gross: f64) -> String {
    let chart_capacity = crane.chart_capacity_kg;

    // The figures are printed in the plan head; what remains here is the working
    // that produced them. The two formulas sit side by side — the required
    // capacity on the left, the usage on the right — so one row carries both.
    // W is a crane load-chart sizing load. It belongs to the crane check only
    // and is deliberately not used by the sling sections.
    let applied_load = design_load_from_gross(gross);
    let ratio = tex_num(Some(DESIGN_LOAD_RATIO), 2);
    let usage = crane.capacity_usage_percent;

    let left = match (chart_capacity, usage) {
        (Some(capacity), Some(usage)) if capacity != 0.0 => format!(
            "<div class=\"work-label\">Usage, % of chart capacity</div>{}",
            equation(&format!(
                "\\text{{Usage}} = \\frac{{\\text{{Gross load at hook}}}}\
                 {{\\text{{Chart capacity at radius}}}} \\times 100\\% = \
                 \\frac{{{}}}{{{}}} \\times 100\\% = {:.2}\\%",
                tex_num(Some(gross), 2),
                tex_num(Some(capacity), 2),
                usage
            ))
        ),
        _ => "<div class=\"work-label\">Usage, % of chart capacity</div>\
              <div class=\"work-empty\">Not evaluated</div>"
            .to_string(),
    };
    let right = format!(
        "<div class=\"work-label\">Required capacity at 75%, W</div>{}",
        equation(&format!(
            "\\text{{W}} = \\frac{{\\text{{Gross load at hook}}}}{{{ratio}}} = \
             \\frac{{{}}}{{{ratio}}} = {}\\ \\text{{kg}}",
            tex_num(Some(gross), 2),
            tex_num(Some(applied_load), 2)
        ))
    );

    format!(
        "<div class=\"working-wrap\"><table class=\"working\"><tbody><tr><td>{right}</td><td>{left}</td></tr></tbody></table></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crane::{example_crane_inputs, solve_crane};
    use crate::model::{LoadItem, TackleItem};

    fn example_crane() -> CraneResult {
        solve_crane(&example_crane_inputs(), 2000.0).unwrap()
    }

    fn report(crane: CraneResult, include: &[&str]) -> String {
        let inputs = ReportInputs {
            totals: LiftTotals::of(&[LoadItem {
                weight_kg: 2000.0,
                ..LoadItem::default()
            }], &[] as &[TackleItem]),
            crane: Some(crane),
            ..ReportInputs::default()
        };
        build_report_html(
            &inputs,
            &include.iter().map(|value| value.to_string()).collect::<Vec<_>>(),
        )
    }

    #[test]
    fn a_crane_check_that_could_not_be_made_is_named_in_the_plan() {
        let inputs = ReportInputs {
            totals: LiftTotals::of(
                &[LoadItem {
                    weight_kg: 2000.0,
                    ..LoadItem::default()
                }],
                &[] as &[TackleItem],
            ),
            crane: None,
            crane_error: Some(
                "The working radius is outside the load chart, which rates 5.00 m to 10.00 m."
                    .to_string(),
            ),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &["crane".to_string()]);
        assert!(html.contains("Crane capacity check"));
        assert!(html.contains("rates 5.00 m to 10.00 m"));
        assert!(html.contains("The crane check is not part of this plan."));
    }

    #[test]
    fn the_plan_prints_its_own_details_and_sign_off_lines() {
        let inputs = ReportInputs {
            totals: LiftTotals::of(
                &[LoadItem {
                    weight_kg: 2000.0,
                    ..LoadItem::default()
                }],
                &[] as &[TackleItem],
            ),
            signoff: crate::dto::PlanSignoff {
                reference: "LP-2026-014".to_string(),
                prepared_by: "A. Riggs".to_string(),
                checked_by: "S. Appointed".to_string(),
                approved_by: String::new(),
            },
            issued_at: Some("13-11-2026".to_string()),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &["crane".to_string()]);
        assert!(html.contains(
            "<span class=\"k\">Reference</span><span class=\"v\">LP-2026-014</span>"
        ));
        assert!(html.contains(
            "<span class=\"k\">Issued</span><span class=\"v\">13-11-2026</span>"
        ));
        assert!(html.contains(
            "<span class=\"k\">Prepared by</span><span class=\"v\">A. Riggs</span>"
        ));
        assert!(html.contains(
            "<span class=\"k\">Checked by</span><span class=\"v\">S. Appointed</span>"
        ));
        assert!(html.contains(
            "<span class=\"k\">Approved by</span><span class=\"v\">\u{2014}</span>"
        ));
    }

    #[test]
    fn a_plan_with_no_details_still_prints_the_blank_lines() {
        let inputs = ReportInputs::default();
        let html = build_report_html(&inputs, &["crane".to_string()]);
        assert!(html.contains("<h2>1. PLAN DETAILS</h2>"));
        for label in ["Reference", "Issued", "Prepared by", "Checked by", "Approved by"] {
            assert!(
                html.contains(&format!(
                    "<span class=\"k\">{label}</span><span class=\"v\">\u{2014}</span>"
                )),
                "missing blank {label} line"
            );
        }
    }

    #[test]
    fn the_plan_shows_the_crane_rating() {
        let html = report(example_crane(), &["crane"]);
        assert!(html.contains(
            "<span class=\"metric-k\">Crane rating</span>\
             <span class=\"metric-v\">90,000.00 kg</span>"
        ));
        assert!(html.contains("<span class=\"metric-sub\">checked against 75%</span>"));
    }

    #[test]
    fn the_usage_card_asks_for_what_is_missing() {
        let mut no_chart = example_crane();
        no_chart.chart_capacity_kg = None;
        no_chart.capacity_usage_percent = None;
        assert!(metrics_table(&no_chart, 0.0).contains("enter the load chart"));

        let mut no_weight = example_crane();
        no_weight.chart_capacity_kg = Some(16_000.0);
        no_weight.capacity_usage_percent = None;
        assert!(metrics_table(&no_weight, 0.0).contains("enter the lift weight"));
    }

    #[test]
    fn the_working_columns_leave_room_for_a_five_digit_lift() {
        assert!(REPORT_CSS.contains("td:first-child { width: 42%;"));
        assert!(REPORT_CSS.contains("td:last-child { width: 58%;"));
    }

    #[test]
    fn the_plan_head_carries_the_crane_figures() {
        let html = report(example_crane(), &["crane"]);
        for expected in [
            "<h1>Lifting Plan",
            "<span class=\"plan-crane\"> - Kobelco CKE90G-2</span>",
            "<span class=\"metric-k\">Chart capacity</span>\
             <span class=\"metric-v\">16,000.00 kg</span>",
            "<span class=\"metric-k\">Gross load at hook</span>\
             <span class=\"metric-v\">2,000.00 kg</span>",
            "<span class=\"metric-k\">Required capacity at 75%</span>\
             <span class=\"metric-v\">2,666.67 kg</span>",
            "<span class=\"metric-k\">Boom / radius / angle</span>\
             <span class=\"metric-v\">40.00 m / 8.00 m / 48.00\u{00b0}</span>",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
        // The head no longer repeats the configuration, the band or the chart
        // provenance: the cards and the verdict carry them once.
        assert!(!html.contains("plan-sub"));
        assert!(!html.contains("class=\"pill"));
        assert!(!html.contains("Capacity read from"));
        assert!(!html.contains("chart-source"));
        assert!(!html.contains("Booked rating is the capacity"));
    }

    #[test]
    fn the_design_load_equation_is_the_references_own_string() {
        let html = report(example_crane(), &["crane"]);
        assert!(html.contains("<div class=\"work-label\">Required capacity at 75%, W</div>"));
        assert!(html.contains(
            "\\[\\text{W} = \\frac{\\text{Gross load at hook}}{0.75} = \
             \\frac{2{,}000.00}{0.75} = 2{,}666.67\\ \\text{kg}\\]"
        ));
    }

    #[test]
    fn the_usage_equation_is_the_references_own_string() {
        let html = report(example_crane(), &["crane"]);
        assert!(html.contains("<div class=\"work-label\">Usage, % of chart capacity</div>"));
        assert!(html.contains(
            "\\[\\text{Usage} = \\frac{\\text{Gross load at hook}}\
             {\\text{Chart capacity at radius}} \\times 100\\% = \
             \\frac{2{,}000.00}{16{,}000.00} \\times 100\\% = 12.50\\%\\]"
        ));
        assert!(html.contains(
            "<span class=\"metric-band within\">WITHIN LIMIT</span>"
        ));
    }

    #[test]
    fn the_usage_card_bands_follow_the_chart_usage() {
        for (gross, class, phrase) in [
            (12_800.0, "caution", "past the 75% target"),
            (17_600.0, "over", "over chart capacity"),
        ] {
            let crane = solve_crane(&example_crane_inputs(), gross).unwrap();
            let html = metrics_table(&crane, gross);
            assert!(
                html.contains(&format!("<span class=\"metric-band {class}\">")),
                "missing {class} band"
            );
            assert!(html.contains(phrase), "missing {phrase}");
        }
    }

    #[test]
    fn a_failed_booked_check_is_named_on_the_crane_rating_card() {
        let mut crane = example_crane();
        crane.booked_ok = Some(false);
        crane.required_capacity_kg = Some(3_600.0);
        crane.crane_capacity_kg = 3_000.0;
        let html = metrics_table(&crane, 2_000.0);
        assert!(html.contains(
            "3,000.00 kg is below the 3,600.00 kg required at the 75% rule"
        ));
        // The usage card stays the chart band: the chart is not the failing check.
        assert!(html.contains("<span class=\"metric-band neutral\">CHART: WITHIN LIMIT</span>"));
        assert!(html.contains("crane rating below the 75% rule"));
    }

    #[test]
    fn each_working_label_precedes_its_formula() {
        let html = report(example_crane(), &["crane"]);
        let usage_label = html.find("Usage, % of chart capacity").unwrap();
        let usage_formula = html.find("\\text{Usage} =").unwrap();
        assert!(usage_label < usage_formula);
        let required_label = html.find("Required capacity at 75%, W").unwrap();
        let required_formula = html
            .find("\\text{W} = \\frac{\\text{Gross load at hook}}")
            .unwrap();
        assert!(required_label < required_formula);
    }

    #[test]
    fn a_blank_crane_says_not_entered_rather_than_zero() {
        let blank = solve_crane(&crate::crane::CraneInputs::default(), 2000.0).unwrap();
        let html = report(blank.clone(), &["crane"]);
        assert!(html.contains(
            "<span class=\"metric-k\">Crane rating</span>\
             <span class=\"metric-v\">Not entered</span>"
        ));
        assert!(html.contains(
            "<span class=\"metric-k\">Chart capacity</span>\
             <span class=\"metric-v\">Not evaluated</span>"
        ));
        assert!(html.contains(
            "<span class=\"metric-k\">Boom / radius / angle</span>\
             <span class=\"metric-v\">\u{2014} / \u{2014} / \u{2014}</span>"
        ));
        assert_eq!(blank.capacity_status, "Enter the load chart to check capacity.");
        assert!(html.contains(
            "<span class=\"metric-band neutral\">NOT EVALUATED</span>"
        ));
        assert!(html.contains("<span class=\"metric-sub\">enter the load chart</span>"));
    }

    #[test]
    fn a_chart_with_no_capacity_omits_the_usage_equation() {
        let html = report(
            solve_crane(&crate::crane::CraneInputs::default(), 2000.0).unwrap(),
            &["crane"],
        );
        assert!(html.contains("\\text{W} = \\frac{\\text{Gross load at hook}}{0.75}"));
        assert!(!html.contains("\\text{Usage} ="));
    }

    #[test]
    fn a_section_is_gated_on_the_include_set() {
        let nothing = report(example_crane(), &[]);
        assert!(nothing.contains("<h1>Lifting Plan"));
        // The plan's own details are always printed; the solved sections are not.
        assert!(nothing.contains("PLAN DETAILS"));
        assert!(!nothing.contains("Crane rating"));
        assert!(!nothing.contains("Kobelco CKE90G-2"));
        assert!(report(example_crane(), &["crane"]).contains("Kobelco CKE90G-2"));
    }

    #[test]
    fn the_setup_note_appears_only_when_there_is_no_load() {
        assert!(!report(example_crane(), &["crane"]).contains("Setup note:"));
        let inputs = ReportInputs {
            totals: LiftTotals::of(&[] as &[LoadItem], &[] as &[TackleItem]),
            crane: Some(example_crane()),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &["crane".to_string()]);
        assert!(html.contains("<b>Setup note:</b>"));
    }

    #[test]
    fn a_null_figure_is_not_a_zero() {
        assert_eq!(fmt(None, 3, ""), "\u{2014}");
        assert_eq!(fmt(Some(0.0), 2, ""), "0.00");
        assert_eq!(fmt(Some(1239.0), 2, "kg"), "1,239.00 kg");
        assert_eq!(fmt(Some(f64::NAN), 2, ""), "0.00");
        assert_eq!(fmt(Some(f64::INFINITY), 2, ""), "0.00");
    }

    #[test]
    fn thousands_separators_are_braced_so_tex_can_read_them() {
        assert_eq!(tex_num(Some(2000.0), 2), "2{,}000.00");
        assert_eq!(tex_num(Some(999.5), 2), "999.50");
    }

    #[test]
    fn the_wrapper_loads_a_bundled_katex_and_nothing_remote() {
        let page = wrap_katex("<p>body</p>");
        for expected in [
            "<link rel=\"stylesheet\" href=\"vendor/katex/katex.min.css\">",
            "<script src=\"vendor/katex/katex.min.js\"></script>",
            "<script src=\"vendor/katex/auto-render.min.js\"></script>",
            "@page { size: A4; margin: 25.4mm; }",
            "data-katex-error",
            "throwOnError: false",
            r#"{left: "\\(", right: "\\)", display: false}"#,
            r#"{left: "\\[", right: "\\]", display: true}"#,
            "<p>body</p>",
        ] {
            assert!(page.contains(expected), "missing {expected}");
        }
        assert!(!page.contains("http://"));
        assert!(!page.contains("https://"));
    }

    #[test]
    fn every_printed_page_carries_the_footer() {
        for expected in [
            "@bottom-left",
            "@bottom-center",
            "@bottom-right",
            "var(--plan-reference",
            "var(--footer-center",
            "counter(page)",
            "counter(pages)",
            "Prepared with Lifting Plan Calculator by Zhafri Syazwi",
        ] {
            assert!(REPORT_CSS.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_footer_uses_the_plan_reference() {
        let inputs = ReportInputs {
            signoff: crate::dto::PlanSignoff {
                reference: "LP-2026-014".to_string(),
                ..crate::dto::PlanSignoff::default()
            },
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &[]);
        assert!(html.contains("--plan-reference: \"LP-2026-014\""));
    }

    #[test]
    fn the_footer_centre_text_is_editable() {
        let inputs = ReportInputs {
            footer_center: Some("Custom footer".to_string()),
            ..ReportInputs::default()
        };
        assert!(build_report_html(&inputs, &[]).contains("--footer-center: \"Custom footer\""));
    }

    #[test]
    fn the_plan_embeds_an_attached_working_diagram() {
        let inputs = ReportInputs {
            working_diagram: Some("data:image/png;base64,AAAA".to_string()),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &["crane".to_string()]);
        assert!(html.contains("<img src=\"data:image/png;base64,AAAA\""));
        assert!(html.contains("Working range diagram"));
    }

    #[test]
    fn an_untrusted_diagram_source_is_ignored() {
        let inputs = ReportInputs {
            working_diagram: Some("https://example.com/x.png".to_string()),
            ..ReportInputs::default()
        };
        assert!(!build_report_html(&inputs, &[]).contains("<div class=\"working-diagram\">"));
    }

    #[test]
    fn a_tabs_report_carries_its_own_section_and_the_crane() {
        assert_eq!(report_sections_for("crane", true), vec!["crane"]);
        assert_eq!(report_sections_for("crane", false), vec!["crane"]);
        assert_eq!(report_sections_for("uniform", true), vec!["crane", "uniform"]);
        assert_eq!(report_sections_for("uniform", false), vec!["uniform"]);
        assert_eq!(report_sections_for("tandem", true), vec!["crane", "tandem"]);
        assert_eq!(
            report_sections_for("nonuniform", false),
            vec!["nonuniform"]
        );
    }

    #[test]
    fn the_next_section_number_is_counted_not_assumed() {
        assert_eq!(next_section_number(&[]), 1);
        assert_eq!(
            next_section_number(&["<h2>1. LIFT INPUT PARAMETERS</h2>".to_string()]),
            2
        );
        assert_eq!(
            next_section_number(&[
                "<h2>1. LIFT INPUT PARAMETERS</h2>".to_string(),
                "<h2>2. GEOMETRIC &amp; HEADROOM CALCULATIONS</h2>".to_string()
            ]),
            3
        );
        assert_eq!(
            next_section_number(&[
                "<h1>Lifting Plan</h1>".to_string(),
                "<h2>1. ONE</h2>".to_string()
            ]),
            2
        );
    }

    #[test]
    fn a_reference_cannot_close_the_style_element() {
        // The reference and the credit line are written into a `<style>` block
        // as CSS strings, and the preview is an unsandboxed `srcdoc` frame, so a
        // closing tag there would run script in the main window.
        let hostile = "</style><img src=x onerror=alert(1)>";
        let escaped = css_string(hostile);
        assert!(!escaped.contains('<'), "{escaped}");
        assert!(escaped.contains("\\3c "), "{escaped}");
        assert!(escaped.contains("alert(1)"), "the text is escaped, not dropped");
        // Quotes and backslashes still cannot end the string early.
        assert_eq!(css_string("a\"b\\c"), "a\\\"b\\\\c");
    }
}
