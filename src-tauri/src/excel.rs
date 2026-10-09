//! Reading a crane load chart out of a workbook, and the rules that decide what
//! an imported chart does to the form. Ported from `Xlsx.kt`, `ExcelLoader.kt`
//! and `ExcelImport.kt`.
//!
//! Two common load-chart layouts are supported, and the layout is detected
//! rather than chosen: a **column format** (radius / load / boom columns under
//! a header row) and a **matrix format** (boom lengths across the top row,
//! working radius in the first column, capacity in the cells). Capacities in a
//! matrix layout are assumed to be metric tons when the values are small
//! (<= 2000) and kg otherwise, and are converted to kg internally.
//!
//! Two answers here are *decisions* rather than parsing, and both are the
//! desktop's own:
//!
//!  * **Which boom column a matrix chart is read from** — the *last* workbook
//!    entry (the longest boom), not `max()`.
//!  * **Which working radius an imported chart settles on** — the farthest
//!    charted radius whose capacity still meets `gross / 0.75`.

use std::collections::BTreeMap;
use std::fmt;
use std::io::Cursor;

use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};
use serde::{Deserialize, Serialize};

use crate::calc::{self, CalcException};
use crate::crane::{ChartRow, CraneInputs};
use crate::figures::fmt;

/// A workbook that could not be read, in words an operator can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelImportException(pub String);

impl fmt::Display for ExcelImportException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ExcelImportException {}

fn err(message: &str) -> ExcelImportException {
    ExcelImportException(message.to_string())
}

/// What `openpyxl` reports when the file is not a workbook at all.
const NOT_A_WORKBOOK: &str =
    "That file is not an Excel workbook. Load charts are read from .xlsx files; a \
     chart from an older .xls or .xlsm file has to be saved as .xlsx first.";

/// A ceiling on the workbook's size.
pub const MAX_CHART_BYTES: usize = 24 * 1024 * 1024;

/// The most rows any one sheet is read for, matching the source's `max_row`.
pub const DEFAULT_MAX_ROWS: usize = 2000;

// ---------------------------------------------------------------------------
// The in-memory sheet shape (a Python-shaped cell: text, number, or empty)
// ---------------------------------------------------------------------------

/// One cell, mirroring the `str` / numeric / `None` that `openpyxl` hands the
/// loader.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    Text(String),
    Number(f64),
}

/// One worksheet: its title, and its cells as rows.
#[derive(Debug, Clone, PartialEq)]
pub struct XlsxSheet {
    pub title: String,
    pub rows: Vec<Vec<Option<CellValue>>>,
}

fn map_data(value: &Data) -> Option<CellValue> {
    match value {
        Data::Int(number) => Some(CellValue::Number(*number as f64)),
        Data::Float(number) => Some(CellValue::Number(*number)),
        Data::Bool(flag) => Some(CellValue::Number(if *flag { 1.0 } else { 0.0 })),
        Data::String(text) => Some(CellValue::Text(text.clone())),
        Data::Error(_) | Data::Empty => None,
        _ => None,
    }
}

/// Read every sheet in the workbook, in the workbook's own order.
pub fn read_xlsx(bytes: &[u8], max_rows: usize) -> Result<Vec<XlsxSheet>, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs::<Xlsx<_>, _>(cursor)
        .map_err(|error| error.to_string())?;

    let names: Vec<String> = workbook
        .sheets_metadata()
        .iter()
        .map(|sheet| sheet.name.clone())
        .collect();

    let mut sheets = Vec::new();
    for name in names {
        let Ok(range) = workbook.worksheet_range(&name) else {
            continue;
        };
        let start_column = range.start().map(|(_, column)| column as usize).unwrap_or(0);
        let mut rows: Vec<Vec<Option<CellValue>>> = Vec::new();
        for row in range.rows() {
            if rows.len() >= max_rows {
                break;
            }
            let mut cells: Vec<Option<CellValue>> = vec![None; start_column];
            cells.extend(row.iter().map(map_data));
            rows.push(cells);
        }
        sheets.push(XlsxSheet { title: name, rows });
    }
    Ok(sheets)
}

// ---------------------------------------------------------------------------
// The parsed chart
// ---------------------------------------------------------------------------

/// One parsed row. A column-format row carries `load`; a matrix row carries
/// `capacities`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartDataRow {
    pub radius: Option<f64>,
    pub load: Option<f64>,
    pub boom: Option<f64>,
    pub capacities: Vec<Option<f64>>,
}

/// A parsed load chart table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadChartData {
    pub rows: Vec<ChartDataRow>,
    pub header_row: usize,
    pub column_map: BTreeMap<String, usize>,
    pub origin: String,
    pub matrix: bool,
    pub booms: Vec<f64>,
    pub unit: String,
}

impl LoadChartData {
    /// Rated capacity (kg) at a working radius, from the load column — or, for
    /// a matrix chart, the selected boom-length column.
    pub fn capacity_at(&self, radius: f64, boom_length: Option<f64>) -> Result<f64, CalcException> {
        let pairs: Vec<(f64, f64)> = if self.matrix {
            let index = self.boom_index(boom_length);
            self.rows
                .iter()
                .filter_map(|row| {
                    let radius = row.radius?;
                    let capacity = row.capacities.get(index).copied().flatten()?;
                    Some((radius, capacity))
                })
                .collect()
        } else {
            self.rows
                .iter()
                .filter_map(|row| {
                    let radius = row.radius?;
                    let load = row.load?;
                    Some((radius, load))
                })
                .collect()
        };
        calc::lookup_chart_capacity(&pairs, radius)
    }

    /// Which boom column to read, by nearest match.
    pub fn boom_index(&self, boom_length: Option<f64>) -> usize {
        if self.booms.is_empty() || boom_length.is_none() {
            return 0;
        }
        let wanted = boom_length.unwrap_or_default();
        let mut best = 0;
        let mut best_diff = f64::MAX;
        for (index, boom) in self.booms.iter().enumerate() {
            let diff = (boom - wanted).abs();
            if diff < best_diff {
                best = index;
                best_diff = diff;
            }
        }
        best
    }
}

