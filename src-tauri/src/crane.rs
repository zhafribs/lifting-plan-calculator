//! The Crane tab's inputs and its calculation, ported from `Crane.kt`.
//!
//! **Two charts are looked up and one wins.** `rows` is the *manual entry* —
//! one charted point. `chart_points` is a workbook's own points, empty for a
//! chart typed by hand. They are looked up separately and the winner is chosen
//! by the source's own rule, not by whichever was filled in last.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::calc::{self, CalcException, UsageState, CRANE_USAGE_RATIO};
use crate::figures::{self, fmt};

/// A crane input that cannot be solved, in words an operator can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraneInputException(pub String);

impl fmt::Display for CraneInputException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CraneInputException {}

impl From<CalcException> for CraneInputException {
    fn from(value: CalcException) -> Self {
        CraneInputException(value.0)
    }
}

/// The manual chart row's own columns, in the source's order and with its own
/// captions: radius (m), rated load (kg), boom angle (deg), boom length (m).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartColumn {
    pub key: &'static str,
    pub caption: &'static str,
    pub unit: &'static str,
    pub decimals: usize,
    pub min: f64,
    pub max: f64,
}

pub const COLUMN_RADIUS: ChartColumn = ChartColumn {
    key: "radius",
    caption: "Radius",
    unit: "m",
    decimals: 2,
    min: 0.0,
    max: 200.0,
};
pub const COLUMN_CAPACITY: ChartColumn = ChartColumn {
    key: "capacity",
    caption: "Rated Load",
    unit: "kg",
    decimals: 0,
    min: 0.0,
    max: 1.0e7,
};
pub const COLUMN_ANGLE: ChartColumn = ChartColumn {
    key: "angle",
    caption: "Boom Angle",
    unit: "deg",
    decimals: 1,
    min: 0.0,
    max: 90.0,
};
pub const COLUMN_BOOM: ChartColumn = ChartColumn {
    key: "boom",
    caption: "Boom Length",
    unit: "m",
    decimals: 2,
    min: 0.0,
    max: 200.0,
};

pub const CHART_COLUMNS: [ChartColumn; 4] = [
    COLUMN_RADIUS,
    COLUMN_CAPACITY,
    COLUMN_ANGLE,
    COLUMN_BOOM,
];

/// One line of the load chart entered by hand.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChartRow {
    pub radius: f64,
    pub capacity: f64,
    pub angle: f64,
    pub boom: f64,
}

impl Default for ChartRow {
    fn default() -> Self {
        Self {
            radius: 0.0,
            capacity: 0.0,
            angle: 0.0,
            boom: 0.0,
        }
    }
}

impl ChartRow {
    /// `(radius, capacity)` for the lookup, or None for a row not in use.
    ///
    /// A row is in use when both a radius and a capacity are entered. A
    /// half-filled row is not an error and not a zero-capacity chart point: it
    /// is a line the operator has not finished, and treating it as a charted
    /// value at 0 kg would fail the whole lift for a row still being typed.
    pub fn pair(&self) -> Option<(f64, f64)> {
        if self.radius > 0.0 && self.capacity > 0.0 {
            Some((self.radius, self.capacity))
        } else {
            None
        }
    }
}

/// The tab's whole input surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CraneInputs {
    pub name: String,
    /// The crane's booked rating in tonnes. Published for the plan's header
    /// rather than used in the usage: the usage is taken against the *load
    /// chart* figure at the working radius.
    pub capacity_t: f64,
    /// The manual entry: one row.
    pub rows: Vec<ChartRow>,
    /// A workbook's charted points at `chart_boom`. Empty for a hand-typed chart.
    pub chart_points: Vec<(f64, f64)>,
    pub working_radius: f64,
    /// `""` for a chart typed by hand, `"excel"` for one read from a workbook.
    pub chart_source: String,
    /// The workbook's display name as the picker reported it, or empty.
    pub chart_name: String,
    /// The composed label the report prints: the name, the unit the capacities
    /// are in, and which boom column was read.
    pub chart_label: String,
    /// The boom lengths the operator may switch between, in workbook order.
    pub chart_booms: Vec<f64>,
    pub chart_boom: Option<f64>,
    /// The radii charted at `chart_boom`, ascending: what the radius dropdown
    /// offers.
    pub chart_radii: Vec<f64>,
}

impl Default for CraneInputs {
    fn default() -> Self {
        Self {
            name: String::new(),
            capacity_t: 0.0,
            rows: vec![ChartRow::default()],
            chart_points: Vec::new(),
            working_radius: 0.0,
            chart_source: String::new(),
            chart_name: String::new(),
            chart_label: String::new(),
            chart_booms: Vec::new(),
            chart_boom: None,
            chart_radii: Vec::new(),
        }
    }
}

