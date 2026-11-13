//! The HTML report builder, ported from `Report.kt`.
//!
//! The report is deliberately independent of the UI: [`build_report_html`]
//! takes the solved state and returns a self-contained body that a webview, a
//! browser, or a test can render. The structure follows the numbered sections
//! in the reference documents, while keeping the formulas readable when KaTeX
//! is unavailable.

use crate::crane::{CraneResult, DESIGN_LOAD_RATIO};
use crate::dto::PlanSignoff;
use crate::model::LiftTotals;
use crate::nonuniform::{NonuniformResult, TandemResult};
use crate::report_capacity::{capacity_section, notes_section, shows_notes, factors_section};
use crate::report_nonuniform::{nonuniform_sections};
use crate::report_uniform::{geometry_section, section_inputs};
use crate::sling::{hitch_spec, SlingResult};

pub const REPORT_CSS: &str = r#"
<style>
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
.report td, .report th { border: 1px solid #7c8998; padding: 1pt 2pt;
    vertical-align: top; font-size: 5.5pt; }
.report th { background: #f1f4f7; text-align: left; font-weight: bold; }
.report .equation { margin: 2pt 0 3pt 12pt; font-family: "DejaVu Serif", serif;
    font-size: 5pt; }
.report .equation .fallback { font-family: "Liberation Serif", serif; }
.report .report-footer { margin-top: 14pt; padding-top: 5pt; text-align: center;
    border-top: 1px solid #9aa9ba; color: #555; font-size: 5.5pt; }
/* **A rule was removed here.** The reference's stylesheet carries
   `@media print { .report .report-footer { display: none; } }`, with no comment
   saying why. The effect was that the footer showed in the preview and vanished
   from the printed page -- so the one thing a printed plan could not tell you was
   why the two disagreed, and the screen gave no hint that they would. Rather than
   guess at an intent nobody wrote down, the printed page now carries the same
   footer the preview does. Restore that single line to get the desktop's
   behaviour back. */
.report .note { border: 1px solid #9aa9ba; background: #f7f7f7; padding: 5pt 7pt;
    margin-top: 5pt; }
.report .critical { border: 2px solid #a33; padding: 5pt 7pt; margin: 5pt 0; }
.report .pass { color: #176b3a; font-weight: bold; }
.report .fail { color: #9b1c1c; font-weight: bold; }
.report .small { font-size: 7pt; } .report .note { font-size: 7pt; } .report .status { font-size: 7pt; } .report .meta { font-size: 7pt; } .report .report-footer { font-size: 7pt; } .report p { font-size: 7pt; } .report td, .report th { font-size: 7pt; } .report .equation { font-size: 6.5pt; } .report h1 { font-size: 12pt; } .report h2 { font-size: 9pt; } .report h3 { font-size: 8pt; }
/* The plan's head: a light card that reads top-down — what the plan is, the
   figures that decide it, then the verdict. No heavy grid; the values carry
   the weight and the labels stay quiet. */
.report { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
.report .plan-head { border: 1px solid #d9e1e9; border-radius: 5pt;
    background: #fbfcfe; padding: 5pt 9pt 5pt; margin: 0 0 6pt; }
.report table.plan-bar { width: 100%; border-collapse: collapse; margin: 0; }
.report table.plan-bar td { border: 0; padding: 0; vertical-align: top; }
.report .plan-bar h1 { font-size: 16pt; line-height: 1; margin: 0;
    text-align: left; letter-spacing: -0.3pt; color: #101820; }
.report .plan-crane { font-size: 9pt; font-weight: bold; color: #1b2733;
    margin: 2pt 0 0; }
.report .plan-sub { font-size: 6.5pt; color: #5d6b7a; margin: 1pt 0 0; }
.report .plan-status { width: 27%; text-align: right; white-space: nowrap;
    vertical-align: bottom; }
.report .pill-label { font-size: 5pt; letter-spacing: 0.4pt;
    text-transform: uppercase; color: #6d7a88; margin-bottom: 1.5pt; }
.report .pill { display: inline-block; padding: 2.5pt 7pt; border-radius: 9pt;
    border: 1px solid #cfd8e2; background: #eef2f6; color: #4a5560;
    font-size: 7.5pt; font-weight: bold; letter-spacing: 0.4pt; }
.report .pill.within { background: #e7f4ec; border-color: #b7dcc6; color: #176b3a; }
.report .pill.caution { background: #fdf1e2; border-color: #e8c79c; color: #8a5517; }
.report .pill.over { background: #fbe9e9; border-color: #e3b4b4; color: #9b1c1c; }
.report .pill.neutral { background: #eef2f6; border-color: #cfd8e2; color: #4a5560; }
.report .plan-details { margin-top: 5pt; padding-top: 3pt;
    border-top: 1px solid #e4eaf1; }
.report .plan-meta-label { font-size: 5.5pt; letter-spacing: 0.6pt;
    text-transform: uppercase; color: #6d7a88; margin-bottom: 2pt; }
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
.report .metric-k { display: block; font-size: 5.5pt; letter-spacing: 0.3pt;
    text-transform: uppercase; color: #6d7a88; }
.report .metric-v { display: block; font-size: 10pt; font-weight: bold;
    line-height: 1.1; color: #0f1720; margin-top: 0.5pt; }
.report .metric-n { display: block; font-size: 5.5pt; color: #5d6b7a;
    margin-top: 1pt; line-height: 1.2; }
.report .verdict { font-size: 8.5pt; font-weight: bold; margin: 4pt 0 0;
    padding: 3pt 6pt; border-left: 3pt solid #7c8998;
    border-radius: 0 3pt 3pt 0; background: #f4f6f8; line-height: 1.2; }
.report .verdict.within { border-left-color: #2e7d52; color: #176b3a;
    background: #f0f8f3; }
.report .verdict.caution { border-left-color: #b06a1e; color: #8a5517;
    background: #fdf7ee; }
.report .verdict.over { border-left-color: #b23b3b; color: #9b1c1c;
    background: #fdf0f0; }
.report .verdict.neutral { border-left-color: #7c8998; color: #4a5560; }
.report .source { font-size: 5.8pt; color: #6d7a88; margin: 2pt 0 0; }
.report .explain { font-size: 5.5pt; color: #6d7a88; margin: 1.5pt 0 0;
    line-height: 1.25; }
.report .explain b { color: #4a5560; }
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
    let mut out = format!(
        "<td><span class=\"metric-k\">{}</span><span class=\"metric-v\">{}</span>",
        escape(key),
        escape(value)
    );
    if !note.is_empty() {
        out.push_str(&format!("<span class=\"metric-n\">{}</span>", escape(note)));
    }
    out.push_str("</td>");
    out
}

/// The verdict as a pill: the band word first, so the page answers "does it
/// pass?" before the reader reaches the sentence that explains why.
fn band_pill(crane: &CraneResult) -> String {
    match crane.capacity_band {
        Some(band) => format!(
            "<span class=\"pill {}\">{}</span>",
            band.key(),
            band.wording()
        ),
        None => "<span class=\"pill neutral\">NOT EVALUATED</span>".to_string(),
    }
}

/// `48.00°` rather than `48.00 °` in the head's prose.
fn degrees(angle: f64) -> String {
    fmt(Some(angle), 2, "\u{00b0}").replace(" \u{00b0}", "\u{00b0}")
}

/// Boom, angle and radius as one quiet line under the crane model.
fn crane_subtitle(crane: &CraneResult) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(boom) = crane.boom_m.filter(|value| *value != 0.0) {
        parts.push(format!("{} boom", fmt(Some(boom), 2, "m")));
    }
    if let Some(angle) = crane.boom_angle_deg.filter(|value| *value != 0.0) {
        parts.push(degrees(angle));
    }
    if crane.radius_m != 0.0 {
        parts.push(format!("{} radius", fmt(Some(crane.radius_m), 2, "m")));
    }
    parts.join(" \u{00b7} ")
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
    let booked_note = match (crane.booked_ok, crane.required_capacity_kg) {
        (Some(true), Some(required)) => format!(
            "passes \u{2014} {} needed at 75%",
            fmt(Some(required), 2, "kg")
        ),
        (Some(false), Some(required)) => format!(
            "below \u{2014} {} needed at 75%",
            fmt(Some(required), 2, "kg")
        ),
        _ => "not checked".to_string(),
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
    let config_value = format!("{boom_value} / {radius_value}");
    let config_note = crane
        .boom_angle_deg
        .filter(|value| *value != 0.0)
        .map(|angle| format!("at {}", degrees(angle)))
        .unwrap_or_default();

    format!(
        "<table class=\"metrics\"><tr>{}{}{}</tr><tr>{}{}{}</tr></table>",
        metric_cell("Gross load at hook", &fmt(Some(gross), 2, "kg"), ""),
        metric_cell(
            "Chart capacity",
            &chart_value,
            "at this radius / boom angle",
        ),
        metric_cell("Usage at working radius", &usage_value, "of chart capacity"),
        metric_cell("Required capacity at 75%", &required_value, ""),
        metric_cell("Crane booked rating", &booked_value, &booked_note),
        metric_cell("Boom / radius", &config_value, &config_note),
    )
}

/// The band sentence, in its own colour and with the working below it.
fn verdict_line(crane: &CraneResult) -> String {
    let band = crane
        .capacity_band
        .map(|band| band.key())
        .unwrap_or("neutral");
    format!(
        "<p class=\"verdict {}\">{}</p>",
        band,
        escape(if crane.capacity_status.is_empty() {
            "Not evaluated"
        } else {
            &crane.capacity_status
        })
    )
}

/// Which chart the figure came from, as a caption rather than a table row.
fn source_line(crane: &CraneResult) -> String {
    let path = crane.chart_path.clone().unwrap_or_default();
    let source = [crane.capacity_source.as_str(), path.as_str()]
        .iter()
        .filter(|value| !value.is_empty())
        .copied()
        .collect::<Vec<&str>>()
        .join(" ");
    format!(
        "<p class=\"source\">Capacity read from <b>{}</b></p>",
        escape(if source.is_empty() {
            "not evaluated"
        } else {
            &source
        })
    )
}

/// The plan's head: title and status, the plan's own lines, then the figures
/// and the verdict. Printed even when no crane section is selected — a plan is
/// signed on paper, and a blank line is what gets a name written on it.
fn plan_head(
    signoff: &PlanSignoff,
    issued_at: Option<&str>,
    gross: f64,
    crane: Option<&CraneResult>,
    crane_selected: bool,
) -> String {
    let entry = |value: &str| {
        if value.trim().is_empty() {
            crate::figures::NOTHING.to_string()
        } else {
            value.trim().to_string()
        }
    };

    let mut out = String::from("<section class=\"plan-head\">");
    out.push_str("<table class=\"plan-bar\"><tr><td colspan=\"2\">");
    out.push_str("<h1>Lifting Plan</h1>");
    out.push_str("</td></tr><tr><td class=\"plan-crane-cell\">");
    if let Some(result) = crane {
        if !result.crane_name.is_empty() {
            out.push_str(&format!(
                "<p class=\"plan-crane\">{}</p>",
                escape(&result.crane_name)
            ));
        }
        let subtitle = crane_subtitle(result);
        if !subtitle.is_empty() {
            out.push_str(&format!(
                "<p class=\"plan-sub\">{}</p>",
                escape(&subtitle)
            ));
        }
    }
    out.push_str("</td><td class=\"plan-status\">");
    if crane_selected {
        match crane {
            Some(result) => {
                out.push_str("<div class=\"pill-label\">Capacity check</div>");
                out.push_str(&band_pill(result));
            }
            None => {
                out.push_str("<div class=\"pill-label\">Crane check</div>");
                out.push_str("<span class=\"pill neutral\">NOT INCLUDED</span>");
            }
        }
    }
    out.push_str("</td></tr></table>");

    out.push_str("<div class=\"plan-details\"><div class=\"plan-meta-label\">Plan details</div>");
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
        out.push_str(&verdict_line(result));
        out.push_str(&source_line(result));
        out.push_str(
            "<p class=\"explain\"><b>Booked rating</b> is the capacity on the \
             crane's plate as booked. <b>Chart capacity</b> is what the chart \
             allows at this radius. The booked check catches a machine that is \
             too small at any radius.</p>",
        );
    }

    out.push_str("</section>");
    out
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
    let mut parts: Vec<String> = vec![
        REPORT_CSS.to_string(),
        "<div class=\"report\">".to_string(),
        plan_head(
            &inputs.signoff,
            inputs.issued_at.as_deref(),
            gross,
            crane_for_head,
            selected("crane"),
        ),
    ];

    if selected("crane") {
        if let Some(crane) = &inputs.crane {
            parts.push(crane_section(crane, gross));
        } else if let Some(error) = &inputs.crane_error {
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
                parts.push(geometry_section(sling, spec));
                parts.push(factors_section(sling, spec));
                parts.push(capacity_section(sling, spec));
                if shows_notes(spec) {
                    parts.push(notes_section(sling, spec));
                }
            }
        }
    }

    if selected("nonuniform") {
        let first = next_section_number(&parts);
        parts.extend(nonuniform_sections(
            inputs.nonuniform.as_ref(),
            first,
            "Nonuniform Load",
            inputs.nonuniform_error.as_deref(),
        ));
    }
    if selected("tandem") {
        let first = next_section_number(&parts);
        parts.extend(nonuniform_sections(
            inputs.tandem.as_ref().map(|tandem| {
                NonuniformResult::Tandem(tandem.clone())
            }).as_ref(),
            first,
            "Tandem Lift",
            inputs.tandem_error.as_deref(),
        ));
    }

    if gross <= 0.0 {
        parts.push(p(
            "<b>Setup note:</b> Add the load and complete the Overall Weight \
             tab before issuing this plan.",
            "border:2px solid #a33; padding:5pt 7pt; margin:5pt 0;",
        ));
    }

    parts.push(
        "<div class=\"report-footer\"><b>Calculation aid only.</b> This plan \
         does not certify the lift, the crane, the equipment or its operation. \
         Check every figure against the manufacturer's load chart and your \
         site's lift plan before the lift. Prepared with Lifting Plan \
         Calculator by Zhafri Syazwi \
         <span class=\"email-icon\">&#9993;</span> zhafribs@gmail.com</div>"
            .to_string(),
    );
    parts.push("</div>".to_string());
    parts.join("")
}

/// The crane, its boom and the capacity check against the load chart.
fn crane_section(crane: &CraneResult, gross: f64) -> String {
    let chart_capacity = crane.chart_capacity_kg;
    let mut parts: Vec<String> = Vec::new();

    // The figures and the verdict are printed in the plan head; what remains
    // here is the working that produced them.
    // W is a crane load-chart sizing load. It belongs to the crane check only
    // and is deliberately not used by the sling sections.
    let applied_load = design_load_from_gross(gross);
    let ratio = tex_num(Some(DESIGN_LOAD_RATIO), 2);
    parts.push(subheading(
        "Crane design load for the 75% allowable check, W:",
    ));
    parts.push(equation(&format!(
        "\\text{{W}} = \\frac{{\\text{{Gross load at hook}}}}{{{ratio}}} = \
         \\frac{{{}}}{{{ratio}}} = {}\\ \\text{{kg}}",
        tex_num(Some(gross), 2),
        tex_num(Some(applied_load), 2)
    )));
    parts.push(subheading("75% Allowable:"));
    let usage = crane.capacity_usage_percent;
    if let (Some(capacity), Some(usage)) = (chart_capacity, usage) {
        if capacity != 0.0 {
            parts.push(equation(&format!(
                "\\text{{Usage}} = \\frac{{\\text{{Gross load at hook}}}}\
                 {{\\text{{Chart capacity at radius}}}} \\times 100\\% = \
                 \\frac{{{}}}{{{}}} \\times 100\\% = {:.2}\\%",
                tex_num(Some(gross), 2),
                tex_num(Some(capacity), 2),
                usage
            )));
        }
    }

    parts.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::UsageState;
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
        assert!(html.contains("<div class=\"pill-label\">Crane check</div>"));
        assert!(html.contains("<span class=\"pill neutral\">NOT INCLUDED</span>"));
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
            issued_at: Some("2026-11-13 09:41".to_string()),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &["crane".to_string()]);
        assert!(html.contains(
            "<span class=\"k\">Reference</span><span class=\"v\">LP-2026-014</span>"
        ));
        assert!(html.contains(
            "<span class=\"k\">Issued</span><span class=\"v\">2026-11-13 09:41</span>"
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
        assert!(html.contains("<div class=\"plan-meta-label\">Plan details</div>"));
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
    fn the_plan_states_the_booked_rating_check() {
        let html = report(example_crane(), &["crane"]);
        assert!(html.contains(
            "<span class=\"metric-k\">Crane booked rating</span>\
             <span class=\"metric-v\">90,000.00 kg</span>"
        ));
        assert!(html.contains(
            "<span class=\"metric-n\">passes \u{2014} 2,666.67 kg needed at 75%</span>"
        ));
    }

    #[test]
    fn the_plan_head_carries_the_crane_figures() {
        let html = report(example_crane(), &["crane"]);
        for expected in [
            "<h1>Lifting Plan</h1>",
            "<p class=\"plan-crane\">Kobelco CKE90G-2</p>",
            "<div class=\"pill-label\">Capacity check</div>",
            "<span class=\"pill within\">WITHIN LIMIT</span>",
            "40.00 m boom",
            "48.00\u{00b0}",
            "8.00 m radius",
            "<span class=\"metric-k\">Chart capacity</span>\
             <span class=\"metric-v\">16,000.00 kg</span>",
            "<span class=\"metric-k\">Gross load at hook</span>\
             <span class=\"metric-v\">2,000.00 kg</span>",
            "<span class=\"metric-k\">Required capacity at 75%</span>\
             <span class=\"metric-v\">2,666.67 kg</span>",
            "The booked check catches a machine that is too small at any radius.",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_chart_source_is_a_quiet_caption() {
        assert!(report(example_crane(), &["crane"])
            .contains("<p class=\"source\">Capacity read from <b>"));
        let blank = report(
            solve_crane(&crate::crane::CraneInputs::default(), 2000.0).unwrap(),
            &["crane"],
        );
        assert!(blank.contains("Capacity read from <b>not evaluated</b>"));
    }

    #[test]
    fn the_pill_follows_the_band_wording() {
        for (band, class, word) in [
            (UsageState::Caution, "caution", "CAUTION"),
            (UsageState::Over, "over", "WARNING"),
        ] {
            let mut crane = example_crane();
            crane.capacity_band = Some(band);
            let html = report(crane, &["crane"]);
            assert!(
                html.contains(&format!("<span class=\"pill {class}\">{word}</span>")),
                "missing {word} pill"
            );
        }
    }

    #[test]
    fn a_booked_rating_below_the_requirement_is_not_hidden() {
        let mut crane = example_crane();
        crane.booked_ok = Some(false);
        crane.required_capacity_kg = Some(2666.67);
        crane.crane_capacity_kg = 10_000.0;
        let html = metrics_table(&crane, 2000.0);
        assert!(html.contains(
            "<span class=\"metric-n\">below \u{2014} 2,666.67 kg needed at 75%</span>"
        ));
    }

    #[test]
    fn the_design_load_equation_is_the_references_own_string() {
        let html = report(example_crane(), &["crane"]);
        assert!(html.contains("<h3>Crane design load for the 75% allowable check, W:</h3>"));
        assert!(html.contains(
            "\\[\\text{W} = \\frac{\\text{Gross load at hook}}{0.75} = \
             \\frac{2{,}000.00}{0.75} = 2{,}666.67\\ \\text{kg}\\]"
        ));
    }

    #[test]
    fn the_usage_equation_is_the_references_own_string() {
        let html = report(example_crane(), &["crane"]);
        assert!(html.contains("<h3>75% Allowable:</h3>"));
        assert!(html.contains(
            "\\[\\text{Usage} = \\frac{\\text{Gross load at hook}}\
             {\\text{Chart capacity at radius}} \\times 100\\% = \
             \\frac{2{,}000.00}{16{,}000.00} \\times 100\\% = 12.50\\%\\]"
        ));
        assert!(html.contains(
            "<p class=\"verdict within\">WITHIN LIMIT \u{2014} 12.50% of chart capacity at working radius</p>"
        ));
    }

    #[test]
    fn a_blank_crane_says_not_entered_rather_than_zero() {
        let blank = solve_crane(&crate::crane::CraneInputs::default(), 2000.0).unwrap();
        let html = report(blank.clone(), &["crane"]);
        assert!(html.contains(
            "<span class=\"metric-k\">Crane booked rating</span>\
             <span class=\"metric-v\">Not entered</span>"
        ));
        assert!(html.contains(
            "<span class=\"metric-k\">Chart capacity</span>\
             <span class=\"metric-v\">Not evaluated</span>"
        ));
        assert!(html.contains(
            "<span class=\"metric-k\">Boom / radius</span>\
             <span class=\"metric-v\">\u{2014} / \u{2014}</span>"
        ));
        assert!(html.contains("<span class=\"pill neutral\">NOT EVALUATED</span>"));
        assert!(html.contains("<span class=\"metric-n\">not checked</span>"));
        assert_eq!(blank.capacity_status, "Enter the load chart to check capacity.");
        assert!(html.contains(
            "<p class=\"verdict neutral\">Enter the load chart to check capacity.</p>"
        ));
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
        assert!(nothing.contains("<h1>Lifting Plan</h1>"));
        // The plan's own details are always printed; the solved sections are not.
        assert!(nothing.contains("Plan details"));
        assert!(!nothing.contains("Crane booked rating"));
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
    fn the_footer_survives_the_print_stylesheet() {
        // Strip CSS comments before checking for a print-media rule.
        let mut stripped = String::new();
        let mut rest = REPORT_CSS;
        while let Some(start) = rest.find("/*") {
            stripped.push_str(&rest[..start]);
            match rest[start..].find("*/") {
                Some(end) => rest = &rest[start + end + 2..],
                None => {
                    rest = "";
                    break;
                }
            }
        }
        stripped.push_str(rest);
        assert!(
            !stripped.contains("@media"),
            "no print-media rule may hide the footer"
        );
        assert!(REPORT_CSS.contains(".report .report-footer"));
        assert!(REPORT_CSS.contains("text-align: center;"));
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
}
