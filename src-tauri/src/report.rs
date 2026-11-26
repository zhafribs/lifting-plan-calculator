//! The HTML report builder, ported from `Report.kt`.
//!
//! The report is deliberately independent of the UI: [`build_report_html`]
//! takes the solved state and returns a self-contained body that a webview, a
//! browser, or a test can render. The structure follows the numbered sections
//! in the reference documents, while keeping the formulas readable when KaTeX
//! is unavailable.

use serde::{Deserialize, Serialize};

use crate::calc::{crane_usage_band, UsageState};
use crate::crane::{CraneResult, DESIGN_LOAD_RATIO};
use crate::dto::PlanSignoff;
use crate::model::{LiftTotals, LoadItem, TackleItem};
use crate::nonuniform::{NonuniformResult, TandemResult};
use crate::outrigger::OutriggerResult;
use crate::report_capacity::{capacity_section, notes_section, shows_notes, factors_section};
use crate::report_nonuniform::{nonuniform_sections};
use crate::report_outrigger::outrigger_section;
use crate::report_uniform::{geometry_section, section_inputs};
use crate::sling::{hitch_spec, SlingResult};

/// The centre text of every printed page's footer, unless the report screen
/// supplies its own.
pub const DEFAULT_FOOTER: &str =
    "Prepared with Lifting Plan Calculator by Zhafri Syazwi — zhafribs@gmail.com";