impl CraneInputs {
    /// The manual entry's `(radius, capacity)` points, in row order.
    pub fn chart_pairs(&self) -> Vec<(f64, f64)> {
        self.rows.iter().filter_map(ChartRow::pair).collect()
    }

    pub fn filled_rows(&self) -> Vec<ChartRow> {
        self.rows
            .iter()
            .copied()
            .filter(|row| row.pair().is_some())
            .collect()
    }
}

/// The working radius that follows the manual entry, or None to leave it alone.
///
/// Without this the operator types a radius of 12 m into the entry and is shown
/// the capacity at the 16 m the dropdown was left on. That figure *agrees with
/// the dropdown*, so it looks right — which is the whole of the problem.
///
/// A row with a radius but no capacity is not counted: a half-typed entry is a
/// line still being entered, not a radius to move the whole tab to.
pub fn manual_radius(rows: &[ChartRow], current: f64) -> Option<f64> {
    let valid: Vec<f64> = rows
        .iter()
        .filter(|row| row.radius > 0.0 && row.capacity > 0.0)
        .map(|row| row.radius)
        .collect();
    if valid.is_empty() {
        return None;
    }
    if valid.len() == 1 {
        return Some(valid[0]);
    }
    if current <= 0.0 {
        Some(valid[0])
    } else {
        None
    }
}

/// A worked lift: a 90 t crane with one charted point, read at 8.00 m.
pub fn example_crane_inputs() -> CraneInputs {
    CraneInputs {
        name: "Kobelco CKE90G-2".to_string(),
        capacity_t: 90.0,
        rows: vec![ChartRow {
            radius: 10.0,
            capacity: 16000.0,
            angle: 48.0,
            boom: 40.0,
        }],
        working_radius: 8.0,
        ..CraneInputs::default()
    }
}

/// The crane's capacity check at the working radius.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CraneResult {
    pub crane_name: String,
    pub crane_capacity_t: f64,
    pub crane_capacity_kg: f64,
    pub radius_m: f64,
    pub chart_loaded: bool,
    pub chart_path: Option<String>,
    pub chart_capacity_kg: Option<f64>,
    pub capacity_usage_percent: Option<f64>,
    pub capacity_source: String,
    pub capacity_status: String,
    pub capacity_band: Option<UsageState>,
    pub capacity_colour: String,
    pub boom_m: Option<f64>,
    pub boom_angle_deg: Option<f64>,
    pub chart_rows: usize,
}

/// The band colours, as the source publishes them for the report to reuse.
pub const COLOUR_WITHIN: &str = "#2e7d52";
pub const COLOUR_CAUTION: &str = "#b06a1e";
pub const COLOUR_OVER: &str = "#b23b3b";
/// Body colour for a figure that is not a judgement, as the desktop's neutral.
pub const COLOUR_NEUTRAL: &str = "#2a3347";