// ---------------------------------------------------------------------------
// Parsing (ExcelLoader.kt)
// ---------------------------------------------------------------------------

/// Canonical name -> the column that holds it. Ordered; the first match wins.
const COLUMN_RULES: [(&str, &str); 13] = [
    ("radius", "radius"),
    ("workingradius", "radius"),
    ("r", "radius"),
    ("load", "load"),
    ("ratedload", "load"),
    ("capacit", "load"),
    // Kept verbatim: the value it is compared against has had every
    // non-alphanumeric character removed, so this entry can never match.
    ("safe working load", "load"),
    ("swl", "load"),
    ("boom", "boom"),
    ("boomlength", "boom"),
    ("boompositio", "boom"),
    ("mainsection", "boom"),
    ("length", "boom"),
];

/// Parse a chart out of the workbook's sheets, in order.
///
/// The first sheet that yields a chart wins. A sheet with nothing on it is
/// skipped rather than ending the search.
pub fn load_chart(
    sheets: &[XlsxSheet],
    column_map: Option<&BTreeMap<String, usize>>,
) -> Result<LoadChartData, CalcException> {
    for sheet in sheets {
        // The reference strips each string cell and turns an empty one into
        // None while it reads, then drops rows that are entirely empty.
        let rows: Vec<Vec<Option<CellValue>>> = sheet
            .rows
            .iter()
            .map(|row| -> Vec<Option<CellValue>> {
                row.iter()
                    .map(|cell| match cell {
                        Some(CellValue::Text(text)) => {
                            let trimmed = text.trim();
                            if trimmed.is_empty() {
                                None
                            } else {
                                Some(CellValue::Text(trimmed.to_string()))
                            }
                        }
                        other => other.clone(),
                    })
                    .collect()
            })
            .filter(|row| row.iter().any(|cell| cell.is_some()))
            .collect();
        if rows.is_empty() {
            continue;
        }

        if let Some(mapping) = column_map {
            if let Some(chart) = parse_column_rows(&rows, 0, mapping, &sheet.title) {
                return Ok(chart);
            }
            continue;
        }

        let (header_index, mapping) = detect_header_row(&rows, 40);
        if mapping.contains_key("radius") && mapping.contains_key("load") {
            if let Some(chart) =
                parse_column_rows(&rows, header_index.unwrap_or(0), &mapping, &sheet.title)
            {
                return Ok(chart);
            }
        }

        let (matrix_row, booms) = detect_matrix(&rows, 12);
        if let Some(matrix_row) = matrix_row {
            if let Some(chart) = parse_matrix_rows(&rows, matrix_row, &booms, &sheet.title) {
                return Ok(chart);
            }
        }
    }
    Err(CalcException(
        "Could not find a load chart in this workbook. Make sure it has \
         either a header row with radius and load columns, or a boom \
         length row across the top with radii in the first column."
            .to_string(),
    ))
}

/// The first row that looks like a header, and the columns it names.
pub fn detect_header_row(
    rows: &[Vec<Option<CellValue>>],
    max_rows: usize,
) -> (Option<usize>, BTreeMap<String, usize>) {
    for index in 0..rows.len().min(max_rows) {
        let mut mapping: BTreeMap<String, usize> = BTreeMap::new();
        for (column, cell) in rows[index].iter().enumerate() {
            if cell.is_none() || text(cell).trim().is_empty() {
                continue;
            }
            let Some(canonical) = match_header(cell) else {
                continue;
            };
            if !mapping.contains_key(canonical) {
                mapping.insert(canonical.to_string(), column);
            }
        }
        if mapping.len() >= 2 && (mapping.contains_key("radius") || mapping.contains_key("load")) {
            return (Some(index), mapping);
        }
    }
    (None, BTreeMap::new())
}

/// Detect a boom-lengths-across-the-top matrix chart.
pub fn detect_matrix(
    rows: &[Vec<Option<CellValue>>],
    max_rows: usize,
) -> (Option<usize>, Vec<f64>) {
    for index in 0..rows.len().min(max_rows) {
        let row = &rows[index];
        if row.is_empty() {
            continue;
        }
        let Some(first) = row.first() else {
            continue;
        };
        if first.is_none() {
            continue;
        }
        let norm = normalise(first);
        if !(!norm.is_empty() && (norm.contains("radius") || norm == "r")) {
            continue;
        }

        let mut booms: Vec<f64> = Vec::new();
        for cell in row.iter().skip(1) {
            let Some(value) = to_double_or_null(cell) else {
                break;
            };
            if value <= 0.0 {
                break;
            }
            booms.push(value);
        }
        if booms.len() < 2 {
            continue;
        }

        let next = rows.get(index + 1);
        if let Some(next) = next {
            if !next.is_empty() && to_double_or_null(&next[0]).is_some() {
                return (Some(index), booms);
            }
        }
    }
    (None, Vec::new())
}

/// Capacity cells <= 2000 are almost certainly metric tons.
pub fn detect_unit(capacities: &[f64]) -> (String, f64) {
    let largest = capacities.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let largest = if capacities.is_empty() { 0.0 } else { largest };
    if largest <= 2000.0 {
        ("ton".to_string(), 1000.0)
    } else {
        ("kg".to_string(), 1.0)
    }
}