/// The `@page` margin boxes that draw the footer when a browser prints the
/// report. The PDF export removes them and stamps the footer onto the pages
/// itself, so the footer does not depend on which engine printed or how new it
/// is; this copy exists so that removal cannot silently miss.
const PAGE_FOOTER_CSS: &str = r#"@page {
    @bottom-left { content: var(--plan-reference, "No reference"); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
    @bottom-center { content: var(--footer-center, "Prepared with Lifting Plan Calculator by Zhafri Syazwi — zhafribs@gmail.com"); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
    @bottom-right { content: counter(page) " / " counter(pages); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
}
"#;

pub use crate::report_style::REPORT_CSS;

/// The page wrapper: assemble the report into a page that renders its own
/// equations. Ported from `ReportPage.kt`.
///
/// **The KaTeX distribution is bundled rather than fetched**, so the page
/// renders with no network and no remote content: a lift plan read on site must
/// not change because a CDN did.
pub const PRINT_MARGIN_MM: f64 = 25.4;

/// The narrowest and widest margin the app will print with: below 5 mm the
/// printer's own unprintable edge is reached, above 50 mm the sheet has no
/// usable page left.
pub const MIN_MARGIN_MM: f64 = 5.0;
pub const MAX_MARGIN_MM: f64 = 50.0;

/// The printed page's margins, in millimetres.
///
/// One value drives all three surfaces — the web view's print, the footer
/// stamping, and the exported HTML's `@page` — so a page cannot be printed
/// with one margin and labelled with another.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PageMargins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for PageMargins {
    fn default() -> Self {
        Self {
            top: PRINT_MARGIN_MM,
            right: PRINT_MARGIN_MM,
            bottom: PRINT_MARGIN_MM,
            left: PRINT_MARGIN_MM,
        }
    }
}

impl PageMargins {
    /// Every side inside what the app will print, so a typed value cannot
    /// produce a page with no content area or an unprintable edge.
    pub fn clamped(self) -> Self {
        let side = |value: f64| {
            if value.is_finite() {
                value.clamp(MIN_MARGIN_MM, MAX_MARGIN_MM)
            } else {
                PRINT_MARGIN_MM
            }
        };
        Self {
            top: side(self.top),
            right: side(self.right),
            bottom: side(self.bottom),
            left: side(self.left),
        }
    }

    /// The area left for the plan on A4, in millimetres.
    pub fn content_mm(self) -> (f64, f64) {
        let margins = self.clamped();
        (
            (210.0 - margins.left - margins.right).max(0.0),
            (297.0 - margins.top - margins.bottom).max(0.0),
        )
    }
}

/// `body` as a complete page that typesets its own equations, at the default
/// one-inch margin.
pub fn wrap_katex(body: &str) -> String {
    wrap_katex_with(body, PageMargins::default())
}

/// `body` as a complete page, with the page margin the operator chose.
pub fn wrap_katex_with(body: &str, margins: PageMargins) -> String {
    let margins = margins.clamped();
    let mut out = String::new();
    out.push_str("<!DOCTYPE html><html><head><meta charset=\"utf-8\">");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    out.push_str("<style>");
    // The four-value shorthand only where the sides differ; the common
    // one-inch page keeps the plain declaration.
    let margin = if margins.top == margins.right
        && margins.right == margins.bottom
        && margins.bottom == margins.left
    {
        format!("{}mm", margins.top)
    } else {
        format!(
            "{}mm {}mm {}mm {}mm",
            margins.top, margins.right, margins.bottom, margins.left
        )
    };
    out.push_str(&format!("@page {{ size: A4; margin: {margin}; }} "));
    out.push_str("html, body { background: #fff; margin: 0; padding: 0; } ");
    out.push_str("body { background-color: #fff; color: #111; font-size: 7pt; line-height: 1.2; } ");
    out.push_str("table { font-size: inherit; } ");
    out.push_str(".katex-display { text-align: left; margin: 3px 0; } ");
    out.push_str("</style>");
    out.push_str("<link rel=\"stylesheet\" href=\"vendor/katex/katex.min.css\">");
    out.push_str("<script src=\"vendor/katex/katex.min.js\"></script>");
    out.push_str("<script src=\"vendor/katex/auto-render.min.js\"></script>");
    // A file, not an inline pass: the app's CSP allows scripts from its own
    // origin only.
    out.push_str("<script src=\"vendor/katex/lpc-autorender.js\"></script>");
    out.push_str("</head><body>");
    out.push_str(body);
    out.push_str("</body></html>");
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
    pub outrigger: Option<OutriggerResult>,
    /// Why the outrigger pad could not be sized, or None when it could.
    pub outrigger_error: Option<String>,
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
    /// The load lines and tackle the plan was built from, in the order the
    /// Overall Weight tab lists them.
    pub load_items: Vec<LoadItem>,
    pub tackle_items: Vec<TackleItem>,
    /// The Graph tab's working-range diagram, as a PNG or SVG data URL, or None.
    pub working_diagram: Option<String>,
    /// The diagram's width as a percentage of its column; None is 100.
    pub diagram_scale: Option<f64>,
    /// The centre text of every printed page's footer; None uses the default.
    pub footer_center: Option<String>,
    /// Sections the operator wants to start on a new page: "crane",
    /// "outrigger", "uniform", "nonuniform", "tandem".
    pub page_breaks: Vec<String>,
}

/// The sections a report may carry, in the builder's own source order.
pub const REPORT_SECTIONS: [&str; 5] = ["crane", "outrigger", "uniform", "nonuniform", "tandem"];

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

/// A metric whose note sits beside the label, wrapping under it only when the
/// cell cannot hold both.
fn metric_cell_aside(key: &str, value: &str, note: &str) -> String {
    format!(
        "<td><span class=\"metric-head\"><span class=\"metric-k\">{}</span>\
         <span class=\"metric-aside\">{}</span></span>\
         <span class=\"metric-v\">{}</span></td>",
        escape(key),
        escape(note),
        escape(value)
    )
}

/// Two workings side by side: a label and its formula in each cell.
///
/// The report's one two-column row, so a pair of formulas that are read
/// together (the crane's required capacity and usage, a sling's angle factor
/// and reeve factor) costs one row instead of two.
pub fn working_pair(left: &str, right: &str) -> String {
    format!(
        "<div class=\"working-wrap\"><table class=\"working\"><tbody>\
         <tr><td>{left}</td><td>{right}</td></tr></tbody></table></div>"
    )
}

/// A working block: a label and the formula under it.
pub fn work_block(label: &str, formula: &str) -> String {
    format!("<div class=\"work-label\">{}</div>{formula}", escape(label))
}

/// Two notes side by side, left-aligned prose with the same divider as a
/// workings row: the section that explains the numbers is read in one pass.
pub fn notes_pair(left: &str, right: &str) -> String {
    format!(
        "<div class=\"working-wrap\"><table class=\"working notes\"><tbody>\
         <tr><td>{left}</td><td>{right}</td></tr></tbody></table></div>"
    )
}

/// The workings laid out two to a row. A block marked `data-wide` — one whose
/// label or formula is too long for half a page — takes the whole row instead:
/// a wider cell is print-safe, while an over-wide page shrinks as a whole.
/// Roughly how wide a LaTeX piece prints: command names and braces are not
/// glyphs, so the raw source length overstates a `\text` or `\frac`.
fn visible_width(latex: &str) -> usize {
    let mut width = 0;
    let mut command = false;
    for ch in latex.chars() {
        if command {
            if ch.is_ascii_alphabetic() {
                continue;
            }
            command = false;
        }
        match ch {
            '\\' => command = true,
            '{' | '}' | '^' | '_' | ' ' => {}
            _ => width += 1,
        }
    }
    width
}

pub fn split_long_equation(part: &str) -> String {
    if part.len() < 120 || part.contains("\\begin{aligned}") {
        return part.to_string();
    }
    let (Some(start), Some(end)) = (part.find("\\["), part.rfind("\\]")) else {
        return part.to_string();
    };
    let latex = &part[start + 2..end];
    let mut pieces: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut from = 0usize;
    for (index, ch) in latex.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            '=' if depth == 0 => {
                pieces.push(latex[from..index].trim().to_string());
                from = index + 1;
            }
            _ => {}
        }
    }
    pieces.push(latex[from..].trim().to_string());
    if pieces.len() < 2 {
        return part.to_string();
    }
    if pieces.len() == 2 {
        let aligned = format!(
            "\\begin{{aligned}}{} &= {}\\end{{aligned}}",
            pieces[0], pieces[1]
        );
        return format!("<div class=\"equation\">\\[{aligned}\\]</div>");
    }

    // Split into two lines at the point that keeps the wider line shortest.
    // A twenty-odd character line fits the half-column; a longer one takes
    // the full row, so the page never has to shrink to make it fit.
    let mut at_best = 2;
    let mut widest = usize::MAX;
    for at in 2..pieces.len() {
        let left = pieces[..at].join(" = ");
        let right = pieces[at..].join(" = ");
        let width = visible_width(&left).max(visible_width(&right));
        if width < widest {
            widest = width;
            at_best = at;
        }
    }
    if widest > 150 {
        // Still too wide in two lines: fall back to one piece per line.
        let mut lines = vec![format!("{} &= {}", pieces[0], pieces[1])];
        for piece in &pieces[2..] {
            lines.push(format!("&= {piece}"));
        }
        let aligned = format!(
            "\\begin{{aligned}}{}\\end{{aligned}}",
            lines.join(" \\\\ ")
        );
        return format!("<div class=\"equation\">\\[{aligned}\\]</div>");
    }
    // The alignment point is the first equals, so the second line continues
    // under it. Putting it at the end would push the tail off the column.
    let first = &pieces[0];
    let head = pieces[1..at_best].join(" = ");
    let tail = pieces[at_best..].join(" = ");
    // About sixty visible characters fit a half-column at this size.
    let wide = if widest > 60 { " data-wide=\"1\"" } else { "" };
    format!(
        "<div class=\"equation\"{wide}>\\[\\begin{{aligned}}{first} &= {head} \\\\ &= {tail}\\end{{aligned}}\\]</div>"
    )
}