/// The crane's capacity check at the working radius.
///
/// The winner of the two charts is chosen by the source's own rule:
/// `use_manual = has_manual and (not from_excel or no_workbook_points) and
/// manual_capacity is not None`. Here `chart_source` *is* the flag — `"excel"`
/// means the workbook is in charge and the manual entry is a mirror of it,
/// anything else means the operator's own line is the chart.
pub fn solve_crane(inputs: &CraneInputs, gross: f64) -> Result<CraneResult, CraneInputException> {
    if gross < 0.0 {
        return Err(CraneInputException(
            "The lift weight cannot be negative.".to_string(),
        ));
    }

    let capacity_kg = if inputs.capacity_t > 0.0 {
        inputs.capacity_t * 1000.0
    } else {
        0.0
    };
    let radius = inputs.working_radius;

    // The workbook at the working radius.
    let mut chart_capacity: Option<f64> = None;
    if !inputs.chart_points.is_empty() && radius > 0.0 {
        chart_capacity = Some(
            calc::lookup_chart_capacity(&inputs.chart_points, radius)
                .map_err(|exc| CraneInputException(exc.0))?,
        );
    }

    let manual_pairs = inputs.chart_pairs();
    // The radius looked up at: the working radius, or the manual entry's own
    // first radius when there is none.
    let effective_radius = if radius > 0.0 {
        radius
    } else {
        manual_pairs.first().map(|pair| pair.0).unwrap_or(0.0)
    };

    let mut manual_capacity: Option<f64> = None;
    if !manual_pairs.is_empty() && effective_radius > 0.0 {
        manual_capacity = Some(
            calc::lookup_chart_capacity(&manual_pairs, effective_radius)
                .map_err(|exc| CraneInputException(exc.0))?,
        );
    }

    let use_manual = !manual_pairs.is_empty()
        && (inputs.chart_source != "excel" || inputs.chart_points.is_empty())
        && manual_capacity.is_some();
    if use_manual {
        chart_capacity = manual_capacity;
    }

    let usage = match (chart_capacity, gross > 0.0) {
        (Some(capacity), true) => Some(
            calc::chart_usage_percent(gross, capacity).map_err(|exc| CraneInputException(exc.0))?,
        ),
        _ => None,
    };

    let mut band: Option<UsageState> = None;
    let mut status = "Enter the load chart to check capacity.".to_string();
    let mut colour = COLOUR_NEUTRAL.to_string();
    if let Some(usage) = usage {
        let state = calc::crane_usage_band(usage);
        band = Some(state);
        status = format!(
            "{} \u{2014} {}% of capacity at working radius",
            state.wording(),
            fmt(Some(usage), 2, "")
        );
        colour = match state {
            UsageState::Within => COLOUR_WITHIN,
            UsageState::Caution => COLOUR_CAUTION,
            UsageState::Over => COLOUR_OVER,
        }
        .to_string();
    } else if chart_capacity.is_some() && gross <= 0.0 {
        status = "Enter the lift weight to check capacity.".to_string();
    }

    Ok(CraneResult {
        crane_name: inputs.name.clone(),
        crane_capacity_t: inputs.capacity_t,
        crane_capacity_kg: capacity_kg,
        radius_m: effective_radius,
        // True whenever *either* chart holds a point, so "loaded" keeps meaning
        // "there is something to look up" rather than "from Excel".
        chart_loaded: !manual_pairs.is_empty() || !inputs.chart_points.is_empty(),
        chart_path: if inputs.chart_label.is_empty() {
            None
        } else {
            Some(inputs.chart_label.clone())
        },
        chart_capacity_kg: chart_capacity,
        capacity_usage_percent: usage,
        capacity_source: if use_manual {
            "Manual chart".to_string()
        } else if inputs.chart_source == "excel" && chart_capacity.is_some() {
            "Excel chart".to_string()
        } else {
            String::new()
        },
        capacity_status: status,
        capacity_band: band,
        capacity_colour: colour,
        // The boom follows the same rule the capacity does: whichever chart is
        // in charge supplies it. Both the angle and the boom are read from the
        // manual row, which is the phone's deliberate divergence from the older
        // desktop build.
        boom_m: if use_manual {
            manual_boom(effective_radius, &inputs.rows)
        } else {
            inputs.chart_boom
        },
        boom_angle_deg: manual_angle(effective_radius, &inputs.rows),
        chart_rows: if use_manual {
            manual_pairs.len()
        } else {
            inputs.chart_points.len()
        },
    })
}