fn parse_column_rows(
    rows: &[Vec<Option<CellValue>>],
    header_row: usize,
    mapping: &BTreeMap<String, usize>,
    title: &str,
) -> Option<LoadChartData> {
    let radius_column = *mapping.get("radius")?;
    let load_column = *mapping.get("load")?;
    let boom_column = mapping.get("boom").copied();

    let mut load_unit = 1.0;
    let norm = normalise(&header_text(rows, header_row, load_column));
    if norm.contains("ton") || norm == "t" || norm == "mt" {
        load_unit = 1000.0;
    }

    let mut data_rows: Vec<ChartDataRow> = Vec::new();
    for row in rows.iter().skip(header_row + 1) {
        let Some(radius) = row.get(radius_column).and_then(to_double_or_null) else {
            continue;
        };
        let Some(load) = row.get(load_column).and_then(to_double_or_null) else {
            continue;
        };
        let boom = match boom_column {
            Some(column) if column < row.len() => {
                row.get(column).and_then(to_double_or_null)
            }
            _ => None,
        };
        data_rows.push(ChartDataRow {
            radius: Some(radius),
            load: Some(load * load_unit),
            boom,
            capacities: Vec::new(),
        });
    }
    if data_rows.is_empty() {
        return None;
    }
    Some(LoadChartData {
        rows: data_rows,
        header_row,
        column_map: mapping.clone(),
        origin: format!("{title}.xlsx / {title} sheet / header row {}", header_row + 1),
        matrix: false,
        booms: Vec::new(),
        unit: "kg".to_string(),
    })
}

fn parse_matrix_rows(
    rows: &[Vec<Option<CellValue>>],
    header_row: usize,
    booms: &[f64],
    title: &str,
) -> Option<LoadChartData> {
    let mut data_rows: Vec<ChartDataRow> = Vec::new();
    let mut cell_values: Vec<f64> = Vec::new();

    for row in rows.iter().skip(header_row + 1) {
        let Some(radius) = row.first().and_then(to_double_or_null) else {
            continue;
        };
        let mut capacities: Vec<Option<f64>> = Vec::new();
        for (index, _) in booms.iter().enumerate() {
            let column = index + 1;
            let value = if column < row.len() {
                to_double_or_null(&row[column])
            } else {
                None
            };
            capacities.push(value);
            if let Some(value) = value {
                cell_values.push(value);
            }
        }
        data_rows.push(ChartDataRow {
            radius: Some(radius),
            load: None,
            boom: None,
            capacities,
        });
    }
    if data_rows.is_empty() {
        return None;
    }

    let (unit, multiplier) = detect_unit(&cell_values);
    let scaled: Vec<ChartDataRow> = data_rows
        .into_iter()
        .map(|row| ChartDataRow {
            capacities: row
                .capacities
                .into_iter()
                .map(|value| value.map(|value| value * multiplier))
                .collect(),
            ..row
        })
        .collect();
    Some(LoadChartData {
        rows: scaled,
        header_row,
        column_map: BTreeMap::new(),
        origin: format!(
            "{title}.xlsx / {title} sheet / boom length row {} / unit {unit}",
            header_row + 1
        ),
        matrix: true,
        booms: booms.to_vec(),
        unit,
    })
}

fn header_text(rows: &[Vec<Option<CellValue>>], header_row: usize, column: usize) -> Option<CellValue> {
    rows.get(header_row).and_then(|row| row.get(column)).cloned().flatten()
}

/// Lowercase, then every non-alphanumeric character removed.
pub fn normalise(value: &Option<CellValue>) -> String {
    text(value)
        .trim()
        .to_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

/// Whether a heading names one of the canonical columns, and which.
pub fn match_header(value: &Option<CellValue>) -> Option<&'static str> {
    let norm = normalise(value);
    if norm.is_empty() {
        return None;
    }
    for (matcher, canonical) in COLUMN_RULES {
        if norm.contains(matcher) || matcher.contains(&norm) {
            return Some(canonical);
        }
    }
    None
}

/// A cell as a number, or None.
///
/// A comma-grouped figure is un-grouped first, because a chart pasted out of a
/// formatted report carries `1,250` rather than `1250`.
pub fn to_double_or_null(value: &Option<CellValue>) -> Option<f64> {
    match value {
        None => None,
        Some(CellValue::Number(number)) => Some(*number),
        Some(CellValue::Text(text)) => {
            let cleaned = text.trim().replace(',', "");
            if cleaned.is_empty() {
                None
            } else {
                cleaned.parse::<f64>().ok()
            }
        }
    }
}