/// Apply the splitter to every display equation in a fragment, so whole
/// sections (nonuniform, tandem, factors, capacity) get the same two-line
/// treatment as the geometry grid.
pub fn split_long_equations(html: &str) -> String {
    let tag = "<div class=\"equation\">";
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find(tag) {
        let Some(end) = rest[start..].find("</div>") else {
            break;
        };
        let end = start + end + "</div>".len();
        out.push_str(&rest[..start]);
        out.push_str(&split_long_equation(&rest[start..end]));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

pub fn working_grid(blocks: &[String]) -> String {
    let mut out = String::from("<div class=\"working-wrap\"><table class=\"working\"><tbody>");
    let mut row: Vec<&String> = Vec::new();
    let flush = |row: &mut Vec<&String>, out: &mut String| {
        match row.as_slice() {
            [one, two] => out.push_str(&format!("<tr><td>{one}</td><td>{two}</td></tr>")),
            [one] => out.push_str(&format!("<tr><td colspan=\"2\">{one}</td></tr>")),
            _ => {}
        }
        row.clear();
    };
    for block in blocks {
        if block.contains("data-wide=\"1\"") {
            flush(&mut row, &mut out);
            out.push_str(&format!("<tr><td colspan=\"2\">{block}</td></tr>"));
        } else {
            row.push(block);
            if row.len() == 2 {
                flush(&mut row, &mut out);
            }
        }
    }
    flush(&mut row, &mut out);
    out.push_str("</tbody></table></div>");
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
    let required_value = if gross > 0.0 {
        fmt(Some(design_load_from_gross(gross)), 2, "kg")
    } else {
        "\u{2014}".to_string()
    };
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
            Some(UsageState::Within) => (
                "<span class=\"metric-band within\">WITHIN LIMIT</span>".to_string(),
                String::new(),
            ),
            Some(UsageState::Caution) => (
                "<span class=\"metric-band caution\">CAUTION</span>".to_string(),
                "past the 75% target".to_string(),
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
    let usage_sub = if usage_sub.is_empty() {
        String::new()
    } else {
        format!("<span class=\"metric-sub\">{}</span>", escape(&usage_sub))
    };
    let usage_cell = format!(
        "<td><span class=\"metric-k\">Usage at working radius</span>\
         <span class=\"metric-line\"><span class=\"metric-v\">{}</span>{}</span>{}</td>",
        escape(&usage_value),
        usage_band,
        usage_sub
    );

    format!(
        "<table class=\"metrics\"><tr>{}{}{}</tr><tr>{}{}{}</tr></table>",
        metric_cell("Gross load at hook", &fmt(Some(gross), 2, "kg"), ""),
        metric_cell_aside(
            "Chart capacity",
            &chart_value,
            "at this radius / boom angle",
        ),
        usage_cell,
        metric_cell("Required capacity at 75%", &required_value, ""),
        metric_cell("Crane rating", &booked_value, ""),
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
    diagram: Option<(&str, f64)>,
    lines: Option<String>,
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
    }
    // The diagram takes the left of the row. Its right side carries the two
    // workings first, then the line lists, top to bottom. When the column
    // cannot fit beside the diagram it wraps underneath as one block.
    let workings = crane
        .map(|result| crane_workings(result, gross))
        .unwrap_or_default();
    let aside = format!(
        "{workings}{}",
        lines.as_deref().map(plan_lines_block).unwrap_or_default()
    );
    match diagram {
        Some((source, scale)) => {
            out.push_str("<div class=\"plan-figure\">");
            out.push_str(&working_diagram_html(source, scale));
            if !aside.is_empty() {
                out.push_str(&format!("<div class=\"plan-aside\">{aside}</div>"));
            }
            out.push_str("</div>");
        }
        None => {
            if !workings.is_empty() {
                out.push_str(&format!("<div class=\"plan-aside\">{workings}</div>"));
            }
            if let Some(lines) = lines {
                out.push_str(&plan_lines_block(&lines));
            }
        }
    }

    out.push_str("</section>");
    out
}

fn working_diagram_html(source: &str, scale: f64) -> String {
    format!(
        "<div class=\"working-diagram\" style=\"width: {scale:.0}%;\">\
         <div class=\"diagram-title\">Working range diagram</div>\
         <img src=\"{}\" alt=\"Working range diagram\" style=\"width: 100%;\"></div>",
        escape(source)
    )
}

/// The line lists as a compact block: a small label like the plan's figures,
/// then the two lists. It sits beside the diagram when there is room.
fn plan_lines_block(lines: &str) -> String {
    format!(
        "<div class=\"plan-lines\"><div class=\"work-label\">\
         Load items and lifting tackle</div>{lines}</div>"
    )
}

/// The load and tackle lines as two side-by-side lists: the description and
/// its subtotal, loads on the left and tackle on the right. A line with no name
/// is a blank row the operator has not used yet, so it is left out.
fn plan_lines(load_items: &[LoadItem], tackle_items: &[TackleItem]) -> String {
    let loads: Vec<&LoadItem> = load_items
        .iter()
        .filter(|item| !item.name.trim().is_empty())
        .collect();
    let tackle: Vec<&TackleItem> = tackle_items
        .iter()
        .filter(|item| !item.description.trim().is_empty())
        .collect();
    if loads.is_empty() && tackle.is_empty() {
        return String::new();
    }

    let totals = LiftTotals::of(load_items, tackle_items);
    let multiple_lifts = totals.lifts.len() > 1;

    let column = |title: &str, rows: &[String]| -> String {
        if rows.is_empty() {
            return String::new();
        }
        format!(
            "<div class=\"line-pair-title\">{}</div>\
             <table class=\"line-pair-list\"><tbody>{}</tbody></table>",
            escape(title),
            rows.concat()
        )
    };

    let load_rows: Vec<String> = loads
        .iter()
        .map(|item| {
            let lift = if item.lift.trim().is_empty() {
                "Lift 1"
            } else {
                item.lift.trim()
            };
            let name = if multiple_lifts {
                format!("{lift} — {}", item.name.trim())
            } else {
                item.name.trim().to_string()
            };
            format!(
                "<tr><td>{}</td><td class=\"n\">{}</td></tr>",
                escape(&name),
                fmt(Some(item.weight()), 2, "kg")
            )
        })
        .collect();
    let tackle_rows: Vec<String> = tackle
        .iter()
        .map(|item| {
            format!(
                "<tr><td>{}</td><td class=\"n\">{}</td></tr>",
                escape(item.description.trim()),
                fmt(Some(item.subtotal()), 2, "kg")
            )
        })
        .collect();

    let mut out = format!(
        "<table class=\"line-pairs\"><tbody><tr><td>{}</td><td>{}</td></tr></tbody></table>",
        column("Load items", &load_rows),
        column("Lifting tackle", &tackle_rows)
    );
    if totals.lifts.len() > 1 {
        if let Some(lift) = totals
            .lifts
            .iter()
            .max_by(|a, b| a.total().total_cmp(&b.total()))
        {
            out.push_str(&format!(
                "<div class=\"line-note\">Overall weight uses {} ({}).</div>",
                escape(&lift.name),
                fmt(Some(lift.total()), 2, "kg")
            ));
        }
    }
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

/// The report as the print engines should receive it: the same document without
/// the `@page` margin boxes, because the export stamps the footer onto the pages
/// after printing. The live preview and the browser Print button keep them.
pub fn print_html(html: &str) -> String {
    html.replace(PAGE_FOOTER_CSS, "")
}


/// One report section's wrapper. Every section goes through here, so a future
/// pagination rule has one place to live.
fn section_card(body: &str, new_page: bool) -> String {
    let extra = if new_page { " new-page" } else { "" };
    format!("<section class=\"section-card{extra}\">{body}</section>")
}

/// Start a section on a fresh page, at the operator's request.
fn with_break(html: String, new_page: bool) -> String {
    if new_page {
        format!("<div class=\"new-page\">{html}</div>")
    } else {
        html
    }
}

/// The selected report sections as HTML.
///
/// Missing or incomplete state is rendered as an explicit setup message rather
/// than raising while the operator is still entering data.
pub fn build_report_html(inputs: &ReportInputs, include: &[String]) -> String {
    let selected = |name: &str| include.iter().any(|section| section == name);
    let breaks = |name: &str| inputs.page_breaks.iter().any(|section| section == name);
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
        .filter(|source| {
            source.starts_with("data:image/png;base64,")
                || source.starts_with("data:image/svg+xml;charset=utf-8,")
        });
    let (reference, footer_center) =
        footer_texts(&reference, inputs.footer_center.as_deref());
    // The report screen can hide the plan header and the line lists; every
    // other caller keeps them, so their absence needs an explicit marker.
    let plan_on = !include.iter().any(|section| section == "no-plan");
    let lines_on = !include.iter().any(|section| section == "no-lines");
    let mut parts: Vec<String> = vec![
        REPORT_CSS.to_string(),
        format!(
            "<style>:root {{ --plan-reference: \"{}\"; --footer-center: \"{}\"; }}</style>",
            css_string(&reference),
            css_string(&footer_center)
        ),
        "<div class=\"report\">".to_string(),
    ];
    // The diagram belongs to the plan details box. When the plan header is
    // hidden it still prints on its own, because its tick is independent.
    let diagram = working_diagram.map(|source| {
        let scale = inputs.diagram_scale.unwrap_or(100.0).clamp(25.0, 100.0);
        (source, scale)
    });
    // The line lists belong under the plan details too. A requested page break
    // still wins: the lists then print as their own card, where the break can
    // take effect.
    let lines = if lines_on {
        let html = plan_lines(&inputs.load_items, &inputs.tackle_items);
        (!html.is_empty()).then_some(html)
    } else {
        None
    };
    let lines_in_head = !breaks("lines");
    if plan_on {
        parts.push(plan_head(
            &inputs.signoff,
            inputs.issued_at.as_deref(),
            gross,
            crane_for_head,
            diagram,
            lines.as_ref().filter(|_| lines_in_head).cloned(),
        ));
    } else if let Some((source, scale)) = diagram {
        parts.push(working_diagram_html(source, scale));
    }
    if let Some(lines) = &lines {
        // Inside the plan box unless there is no plan box, or a page break
        // needs the lists to start their own section.
        if !plan_on || !lines_in_head {
            parts.push(section_card(
                &format!("<h2>LOAD ITEMS AND LIFTING TACKLE</h2>{lines}"),
                breaks("lines"),
            ));
        }
    }

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

    if selected("outrigger") {
        if let Some(outrigger) = &inputs.outrigger {
            parts.push(section_card(&outrigger_section(outrigger), breaks("outrigger")));
        } else if let Some(error) = &inputs.outrigger_error {
            parts.push(section_card(&format!(
                "{}<p>{}</p>",
                heading(2, "Outrigger Pad Size"),
                escape(&format!(
                    "{error} The outrigger pad is not part of this plan."
                ))
            ), breaks("outrigger")));
        }
    }

    if selected("uniform") {
        if let Some(sling) = &inputs.sling {
            if let Ok(spec) = hitch_spec(&sling.hitch) {
                parts.push(with_break(section_inputs(sling, spec), breaks("uniform")));
                parts.push(section_card(&geometry_section(sling, spec), false));
                parts.push(section_card(&factors_section(sling, spec), false));
                parts.push(section_card(&capacity_section(sling, spec), false));
                if shows_notes(spec) {
                    parts.push(section_card(&notes_section(sling, spec), false));
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
        let section = split_long_equations(&section);
        parts.push(with_break(section_card(&section, false), breaks("nonuniform")));
    }
    if selected("tandem") {
        let first = next_section_number(&parts);
        let section = nonuniform_sections(
            inputs.tandem.as_ref().map(|tandem| {
                NonuniformResult::Tandem(tandem.clone())
            }).as_ref(),
            first,
            "Tandem",
            inputs.tandem_error.as_deref(),
        )
        .join("");
        let section = split_long_equations(&section);
        parts.push(with_break(section_card(&section, false), breaks("tandem")));
    }

    if plan_on && gross <= 0.0 {
        parts.push(p(
            "<b>Setup note:</b> Add the load and complete the Overall Weight \
             tab before issuing this plan.",
            "border:2px solid #a33; padding:5pt 7pt; margin:5pt 0;",
        ));
    }

    if parts.len() == 3 {
        // Every section is unticked: an empty body prints no pages at all, and
        // the print backend then fails with "Invalid page range". One blank
        // line keeps the single, empty page.
        parts.push("&nbsp;".to_string());
    }

    parts.push("</div>".to_string());
    renumber_sections(&parts.join(""))
}

/// Number the report's sections in the order they actually appear. The section
/// builders carry their own numbers, which collide as soon as an optional
/// section — the outrigger pad, for instance — is inserted before them.
fn renumber_sections(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    let mut number = 0usize;
    while let Some(start) = rest.find("<h2>") {
        let after = start + 4;
        out.push_str(&rest[..after]);
        let tail = &rest[after..];
        let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && tail.as_bytes().get(digits) == Some(&b'.') {
            number += 1;
            out.push_str(&number.to_string());
            rest = &tail[digits..];
        } else {
            rest = tail;
        }
    }
    out.push_str(rest);
    out
}

/// The crane, its boom and the capacity check against the load chart.
fn crane_workings(crane: &CraneResult, gross: f64) -> String {
    let chart_capacity = crane.chart_capacity_kg;

    // The figures are printed in the plan head; what remains here is the working
    // that produced them. Required capacity comes first, then usage - the order
    // the right column reads.
    // W is a crane load-chart sizing load. It belongs to the crane check only
    // and is deliberately not used by the sling sections.
    let applied_load = design_load_from_gross(gross);
    let ratio = tex_num(Some(DESIGN_LOAD_RATIO), 2);
    let usage = crane.capacity_usage_percent;

    let required = format!(
        "<div class=\"plan-work\"><div class=\"work-label\">\
         Required capacity at 75%, W</div>{}</div>",
        equation(&format!(
            "\\text{{W}} = \\frac{{\\text{{Gross load at hook}}}}{{{ratio}}} = \
             \\frac{{{}}}{{{ratio}}} = {}\\ \\text{{kg}}",
            tex_num(Some(gross), 2),
            tex_num(Some(applied_load), 2)
        ))
    );
    let used = match (chart_capacity, usage) {
        (Some(capacity), Some(usage)) if capacity != 0.0 => {
            // Two lines at most: the side column is narrower than the full
            // working. The arithmetic is unchanged.
            let lines = [
                "\\text{Usage} &= \\frac{\\text{Gross load at hook}}\
                 {\\text{Chart capacity at radius}} \\times 100\\%"
                    .to_string(),
                format!(
                    "&= \\frac{{{}}}{{{}}} \\times 100\\% = {:.2}\\%",
                    tex_num(Some(gross), 2),
                    tex_num(Some(capacity), 2),
                    usage
                ),
            ];
            format!(
                "<div class=\"plan-work\"><div class=\"work-label\">\
                 Usage, % of chart capacity</div>{}</div>",
                equation(&format!(
                    "\\begin{{aligned}}{}\\end{{aligned}}",
                    lines.join(" \\\\ ")
                ))
            )
        }
        _ => "<div class=\"plan-work\"><div class=\"work-label\">\
              Usage, % of chart capacity</div>\
              <div class=\"work-empty\">Not evaluated</div></div>"
            .to_string(),
    };
    format!("{required}{used}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crane::{example_crane_inputs, solve_crane};
    use crate::model::{LoadItem, TackleItem};

    fn example_crane() -> CraneResult {
        solve_crane(&example_crane_inputs(), 2000.0).unwrap()
    }

    /// The alignment markup removed, so a formula split over lines still
    /// compares with the reference's one-line string.
    fn unaligned(html: &str) -> String {
        let mut out = html
            .replace("\\begin{aligned}", "")
            .replace("\\end{aligned}", "")
            .replace("\\\\", " ")
            .replace("&=", "=");
        while out.contains("  ") {
            out = out.replace("  ", " ");
        }
        out
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
        // The card reports the machine; it makes no check of its own.
        assert!(!html.contains("checked against 75%"));
        assert!(!html.contains("below the 75% rule"));
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
    fn the_rigging_factors_pair_up_two_to_a_row() {
        use crate::sling::{example_inputs, solve_one_leg};

        let report_for = |hitch: &str| {
            let inputs = ReportInputs {
                totals: LiftTotals::of(
                    &[LoadItem {
                        weight_kg: 5000.0,
                        ..LoadItem::default()
                    }],
                    &[] as &[TackleItem],
                ),
                sling: Some(solve_one_leg(&example_inputs(hitch).unwrap(), 5000.0).unwrap()),
                ..ReportInputs::default()
            };
            build_report_html(&inputs, &["uniform".to_string()])
        };

        // A direct hitch: the two factors share one row, the angle block's cell
        // followed straight by the reeve block's cell.
        let direct = report_for("two_leg_direct");
        assert!(direct.contains("<div class=\"work-label\">Sling Angle Factor (AF)</div>"));
        assert!(direct.contains(
            "</div></td><td><div class=\"work-label\">Base Reeve Factor (RF)</div>"
        ));

        // A choke adds the reduction and the combined factor: four, two rows.
        let choke = report_for("two_leg_round_choke");
        for label in [
            "Choke Angle Reduction Factor",
            "Combined Reeve Factor",
        ] {
            assert!(
                choke.contains(&format!("<div class=\"work-label\">{label}</div>")),
                "missing {label}"
            );
        }
        // A two-leg choke still has an angle; it must not claim to hang vertical.
        assert!(!choke.contains("stands perfectly vertical"));

        // A nested arrangement lists every step of its angle factor, and RF.
        let nested = report_for("two_leg_nested_3leg");
        assert!(nested.contains("<div class=\"work-label\">Base Reeve Factor (RF)</div>"));
        assert!(nested.contains("Longitudinal Sling Angle Factor (AF Long)"));
    }

    #[test]
    fn the_working_columns_follow_their_content() {
        // Auto layout: an even split is the aim, and the divider moves when one
        // side needs the room. A fixed split wrapped a long label beside a
        // short one.
        assert!(REPORT_CSS.contains("table-layout: auto"));
        assert!(!REPORT_CSS.contains("td:first-child { width:"));
        assert!(!REPORT_CSS.contains("td:last-child { width:"));
    }

    #[test]
    fn the_plan_head_carries_the_crane_figures() {
        let html = report(example_crane(), &["crane"]);
        for expected in [
            "<h1>Lifting Plan",
            "<span class=\"plan-crane\"> - Kobelco CKE90G-2</span>",
            "<span class=\"metric-k\">Chart capacity</span>\
             <span class=\"metric-aside\">at this radius / boom angle</span></span>\
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
        let html = unaligned(&report(example_crane(), &["crane"]));
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
    fn each_working_label_precedes_its_formula() {
        let html = unaligned(&report(example_crane(), &["crane"]));
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
             <span class=\"metric-aside\">at this radius / boom angle</span></span>\
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
    fn an_empty_plan_still_prints_one_blank_page() {
        let html = report(example_crane(), &["no-plan", "no-lines"]);
        assert!(!html.contains("<h1>Lifting Plan"));
        assert!(html.contains("&nbsp;"));
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
    fn the_page_margin_defaults_to_one_inch_and_clamps_what_it_is_given() {
        let one = PageMargins::default();
        let (width, height) = one.content_mm();
        assert!((width - 159.2).abs() < 0.01, "{width}");
        assert!((height - 246.2).abs() < 0.01, "{height}");

        let wild = PageMargins {
            top: -10.0,
            right: 999.0,
            bottom: f64::NAN,
            left: 12.0,
        };
        let clamped = wild.clamped();
        assert!((clamped.top - MIN_MARGIN_MM).abs() < 1e-9);
        assert!((clamped.right - MAX_MARGIN_MM).abs() < 1e-9);
        assert!((clamped.bottom - PRINT_MARGIN_MM).abs() < 1e-9);
        assert!((clamped.left - 12.0).abs() < 1e-9);
    }

    #[test]
    fn a_chosen_margin_reaches_the_page_that_is_printed() {
        let margins = PageMargins {
            top: 20.0,
            right: 15.0,
            bottom: 30.0,
            left: 15.0,
        };
        let html = wrap_katex_with("<p>body</p>", margins);
        assert!(
            html.contains("@page { size: A4; margin: 20mm 15mm 30mm 15mm; }"),
            "the four sides must reach the page"
        );
        // An evenly margined page keeps the plain shorthand.
        assert!(wrap_katex("<p>body</p>").contains("@page { size: A4; margin: 25.4mm; }"));
    }

    #[test]
    fn the_wrapper_loads_a_bundled_katex_and_nothing_remote() {
        let page = wrap_katex("<p>body</p>");
        for expected in [
            "<link rel=\"stylesheet\" href=\"vendor/katex/katex.min.css\">",
            "<script src=\"vendor/katex/katex.min.js\"></script>",
            "<script src=\"vendor/katex/auto-render.min.js\"></script>",
            "<script src=\"vendor/katex/lpc-autorender.js\"></script>",
            "@page { size: A4; margin: 25.4mm; }",
            "<p>body</p>",
        ] {
            assert!(page.contains(expected), "missing {expected}");
        }
        // The app runs under a CSP that allows scripts from its own origin only,
        // so the report must not carry an inline script.
        assert!(
            !page.contains("<script>"),
            "the report must not carry an inline script"
        );
    }

    #[test]
    fn the_auto_render_pass_is_a_file_that_sets_the_equations() {
        let script = include_str!("../../ui/vendor/katex/lpc-autorender.js");
        for expected in [
            "renderMathInElement",
            "throwOnError: false",
            "data-katex-error",
            "{ left: \"\\\\(\", right: \"\\\\)\", display: false }",
            "postMessage",
        ] {
            assert!(script.contains(expected), "missing {expected}");
        }
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
    fn section_numbers_follow_the_sections_that_are_printed() {
        let html = "<h2>1. PLAN DETAILS</h2><h2>2. OUTRIGGER PAD SIZE</h2>\
                    <h2>2. LIFT INPUT PARAMETERS</h2><h2>3. GEOMETRY</h2>\
                    <h2>6. FINAL CAPACITY</h2>";
        let numbered = renumber_sections(html);
        for expected in [
            "<h2>1. PLAN DETAILS</h2>",
            "<h2>2. OUTRIGGER PAD SIZE</h2>",
            "<h2>3. LIFT INPUT PARAMETERS</h2>",
            "<h2>4. GEOMETRY</h2>",
            "<h2>5. FINAL CAPACITY</h2>",
        ] {
            assert!(numbered.contains(expected), "{expected} missing from {numbered}");
        }
        // A heading without a number is left alone.
        assert_eq!(renumber_sections("<h2>NO NUMBER</h2>"), "<h2>NO NUMBER</h2>");
    }

    #[test]
    fn a_section_can_be_pushed_to_its_own_page() {
        assert!(section_card("<h2>x</h2>", true).contains("section-card new-page"));
        assert!(!section_card("<h2>x</h2>", false).contains("new-page"));
        assert_eq!(
            with_break("x".to_string(), true),
            "<div class=\"new-page\">x</div>"
        );
        assert_eq!(with_break("x".to_string(), false), "x");
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
    fn the_plan_accepts_an_svg_diagram() {
        let source = "data:image/svg+xml;charset=utf-8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%3E%3C%2Fsvg%3E";
        let inputs = ReportInputs {
            working_diagram: Some(source.to_string()),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &[]);
        assert!(html.contains("<img src=\"data:image/svg+xml;charset=utf-8,"));
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
        assert_eq!(
            report_sections_for("outrigger", true),
            vec!["crane", "outrigger"]
        );
        assert_eq!(
            report_sections_for("outrigger", false),
            vec!["outrigger"]
        );
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
    fn the_plan_lists_the_load_and_tackle_lines() {
        let inputs = ReportInputs {
            load_items: vec![LoadItem {
                name: "Steel plate".to_string(),
                qty: 2,
                weight_kg: 500.0,
                ..LoadItem::default()
            }],
            tackle_items: vec![TackleItem {
                description: "4-leg chain sling".to_string(),
                qty: 2,
                weight_kg: 25.0,
                ..TackleItem::default()
            }],
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &[]);
        assert!(html.contains("Load items"));
        assert!(html.contains("Steel plate"));
        assert!(html.contains("1,000.00 kg"));
        assert!(html.contains("4-leg chain sling"));
        assert!(html.contains("50.00 kg"));
    }

    #[test]
    fn the_plan_names_the_lift_that_sets_the_overall_weight() {
        let inputs = ReportInputs {
            load_items: vec![
                LoadItem {
                    name: "Steel plate".to_string(),
                    lift: "Steel".to_string(),
                    weight_kg: 800.0,
                    ..LoadItem::default()
                },
                LoadItem {
                    name: "Concrete".to_string(),
                    lift: "Concrete".to_string(),
                    weight_kg: 1500.0,
                    ..LoadItem::default()
                },
                LoadItem {
                    name: "Concrete bucket".to_string(),
                    lift: "Concrete".to_string(),
                    weight_kg: 80.0,
                    ..LoadItem::default()
                },
            ],
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &[]);
        assert!(html.contains("Steel — Steel plate"));
        assert!(html.contains("Concrete — Concrete"));
        assert!(html.contains("Overall weight uses Concrete (1,580.00 kg)."));
    }

    #[test]
    fn the_line_lists_follow_the_working_diagram_when_one_is_printed() {
        let inputs = ReportInputs {
            load_items: vec![LoadItem {
                name: "Steel plate".to_string(),
                weight_kg: 500.0,
                ..LoadItem::default()
            }],
            working_diagram: Some("data:image/png;base64,AAAA".to_string()),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &[]);
        assert!(html.find("Working range diagram") < html.find("Load items"));
        // The lists sit inside the plan details box as well.
        let head = html.find("<section class=\"plan-head\">").unwrap();
        let end = head + html[head..].find("</section>").unwrap();
        assert!(html[..end].contains("Load items"));
    }

    #[test]
    fn the_diagram_and_lines_share_a_row_while_they_fit() {
        let inputs = ReportInputs {
            load_items: vec![LoadItem {
                name: "Steel plate".to_string(),
                weight_kg: 500.0,
                ..LoadItem::default()
            }],
            working_diagram: Some("data:image/png;base64,AAAA".to_string()),
            diagram_scale: Some(50.0),
            crane: Some(example_crane()),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &["crane".to_string()]);
        let figure = html.find("class=\"plan-figure\"").unwrap();
        let diagram = html.find("class=\"working-diagram\"").unwrap();
        let required = html.find("Required capacity at 75%, W").unwrap();
        let usage = html.find("Usage, % of chart capacity").unwrap();
        let lines = html.find("class=\"plan-lines\"").unwrap();
        let end = figure + html[figure..].find("</section>").unwrap();
        assert!(diagram > figure && diagram < required, "diagram on the left");
        assert!(required < usage && usage < lines, "required, then usage, then the lists");
        assert!(lines < end, "the lists stay in the plan box");
    }

    #[test]
    fn a_page_break_for_the_lines_keeps_them_in_their_own_section() {
        let inputs = ReportInputs {
            load_items: vec![LoadItem {
                name: "Steel plate".to_string(),
                weight_kg: 500.0,
                ..LoadItem::default()
            }],
            page_breaks: vec!["lines".to_string()],
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &[]);
        let head = html.find("<section class=\"plan-head\">").unwrap();
        let end = head + html[head..].find("</section>").unwrap();
        assert!(!html[..end].contains("Load items"));
        assert!(html.contains("section-card new-page"));
    }

    #[test]
    fn the_diagram_scale_reaches_the_image() {
        let inputs = ReportInputs {
            working_diagram: Some("data:image/png;base64,AAAA".to_string()),
            diagram_scale: Some(50.0),
            ..ReportInputs::default()
        };
        let html = build_report_html(&inputs, &[]);
        assert!(html.contains("style=\"width: 50%;\""));
        // The diagram sits inside the plan details box, not beside it.
        let head = html.find("<section class=\"plan-head\">").unwrap();
        let end = head + html[head..].find("</section>").unwrap();
        let diagram = html.find("class=\"working-diagram\"").unwrap();
        assert!(diagram > head && diagram < end);
    }

    #[test]
    fn the_chart_capacity_note_sits_beside_the_label() {
        let html = report(example_crane(), &["crane"]);
        assert!(html.contains(
            "<span class=\"metric-head\"><span class=\"metric-k\">Chart capacity</span>"
        ));
        assert!(html.contains("metric-aside"));
    }

    #[test]
    fn the_print_copy_drops_the_margin_boxes_but_keeps_the_page() {
        let html = build_report_html(
            &ReportInputs {
                totals: LiftTotals { gross: 0.0, ..Default::default() },
                crane: None,
                outrigger: None,
                outrigger_error: None,
                sling: None,
                nonuniform: None,
                nonuniform_error: None,
                tandem: None,
                tandem_error: None,
                crane_error: None,
                load_items: Vec::new(),
                tackle_items: Vec::new(),
                signoff: PlanSignoff::default(),
                issued_at: None,
                working_diagram: None,
                diagram_scale: None,
                footer_center: None,
                page_breaks: Vec::new(),
            },
            &[],
        );
        assert!(html.contains("@bottom-center"), "the report itself keeps them");
        let print = print_html(&html);
        assert!(!print.contains("@bottom-center"), "the export must not draw a second footer");
        assert!(!print.contains("@bottom-left"));
        // The document is otherwise intact.
        assert!(print.contains(".report"));
        assert!(print.contains("--plan-reference"));
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