/// The boom angle of the manual row nearest `radius`.
///
/// Zero is "not entered", not "0 degrees", and a report row reading `0.0 deg`
/// beside a blank angle box would be the app inventing a figure.
pub fn manual_angle(radius: f64, rows: &[ChartRow]) -> Option<f64> {
    let usable: Vec<&ChartRow> = rows.iter().filter(|row| row.pair().is_some()).collect();
    if usable.is_empty() || radius <= 0.0 {
        return None;
    }
    let nearest = usable
        .iter()
        .min_by(|a, b| {
            (a.radius - radius)
                .abs()
                .partial_cmp(&(b.radius - radius).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
    if nearest.angle != 0.0 {
        Some(nearest.angle)
    } else {
        None
    }
}

/// The boom length of the manual row nearest `radius`, or None when none is set.
pub fn manual_boom(radius: f64, rows: &[ChartRow]) -> Option<f64> {
    let usable: Vec<&ChartRow> = rows.iter().filter(|row| row.pair().is_some()).collect();
    if usable.is_empty() || radius <= 0.0 {
        return None;
    }
    let nearest = usable
        .iter()
        .min_by(|a, b| {
            (a.radius - radius)
                .abs()
                .partial_cmp(&(b.radius - radius).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
    if nearest.boom != 0.0 {
        Some(nearest.boom)
    } else {
        None
    }
}

/// The crane design load ratio, exposed for the report.
pub const DESIGN_LOAD_RATIO: f64 = CRANE_USAGE_RATIO;

#[allow(dead_code)]
fn unused_figures_marker() {
    // Keeps the figures import obviously used in the module header; the
    // compiler folds this away.
    let _ = figures::NOTHING;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_worked_example_publishes_the_references_own_figures() {
        let result = solve_crane(&example_crane_inputs(), 2000.0).unwrap();
        assert_eq!(result.crane_name, "Kobelco CKE90G-2");
        assert!((result.crane_capacity_t - 90.0).abs() < 1e-9);
        assert!((result.crane_capacity_kg - 90000.0).abs() < 1e-9);
        assert!((result.radius_m - 8.0).abs() < 1e-9);
        assert!(result.chart_loaded);
        assert_eq!(result.chart_path, None);
        assert!((result.chart_capacity_kg.unwrap() - 16000.0).abs() < 1e-9);
        assert!((result.capacity_usage_percent.unwrap() - 12.5).abs() < 1e-9);
        assert_eq!(result.capacity_source, "Manual chart");
        assert_eq!(
            result.capacity_status,
            "WITHIN LIMIT \u{2014} 12.50% of capacity at working radius"
        );
        assert_eq!(result.capacity_band, Some(UsageState::Within));
        assert_eq!(result.capacity_colour, "#2e7d52");
        assert!((result.boom_m.unwrap() - 40.0).abs() < 1e-9);
        assert!((result.boom_angle_deg.unwrap() - 48.0).abs() < 1e-9);
        assert_eq!(result.chart_rows, 1);
    }

    #[test]
    fn a_workbook_is_interpolated_and_wins_when_the_operator_has_not_typed_in_the_entry() {
        let inputs = CraneInputs {
            chart_points: vec![(5.0, 10000.0), (10.0, 5000.0)],
            working_radius: 7.5,
            chart_source: "excel".to_string(),
            ..CraneInputs::default()
        };
        let result = solve_crane(&inputs, 2000.0).unwrap();
        assert!((result.chart_capacity_kg.unwrap() - 7500.0).abs() < 1e-9);
        assert!((result.capacity_usage_percent.unwrap() - 26.666666666666668).abs() < 1e-9);
        assert_eq!(result.capacity_source, "Excel chart");
        assert_eq!(result.capacity_band, Some(UsageState::Within));
        assert_eq!(result.chart_rows, 2);
    }

    #[test]
    fn excel_only_wins_when_a_capacity_was_actually_found_at_the_radius() {
        let inputs = CraneInputs {
            rows: vec![ChartRow {
                radius: 12.0,
                capacity: 8000.0,
                ..ChartRow::default()
            }],
            chart_points: vec![(5.0, 10000.0), (10.0, 5000.0)],
            working_radius: 7.5,
            chart_source: "excel".to_string(),
            ..CraneInputs::default()
        };
        let result = solve_crane(&inputs, 2000.0).unwrap();
        assert_eq!(result.capacity_source, "Excel chart");
        assert!((result.chart_capacity_kg.unwrap() - 7500.0).abs() < 1e-9);
    }

    #[test]
    fn the_manual_entry_is_the_chart_when_no_workbook_is_in_charge() {
        let inputs = CraneInputs {
            rows: vec![ChartRow {
                radius: 12.0,
                capacity: 8000.0,
                ..ChartRow::default()
            }],
            chart_points: vec![(5.0, 10000.0), (10.0, 5000.0)],
            working_radius: 7.5,
            ..CraneInputs::default()
        };
        let result = solve_crane(&inputs, 2000.0).unwrap();
        assert_eq!(result.capacity_source, "Manual chart");
        assert!((result.chart_capacity_kg.unwrap() - 8000.0).abs() < 1e-9);
        assert_eq!(result.chart_rows, 1);
    }

    #[test]
    fn a_negative_lift_weight_is_refused_with_the_references_wording() {
        assert_eq!(
            solve_crane(&example_crane_inputs(), -1.0).unwrap_err().0,
            "The lift weight cannot be negative."
        );
    }

    #[test]
    fn an_empty_chart_reports_that_there_is_nothing_to_check_against() {
        let result = solve_crane(&CraneInputs::default(), 2000.0).unwrap();
        assert!(!result.chart_loaded);
        assert_eq!(result.chart_capacity_kg, None);
        assert_eq!(result.capacity_usage_percent, None);
        assert_eq!(result.capacity_status, "Enter the load chart to check capacity.");
    }

    #[test]
    fn the_working_radius_follows_a_single_complete_entry_and_ignores_a_half_typed_one() {
        let rows = vec![ChartRow {
            radius: 12.0,
            capacity: 8000.0,
            ..ChartRow::default()
        }];
        assert!((manual_radius(&rows, 0.0).unwrap() - 12.0).abs() < 1e-9);
        assert_eq!(
            manual_radius(
                &[ChartRow {
                    radius: 12.0,
                    ..ChartRow::default()
                }],
                0.0
            ),
            None
        );
        assert_eq!(manual_radius(&[ChartRow::default()], 0.0), None);
    }

    #[test]
    fn a_zero_boom_angle_is_not_entered_rather_than_zero_degrees() {
        assert_eq!(
            manual_angle(
                10.0,
                &[ChartRow {
                    radius: 10.0,
                    capacity: 100.0,
                    ..ChartRow::default()
                }]
            ),
            None
        );
    }

    #[test]
    fn manual_radius_only_fills_an_unset_current() {
        let rows = vec![
            ChartRow {
                radius: 10.0,
                capacity: 5000.0,
                ..ChartRow::default()
            },
            ChartRow {
                radius: 16.0,
                capacity: 3000.0,
                ..ChartRow::default()
            },
        ];
        assert_eq!(manual_radius(&rows, 0.0), Some(10.0));
        assert_eq!(manual_radius(&rows, 14.0), None);
    }

    #[test]
    fn a_hand_typed_line_carries_all_four_of_its_columns_into_the_result() {
        let inputs = CraneInputs {
            name: "tadano".to_string(),
            capacity_t: 20.0,
            rows: vec![ChartRow {
                radius: 10.0,
                capacity: 1600.0,
                angle: 48.0,
                boom: 31.0,
            }],
            working_radius: 10.0,
            ..CraneInputs::default()
        };
        let result = solve_crane(&inputs, 500.0).unwrap();
        assert!((result.boom_m.unwrap() - 31.0).abs() < 1e-9);
        assert!((result.boom_angle_deg.unwrap() - 48.0).abs() < 1e-9);
        assert!((result.chart_capacity_kg.unwrap() - 1600.0).abs() < 1e-9);
        assert!((result.radius_m - 10.0).abs() < 1e-9);
        assert_eq!(result.capacity_source, "Manual chart");
    }

    #[test]
    fn the_boom_is_looked_up_at_the_same_radius_the_rest_of_the_row_is_read_at() {
        let inputs = CraneInputs {
            rows: vec![
                ChartRow {
                    radius: 6.0,
                    capacity: 9000.0,
                    boom: 20.0,
                    ..ChartRow::default()
                },
                ChartRow {
                    radius: 10.0,
                    capacity: 1600.0,
                    boom: 31.0,
                    ..ChartRow::default()
                },
            ],
            working_radius: 0.0,
            ..CraneInputs::default()
        };
        let result = solve_crane(&inputs, 500.0).unwrap();
        assert!((result.radius_m - 6.0).abs() < 1e-9);
        assert!((result.boom_m.unwrap() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn a_workbook_supplies_the_boom_when_it_is_the_chart_in_charge() {
        let inputs = CraneInputs {
            name: "Kobelco CKE90G-2".to_string(),
            capacity_t: 90.0,
            rows: vec![ChartRow {
                radius: 10.0,
                capacity: 16000.0,
                boom: 40.0,
                ..ChartRow::default()
            }],
            working_radius: 10.0,
            chart_source: "excel".to_string(),
            chart_points: vec![(10.0, 16000.0)],
            chart_boom: Some(40.0),
            ..CraneInputs::default()
        };
        let result = solve_crane(&inputs, 2000.0).unwrap();
        assert!((result.boom_m.unwrap() - 40.0).abs() < 1e-9);
        assert_eq!(result.capacity_source, "Excel chart");
    }

    #[test]
    fn an_unset_boom_is_absent_rather_than_zero_metres() {
        let inputs = CraneInputs {
            rows: vec![ChartRow {
                radius: 10.0,
                capacity: 1600.0,
                angle: 48.0,
                ..ChartRow::default()
            }],
            working_radius: 10.0,
            ..CraneInputs::default()
        };
        let result = solve_crane(&inputs, 500.0).unwrap();
        assert_eq!(result.boom_m, None);
        assert!((result.boom_angle_deg.unwrap() - 48.0).abs() < 1e-9);
    }

    #[test]
    fn negative_radius_on_the_chart_is_named() {
        let inputs = CraneInputs {
            chart_points: vec![(10.0, 0.0)],
            working_radius: 5.0,
            ..CraneInputs::default()
        };
        assert_eq!(
            solve_crane(&inputs, 100.0).unwrap_err().0,
            "Load chart contains a non-positive capacity."
        );
    }
}