/// A cell as text, the way Python's `str()` would render it.
fn text(value: &Option<CellValue>) -> String {
    match value {
        None => String::new(),
        Some(CellValue::Text(text)) => text.clone(),
        Some(CellValue::Number(number)) => {
            // A whole number is rendered without a decimal point so a heading
            // like "9" normalises the same way.
            if number.is_finite() && *number == number.floor() {
                format!("{}", *number as i64)
            } else {
                format!("{number}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// What a chart does to the form (ExcelImport.kt)
// ---------------------------------------------------------------------------

/// Parse a workbook already in memory.
pub fn read_chart(bytes: &[u8]) -> Result<LoadChartData, ExcelImportException> {
    if bytes.is_empty() {
        return Err(err("That file was empty."));
    }
    if bytes.len() > MAX_CHART_BYTES {
        return Err(err(&format!(
            "That file is {} MB, too large to be a load chart. Load charts are small tables.",
            bytes.len() / 1048576
        )));
    }
    if !bytes.starts_with(b"PK") {
        return Err(err(NOT_A_WORKBOOK));
    }
    let sheets = match read_xlsx(bytes, DEFAULT_MAX_ROWS) {
        Ok(sheets) => sheets,
        Err(message) => {
            if message.to_lowercase().contains("zip") || message.to_lowercase().contains("central directory") {
                return Err(err(NOT_A_WORKBOOK));
            }
            return Err(err(&format!("That workbook could not be read: {message}")));
        }
    };
    load_chart(&sheets, None).map_err(|exc| ExcelImportException(exc.0))
}

/// The boom lengths the operator may switch between, in workbook order.
pub fn available_booms(chart: &LoadChartData) -> Vec<f64> {
    if !chart.matrix || chart.booms.is_empty() {
        return Vec::new();
    }
    chart.booms.clone()
}

/// Which boom column the chart is read from.
///
/// Given none, **the desktop's default, which is the last entry and not
/// `max()`**.
pub fn selected_boom_index(chart: &LoadChartData, boom: Option<f64>) -> usize {
    if !chart.matrix || chart.booms.is_empty() {
        return 0;
    }
    let wanted = boom.unwrap_or_else(|| *chart.booms.last().unwrap());
    chart.boom_index(Some(wanted))
}

/// The boom length the chart is read from, or None for a column-format chart.
pub fn selected_boom_length(chart: &LoadChartData, boom: Option<f64>) -> Option<f64> {
    if !chart.matrix || chart.booms.is_empty() {
        return None;
    }
    chart.booms.get(selected_boom_index(chart, boom)).copied()
}

/// The radii the operator may pick at this boom, ascending and deduplicated.
pub fn chart_radii(chart: &LoadChartData, boom: Option<f64>) -> Vec<f64> {
    let mut found: Vec<f64> = Vec::new();
    if chart.matrix {
        let index = selected_boom_index(chart, boom);
        for row in &chart.rows {
            if let Some(radius) = row.radius {
                if row.capacities.get(index).copied().flatten().is_some() {
                    found.push(radius);
                }
            }
        }
    } else {
        for row in &chart.rows {
            if let Some(radius) = row.radius {
                if row.load.is_some() {
                    found.push(radius);
                }
            }
        }
    }
    found.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    found.dedup();
    found
}

/// The charted points at `boom`, as `(radius, capacity, boom or None)`.
pub fn usable_rows(
    chart: &LoadChartData,
    boom: Option<f64>,
) -> Vec<(f64, f64, Option<f64>)> {
    let index = selected_boom_index(chart, boom);
    let boom_out = selected_boom_length(chart, boom);
    let mut out = Vec::new();
    if chart.matrix {
        for row in &chart.rows {
            let Some(radius) = row.radius else {
                continue;
            };
            let Some(capacity) = row.capacities.get(index).copied().flatten() else {
                continue;
            };
            out.push((radius, capacity, boom_out));
        }
    } else {
        for row in &chart.rows {
            let Some(radius) = row.radius else {
                continue;
            };
            let Some(load) = row.load else {
                continue;
            };
            out.push((radius, load, row.boom));
        }
    }
    out
}

/// The chart at `boom` as `(radius, capacity)` pairs, in workbook order.
pub fn chart_points(
    chart: &LoadChartData,
    boom: Option<f64>,
) -> Result<Vec<(f64, f64)>, ExcelImportException> {
    let points: Vec<(f64, f64)> = usable_rows(chart, boom)
        .into_iter()
        .filter(|(radius, capacity, _)| *radius > 0.0 && *capacity > 0.0)
        .map(|(radius, capacity, _)| (radius, capacity))
        .collect();
    if points.is_empty() {
        return Err(err(
            "No charted capacities were found in that workbook. It has the \
             shape of a load chart but every row is empty.",
        ));
    }
    Ok(points)
}

/// The working radius the desktop would settle on for this lift, or None.
///
/// The lift needs `gross / 0.75` of rated capacity, and the radius picked is
/// the **farthest** one whose capacity still meets it.
pub fn suggest_radius(chart: &LoadChartData, gross: f64, boom: Option<f64>) -> Option<f64> {
    if gross <= 0.0 {
        return None;
    }
    let required = calc::required_chart_capacity(gross, calc::CRANE_USAGE_RATIO).ok()?;
    let items: Vec<(Option<f64>, Option<f64>)> = if chart.matrix {
        let index = selected_boom_index(chart, boom);
        chart
            .rows
            .iter()
            .map(|row| (row.radius, row.capacities.get(index).copied().flatten()))
            .collect()
    } else {
        chart.rows.iter().map(|row| (row.radius, row.load)).collect()
    };
    let mut feasible: Vec<f64> = items
        .into_iter()
        .filter_map(|(radius, capacity)| {
            let radius = radius?;
            let capacity = capacity?;
            if capacity >= required {
                Some(radius)
            } else {
                None
            }
        })
        .collect();
    feasible.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    feasible.last().copied()
}

/// The charted capacity at `radius`, or None where the desktop has none.
pub fn capacity_at(chart: &LoadChartData, radius: f64, boom: Option<f64>) -> Option<f64> {
    if radius <= 0.0 {
        return None;
    }
    let points = chart_points(chart, boom).ok()?;
    calc::lookup_chart_capacity(&points, radius).ok()
}

/// The boom of the workbook row nearest `radius`, or None.
pub fn nearest_row_boom(chart: &LoadChartData, radius: f64) -> Option<f64> {
    let mut best: Option<f64> = None;
    let mut best_diff = f64::MAX;
    for row in &chart.rows {
        let Some(boom) = row.boom else {
            continue;
        };
        let Some(row_radius) = row.radius else {
            continue;
        };
        let diff = (row_radius - radius).abs();
        if diff < best_diff {
            best = Some(boom);
            best_diff = diff;
        }
    }
    best
}

/// What a workbook writes down into the one manual entry, or None.
///
/// The desktop's *"mirror the current Excel selection (radius, rated load,
/// boom length) into the manual entry so both load-chart views stay in sync"* —
/// three fields, and **not the boom angle**.
pub fn mirrored_point(
    chart: &LoadChartData,
    radius: f64,
    boom: Option<f64>,
) -> Option<(f64, f64, Option<f64>)> {
    if radius <= 0.0 {
        return None;
    }
    let column = if chart.matrix { boom } else { None };
    let capacity = capacity_at(chart, radius, column)?;
    if capacity <= 0.0 {
        return None;
    }
    let out_boom = if chart.matrix {
        boom
    } else {
        nearest_row_boom(chart, radius)
    };
    Some((radius, capacity, out_boom))
}

/// Which boom column a matrix chart is read from, or empty for a column chart.
///
/// **Recomputed on every boom change**: a label still reading "31 m" over a
/// 27.5 m chart is worse than no label.
pub fn matrix_note(chart: &LoadChartData, boom: Option<f64>) -> String {
    match selected_boom_length(chart, boom) {
        Some(selected) => format!(" \u{2014} read from boom {} m", trimmed(selected)),
        None => String::new(),
    }
}

/// The line shown under the chart, in the desktop's own words.
pub fn chart_label(chart: &LoadChartData, name: &str, boom: Option<f64>) -> String {
    let unit = if chart.unit.is_empty() {
        "kg".to_string()
    } else {
        chart.unit.clone()
    };
    format!(
        "{name} (capacity in {unit}){}",
        matrix_note(chart, boom)
    )
}

/// A whole number without its trailing zeros, the way Python's `%g` renders it.
fn trimmed(value: f64) -> String {
    let formatted = format!("{value}");
    if formatted.contains('.') {
        formatted.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        formatted
    }
}

/// The inputs after importing `chart`.
pub fn imported_inputs(
    inputs: &CraneInputs,
    chart: &LoadChartData,
    name: &str,
    gross: f64,
) -> CraneInputs {
    let booms = available_booms(chart);
    let boom = booms.last().copied();
    applied(inputs, chart, name, boom, gross)
}

/// The inputs after the operator picks a different boom.
pub fn boom_changed(
    inputs: &CraneInputs,
    chart: &LoadChartData,
    boom: f64,
    gross: f64,
) -> CraneInputs {
    applied(inputs, chart, &inputs.chart_name, Some(boom), gross)
}

fn applied(
    inputs: &CraneInputs,
    chart: &LoadChartData,
    name: &str,
    boom: Option<f64>,
    gross: f64,
) -> CraneInputs {
    let suggested = suggest_radius(chart, gross, boom);
    // A declined suggestion leaves the radius where it was.
    let radius = suggested.unwrap_or(inputs.working_radius);
    let points = chart_points(chart, boom).unwrap_or_default();
    let mirrored = mirrored_point(chart, radius, boom);
    let rows = mirror_into(&inputs.rows, mirrored);

    CraneInputs {
        rows,
        chart_points: points,
        chart_source: "excel".to_string(),
        chart_name: name.to_string(),
        chart_label: chart_label(chart, name, boom),
        chart_booms: available_booms(chart),
        chart_boom: boom,
        chart_radii: chart_radii(chart, boom),
        working_radius: radius,
        ..inputs.clone()
    }
}

/// The inputs after the operator picks a different working radius.
///
/// **The mirror follows, and that is the whole point of it.** The suggested
/// radius is deliberately **not** re-asked here.
pub fn radius_changed(inputs: &CraneInputs, chart: &LoadChartData, radius: f64) -> CraneInputs {
    let mirrored = mirrored_point(chart, radius, inputs.chart_boom);
    CraneInputs {
        working_radius: radius,
        rows: mirror_into(&inputs.rows, mirrored),
        ..inputs.clone()
    }
}

/// Write a mirrored point into the one manual entry.
fn mirror_into(
    rows: &[ChartRow],
    mirrored: Option<(f64, f64, Option<f64>)>,
) -> Vec<ChartRow> {
    let Some((radius, capacity, boom)) = mirrored else {
        return rows.to_vec();
    };
    let entry = rows.first().copied().unwrap_or_default();
    vec![ChartRow {
        radius,
        capacity,
        boom: boom.unwrap_or(entry.boom),
        ..entry
    }]
}

/// The inputs after the operator types in the manual entry.
///
/// **This drops the workbook entirely — boom and all.** — the narrowing the
/// Android build states deliberately. The working radius then follows the
/// edited row through [`crate::crane::manual_radius`].
pub fn forget_chart(inputs: &CraneInputs) -> CraneInputs {
    CraneInputs {
        rows: vec![inputs.rows.first().copied().unwrap_or_default()],
        chart_points: Vec::new(),
        chart_source: String::new(),
        chart_name: String::new(),
        chart_label: String::new(),
        chart_booms: Vec::new(),
        chart_boom: None,
        chart_radii: Vec::new(),
        ..inputs.clone()
    }
}

/// The inputs after the operator edits the one manual entry row.
///
/// The workbook is dropped, the row becomes the only row, and the working
/// radius follows the row when the rule says it should (a single complete row
/// always moves it; several complete rows only fill an unset radius).
pub fn manual_edit(inputs: &CraneInputs, row: ChartRow) -> CraneInputs {
    let forgotten = forget_chart(inputs);
    let rows = vec![row];
    let radius = crate::crane::manual_radius(&rows, inputs.working_radius)
        .unwrap_or(inputs.working_radius);
    CraneInputs {
        rows,
        working_radius: radius,
        ..forgotten
    }
}

/// The workbook table the Crane tab shows under the chart controls.
///
/// `(headers, rows)` of already-formatted strings; the matrix layout lists the
/// column read at `boom` (which is column 0 when no boom is named — the same
/// reading `capacity_at` takes), the column layout lists its own three columns.
pub fn chart_table(chart: &LoadChartData, boom: Option<f64>) -> (Vec<String>, Vec<Vec<String>>) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    if chart.matrix {
        for row in &chart.rows {
            let Some(radius) = row.radius else {
                continue;
            };
            if radius <= 0.0 {
                continue;
            }
            let Some(capacity) = chart.capacity_at(radius, boom).ok() else {
                continue;
            };
            if capacity > 0.0 {
                rows.push(vec![
                    fmt(Some(radius), 2, "m"),
                    fmt(Some(capacity), 2, "kg"),
                ]);
            }
        }
        (vec!["Radius".to_string(), "Capacity".to_string()], rows)
    } else {
        for row in &chart.rows {
            let Some(radius) = row.radius else {
                continue;
            };
            let Some(load) = row.load else {
                continue;
            };
            if radius > 0.0 && load > 0.0 {
                rows.push(vec![
                    fmt(Some(radius), 2, "m"),
                    fmt(Some(load), 2, "kg"),
                    fmt(row.boom, 2, "m"),
                ]);
            }
        }
        (
            vec![
                "Radius".to_string(),
                "Load".to_string(),
                "Boom".to_string(),
            ],
            rows,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::write::SimpleFileOptions;

    use super::*;

    #[derive(Clone, Copy)]
    enum FixtureCell {
        S(&'static str),
        N(f64),
    }

    use FixtureCell::{N, S};

    fn column_letters(mut index: usize) -> String {
        let mut out = String::new();
        loop {
            out.insert(0, (b'A' + (index % 26) as u8) as char);
            if index < 26 {
                break;
            }
            index = index / 26 - 1;
        }
        out
    }

    fn build_workbook(sheet_name: &str, rows: &[Vec<FixtureCell>]) -> Vec<u8> {
        let mut shared: Vec<String> = Vec::new();
        let mut sheet_rows = String::new();
        for (row_index, row) in rows.iter().enumerate() {
            sheet_rows.push_str(&format!("<row r=\"{}\">", row_index + 1));
            for (column, cell) in row.iter().enumerate() {
                let reference = format!("{}{}", column_letters(column), row_index + 1);
                match cell {
                    S(value) => {
                        let index = shared.iter().position(|item| item == value).unwrap_or_else(|| {
                            shared.push(value.to_string());
                            shared.len() - 1
                        });
                        sheet_rows.push_str(&format!(
                            "<c r=\"{reference}\" t=\"s\"><v>{index}</v></c>"
                        ));
                    }
                    N(value) => {
                        sheet_rows.push_str(&format!(
                            "<c r=\"{reference}\"><v>{value}</v></c>"
                        ));
                    }
                }
            }
            sheet_rows.push_str("</row>");
        }
        let shared_items: String = shared
            .iter()
            .map(|item| format!("<si><t>{item}</t></si>"))
            .collect();

        let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
<Default Extension=\"xml\" ContentType=\"application/xml\"/>\
<Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>\
<Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>\
<Override PartName=\"/xl/sharedStrings.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml\"/>\
</Types>";
        let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>\
</Relationships>";
        let workbook = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
<sheets><sheet name=\"{sheet_name}\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>"
        );
        let workbook_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/>\
</Relationships>";
        let sheet = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">\
<sheetData>{sheet_rows}</sheetData></worksheet>"
        );
        let shared_strings = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<sst xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" count=\"{}\" uniqueCount=\"{}\">\
{shared_items}</sst>",
            shared.len(),
            shared.len()
        );

        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();
        let mut put = |name: &str, content: &str| {
            writer.start_file(name, options).unwrap();
            writer.write_all(content.as_bytes()).unwrap();
        };
        put("[Content_Types].xml", content_types);
        put("_rels/.rels", rels);
        put("xl/workbook.xml", &workbook);
        put("xl/_rels/workbook.xml.rels", workbook_rels);
        put("xl/worksheets/sheet1.xml", &sheet);
        put("xl/sharedStrings.xml", &shared_strings);
        writer.finish().unwrap().into_inner()
    }

    fn matrix_rows() -> Vec<Vec<FixtureCell>> {
        vec![
            vec![S("Radius"), N(9.8), N(13.3), N(16.9), N(20.4), N(23.9), N(27.5), N(31.0)],
            vec![N(3.0), N(20.0), N(17.5), N(14.5), N(12.0), N(10.0), N(8.5), N(7.0)],
            vec![N(5.0), N(18.0), N(15.0), N(12.5), N(10.5), N(8.5), N(7.0), N(5.5)],
            vec![N(10.0), N(12.0), N(10.0), N(8.0), N(6.5), N(5.0), N(3.5), N(2.5)],
            vec![N(16.0), N(8.0), N(6.5), N(5.0), N(4.0), N(3.0), N(2.0), N(1.5)],
        ]
    }

    fn column_rows() -> Vec<Vec<FixtureCell>> {
        vec![
            vec![S("Radius"), S("Load"), S("Boom Length")],
            vec![N(5.0), N(10000.0), N(20.0)],
            vec![N(10.0), N(5000.0), N(20.0)],
            vec![N(15.0), N(2500.0), N(20.0)],
        ]
    }

    fn descending_rows() -> Vec<Vec<FixtureCell>> {
        vec![
            vec![S("Radius"), N(31.0), N(27.5), N(9.8)],
            vec![N(10.0), N(2500.0), N(3500.0), N(12000.0)],
            vec![N(16.0), N(1500.0), N(2000.0), N(8000.0)],
        ]
    }

    fn matrix_chart() -> LoadChartData {
        read_chart(&build_workbook("Chart", &matrix_rows())).unwrap()
    }

    fn column_chart() -> LoadChartData {
        read_chart(&build_workbook("Chart", &column_rows())).unwrap()
    }

    #[test]
    fn a_matrix_workbook_parses_to_the_references_own_table() {
        let chart = matrix_chart();
        assert!(chart.matrix);
        assert_eq!(
            chart.booms,
            vec![9.8, 13.3, 16.9, 20.4, 23.9, 27.5, 31.0]
        );
        assert_eq!(chart.unit, "ton");
        assert_eq!(chart.origin, "Chart.xlsx / Chart sheet / boom length row 1 / unit ton");
        assert_eq!(chart.rows.len(), 4);
        assert!((chart.rows[0].radius.unwrap() - 3.0).abs() < 1e-9);
        let capacities: Vec<f64> = chart.rows[0]
            .capacities
            .iter()
            .map(|value| value.unwrap())
            .collect();
        assert_eq!(
            capacities,
            vec![
                20000.0, 17500.0, 14500.0, 12000.0, 10000.0, 8500.0, 7000.0
            ]
        );
        assert!((chart.rows[3].radius.unwrap() - 16.0).abs() < 1e-9);
        let last: Vec<f64> = chart.rows[3]
            .capacities
            .iter()
            .map(|value| value.unwrap())
            .collect();
        assert_eq!(
            last,
            vec![8000.0, 6500.0, 5000.0, 4000.0, 3000.0, 2000.0, 1500.0]
        );
    }

    #[test]
    fn a_matrix_workbook_is_looked_up_at_the_named_boom() {
        let chart = matrix_chart();
        assert!((chart.capacity_at(10.0, Some(27.5)).unwrap() - 3500.0).abs() < 1e-9);
        assert!((chart.capacity_at(10.0, Some(31.0)).unwrap() - 2500.0).abs() < 1e-9);
    }

    #[test]
    fn a_column_workbook_parses_to_the_references_own_table() {
        let chart = column_chart();
        assert!(!chart.matrix);
        assert!(chart.booms.is_empty());
        assert_eq!(chart.unit, "kg");
        assert_eq!(chart.origin, "Chart.xlsx / Chart sheet / header row 1");
        assert_eq!(chart.rows.len(), 3);
        assert!((chart.rows[0].radius.unwrap() - 5.0).abs() < 1e-9);
        assert!((chart.rows[0].load.unwrap() - 10000.0).abs() < 1e-9);
        assert!((chart.rows[0].boom.unwrap() - 20.0).abs() < 1e-9);
        assert!((chart.capacity_at(10.0, None).unwrap() - 5000.0).abs() < 1e-9);
    }

    #[test]
    fn a_file_that_is_not_a_workbook_is_refused_in_words_an_operator_can_act_on() {
        assert_eq!(read_chart(&[]).unwrap_err().0, "That file was empty.");
        let error = read_chart(b"this is not a spreadsheet").unwrap_err();
        assert!(error.0.contains("not an Excel workbook"), "{error}");
    }

    #[test]
    fn the_boom_column_defaults_to_the_last_one_and_not_the_greatest() {
        let chart = read_chart(&build_workbook("Chart", &descending_rows())).unwrap();
        assert_eq!(available_booms(&chart), vec![31.0, 27.5, 9.8]);
        assert_eq!(selected_boom_index(&chart, None), 2);
        assert!((selected_boom_length(&chart, None).unwrap() - 9.8).abs() < 1e-9);
    }

    #[test]
    fn a_column_workbook_has_no_boom_to_select() {
        let chart = column_chart();
        assert!(available_booms(&chart).is_empty());
        assert_eq!(selected_boom_length(&chart, None), None);
        assert_eq!(selected_boom_index(&chart, None), 0);
    }

    #[test]
    fn the_offered_radii_are_the_ones_this_boom_has_a_capacity_for() {
        let chart = matrix_chart();
        assert_eq!(chart_radii(&chart, Some(31.0)), vec![3.0, 5.0, 10.0, 16.0]);
        assert_eq!(chart_radii(&chart, Some(27.5)), vec![3.0, 5.0, 10.0, 16.0]);
        assert_eq!(chart_radii(&column_chart(), None), vec![5.0, 10.0, 15.0]);
    }

    #[test]
    fn the_suggested_radius_is_the_farthest_that_still_carries_the_lift() {
        let chart = matrix_chart();
        assert!((suggest_radius(&chart, 2000.0, None).unwrap() - 5.0).abs() < 1e-9);
        assert!((suggest_radius(&chart, 2000.0, Some(27.5)).unwrap() - 10.0).abs() < 1e-9);
        assert!((suggest_radius(&chart, 2000.0, Some(9.8)).unwrap() - 16.0).abs() < 1e-9);
    }

    #[test]
    fn a_declined_suggestion_is_null_rather_than_zero() {
        let chart = matrix_chart();
        assert_eq!(suggest_radius(&chart, 0.0, None), None);
        assert_eq!(suggest_radius(&chart, 50000.0, None), None);
    }

    #[test]
    fn the_offered_radius_for_a_column_workbook_comes_from_its_own_rows() {
        assert!((suggest_radius(&column_chart(), 2000.0, None).unwrap() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn the_mirrored_entry_carries_three_fields_and_never_the_angle() {
        let matrix = matrix_chart();
        let point = mirrored_point(&matrix, 5.0, None).unwrap();
        assert!((point.0 - 5.0).abs() < 1e-9);
        assert!((point.1 - 5500.0).abs() < 1e-9);
        assert_eq!(point.2, None);
        let column = column_chart();
        let point = mirrored_point(&column, 10.0, None).unwrap();
        assert!((point.0 - 10.0).abs() < 1e-9);
        assert!((point.1 - 5000.0).abs() < 1e-9);
        assert!((point.2.unwrap() - 20.0).abs() < 1e-9);
        assert_eq!(mirrored_point(&matrix, 0.0, None), None);
    }

    #[test]
    fn the_label_names_the_file_the_unit_and_the_boom() {
        assert_eq!(
            chart_label(&matrix_chart(), "Chart.xlsx", None),
            "Chart.xlsx (capacity in ton) \u{2014} read from boom 31 m"
        );
        assert_eq!(
            chart_label(&matrix_chart(), "Chart.xlsx", Some(27.5)),
            "Chart.xlsx (capacity in ton) \u{2014} read from boom 27.5 m"
        );
        assert_eq!(
            chart_label(&column_chart(), "column.xlsx", None),
            "column.xlsx (capacity in kg)"
        );
    }

    #[test]
    fn importing_chooses_the_desktop_boom_and_the_desktop_radius() {
        let chart = matrix_chart();
        let inputs = CraneInputs {
            working_radius: 8.0,
            ..CraneInputs::default()
        };
        let imported = imported_inputs(&inputs, &chart, "Chart.xlsx", 2000.0);
        assert_eq!(imported.chart_source, "excel");
        assert!((imported.chart_boom.unwrap() - 31.0).abs() < 1e-9);
        assert_eq!(imported.chart_radii, vec![3.0, 5.0, 10.0, 16.0]);
        assert!((imported.working_radius - 5.0).abs() < 1e-9);
        assert_eq!(
            imported.chart_points,
            vec![(3.0, 7000.0), (5.0, 5500.0), (10.0, 2500.0), (16.0, 1500.0)]
        );
        assert_eq!(imported.rows.len(), 1);
        assert!((imported.rows[0].radius - 5.0).abs() < 1e-9);
        assert!((imported.rows[0].capacity - 5500.0).abs() < 1e-9);
    }

    #[test]
    fn switching_boom_re_asks_the_radius_and_the_figures() {
        let chart = matrix_chart();
        let imported = imported_inputs(&CraneInputs::default(), &chart, "Chart.xlsx", 2000.0);
        let switched = boom_changed(&imported, &chart, 27.5, 2000.0);
        assert!((switched.chart_boom.unwrap() - 27.5).abs() < 1e-9);
        assert!((switched.working_radius - 10.0).abs() < 1e-9);
        assert_eq!(
            switched.chart_points,
            vec![(3.0, 8500.0), (5.0, 7000.0), (10.0, 3500.0), (16.0, 2000.0)]
        );
        assert_eq!(
            switched.chart_label,
            "Chart.xlsx (capacity in ton) \u{2014} read from boom 27.5 m"
        );
    }

    #[test]
    fn typing_in_the_entry_drops_the_workbook_entirely() {
        let chart = matrix_chart();
        let imported = imported_inputs(&CraneInputs::default(), &chart, "Chart.xlsx", 2000.0);
        let typed = forget_chart(&imported);
        assert!(typed.chart_points.is_empty());
        assert_eq!(typed.chart_source, "");
        assert!(typed.chart_booms.is_empty());
        assert_eq!(typed.chart_boom, None);
        assert!(typed.chart_radii.is_empty());
        assert_eq!(typed.rows.len(), 1);
    }

    #[test]
    fn changing_the_working_radius_writes_through_to_the_entry() {
        let chart = matrix_chart();
        let with_angle = CraneInputs {
            rows: vec![ChartRow {
                angle: 48.0,
                ..ChartRow::default()
            }],
            ..CraneInputs::default()
        };
        let imported = imported_inputs(&with_angle, &chart, "Chart.xlsx", 2000.0);
        let changed = radius_changed(&imported, &chart, 3.0);
        assert!((changed.working_radius - 3.0).abs() < 1e-9);
        assert!((changed.rows[0].radius - 3.0).abs() < 1e-9);
        assert!((changed.rows[0].capacity - 7000.0).abs() < 1e-9);
        assert!((changed.rows[0].boom - 31.0).abs() < 1e-9);
        assert!((changed.rows[0].angle - 48.0).abs() < 1e-9);
    }

    #[test]
    fn a_radius_change_re_looks_up_the_capacity_rather_than_keeping_the_old_one() {
        let chart = matrix_chart();
        let imported = imported_inputs(&CraneInputs::default(), &chart, "Chart.xlsx", 2000.0);
        assert!((radius_changed(&imported, &chart, 3.0).rows[0].capacity - 7000.0).abs() < 1e-9);
        assert!((radius_changed(&imported, &chart, 16.0).rows[0].capacity - 1500.0).abs() < 1e-9);
        assert!((radius_changed(&imported, &chart, 7.0).rows[0].capacity - 4300.0).abs() < 1e-9);
        let at_short_boom = boom_changed(&imported, &chart, 27.5, 2000.0);
        assert!(
            (radius_changed(&at_short_boom, &chart, 3.0).rows[0].capacity - 8500.0).abs() < 1e-9
        );
    }

    #[test]
    fn a_radius_with_no_chartable_capacity_leaves_the_entry_alone() {
        let chart = matrix_chart();
        let imported = imported_inputs(&CraneInputs::default(), &chart, "Chart.xlsx", 2000.0);
        let unchanged = radius_changed(&imported, &chart, 0.0);
        assert!((unchanged.rows[0].radius - imported.rows[0].radius).abs() < 1e-9);
        assert!((unchanged.rows[0].capacity - imported.rows[0].capacity).abs() < 1e-9);
    }

    #[test]
    fn a_matrix_workbook_lists_the_column_the_app_is_reading() {
        let (headers, rows) = chart_table(&matrix_chart(), Some(31.0));
        assert_eq!(headers, vec!["Radius", "Capacity"]);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0], vec!["3.00 m", "7,000.00 kg"]);
        assert_eq!(rows[1], vec!["5.00 m", "5,500.00 kg"]);
        assert_eq!(rows[2], vec!["10.00 m", "2,500.00 kg"]);
        assert_eq!(rows[3], vec!["16.00 m", "1,500.00 kg"]);
    }

    #[test]
    fn the_listed_column_follows_the_boom_selector() {
        let chart = matrix_chart();
        let (_, rows) = chart_table(&chart, Some(27.5));
        assert_eq!(rows[0], vec!["3.00 m", "8,500.00 kg"]);
        assert_eq!(rows[1], vec!["5.00 m", "7,000.00 kg"]);
        assert_eq!(rows[2], vec!["10.00 m", "3,500.00 kg"]);
        assert_eq!(rows[3], vec!["16.00 m", "2,000.00 kg"]);
        for asked in [Some(9.8), Some(20.4), Some(31.0), None] {
            let (_, listed) = chart_table(&chart, asked);
            for (index, radius) in [3.0, 5.0, 10.0, 16.0].iter().enumerate() {
                let expected = fmt(Some(chart.capacity_at(*radius, asked).unwrap()), 2, "kg");
                assert_eq!(listed[index][1], expected, "boom {asked:?} at {radius}");
            }
        }
    }

    #[test]
    fn a_column_workbook_lists_its_own_three_columns_including_the_boom() {
        let (headers, rows) = chart_table(&column_chart(), None);
        assert_eq!(headers, vec!["Radius", "Load", "Boom"]);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec!["5.00 m", "10,000.00 kg", "20.00 m"]);
        assert_eq!(rows[1], vec!["10.00 m", "5,000.00 kg", "20.00 m"]);
        assert_eq!(rows[2], vec!["15.00 m", "2,500.00 kg", "20.00 m"]);
    }
}
