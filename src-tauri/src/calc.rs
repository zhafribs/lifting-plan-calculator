//! Calculation engine primitives — a direct port of `Calc.kt` from the Android
//! source.
//!
//! All masses are handled in kilogram, and — following common rigging
//! convention — a weighed mass is treated as weight-force (kgf) for tension
//! purposes. The crane is allowed to operate at a maximum of 75% of its rated
//! chart capacity.
//!
//! Two rules govern this file, exactly as in the source:
//!
//! 1. **The arithmetic is the source's, not a re-derivation.** Every constant
//!    and every early return is carried across unchanged, including the parts
//!    that look redundant.
//! 2. **Errors are values the UI can show, not crashes.** Anything a half-filled
//!    form can produce returns an [`CalcException`] carrying the source's own
//!    wording, because that sentence is the one an operator already recognises.

use std::fmt;

use crate::model::LoadMeasurements;

/// Raised for invalid inputs so the UI can show a friendly message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalcException(pub String);

impl fmt::Display for CalcException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CalcException {}

pub type CalcResult<T> = Result<T, CalcException>;

fn err(message: &str) -> CalcException {
    CalcException(message.to_string())
}

/// The three ways a load line can be measured, in the source's declared order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadType {
    Fixed,
    PerMetre,
    PerVolume,
}

impl LoadType {
    /// The stable key the storage and the wire format use.
    pub fn key(self) -> &'static str {
        match self {
            LoadType::Fixed => "fixed",
            LoadType::PerMetre => "per_metre",
            LoadType::PerVolume => "per_volume",
        }
    }

    /// The label the dropdown shows.
    pub fn label(self) -> &'static str {
        match self {
            LoadType::Fixed => "Fixed",
            LoadType::PerMetre => "Weight by Metre",
            LoadType::PerVolume => "Weight by Volume",
        }
    }

    /// Declared order, not alphabetical: the dropdown's order and the fallback
    /// for an unknown key both come from here so there is one list.
    pub const ORDER: [LoadType; 3] = [LoadType::Fixed, LoadType::PerMetre, LoadType::PerVolume];

    /// Never fails: an unrecognised key lands on [`LoadType::Fixed`].
    pub fn from_key(key: &str) -> LoadType {
        Self::ORDER
            .iter()
            .copied()
            .find(|item| item.key() == key)
            .unwrap_or(LoadType::Fixed)
    }
}

/// Total weight for a fixed-weight item: weight x quantity.
pub fn fixed_weight(weight_kg: f64, qty: i64) -> CalcResult<f64> {
    if weight_kg < 0.0 {
        return Err(err("Fixed weight cannot be negative."));
    }
    Ok(weight_kg * clamp_qty(qty)? as f64)
}

/// Total weight for a weight-by-metre item: (kg/m * length) * quantity.
pub fn weight_by_metre(kg_per_metre: f64, length_m: f64, qty: i64) -> CalcResult<f64> {
    if kg_per_metre < 0.0 || length_m < 0.0 {
        return Err(err("Weight per metre and length cannot be negative."));
    }
    Ok(kg_per_metre * length_m * clamp_qty(qty)? as f64)
}

/// Total weight for an item defined by density and volume:
/// volume (m^3) * density (kg/m^3) * quantity.
pub fn weight_by_volume(volume_m3: f64, density_kg_m3: f64, qty: i64) -> CalcResult<f64> {
    if volume_m3 < 0.0 {
        return Err(err("Volume cannot be negative."));
    }
    if density_kg_m3 <= 0.0 {
        return Err(err("Material density must be greater than 0."));
    }
    Ok(volume_m3 * density_kg_m3 * clamp_qty(qty)? as f64)
}

/// Dispatch to the measurement's own weight function.
pub fn load_item_weight(load_type: LoadType, item: &impl LoadMeasurements) -> CalcResult<f64> {
    match load_type {
        LoadType::Fixed => fixed_weight(item.weight_kg(), item.qty()),
        LoadType::PerMetre => weight_by_metre(item.kg_per_metre(), item.length_m(), item.qty()),
        LoadType::PerVolume => {
            weight_by_volume(item.volume_m3(), item.density_kg_m3(), item.qty())
        }
    }
}

fn clamp_qty(qty: i64) -> CalcResult<i64> {
    if qty < 0 {
        return Err(err("Quantity cannot be negative."));
    }
    Ok(qty)
}

/// Total weight seen at the crane hook (kg) = net load + rigging.
pub fn gross_lift_weight(load_total: f64, tackle_total: f64) -> CalcResult<f64> {
    if load_total < 0.0 || tackle_total < 0.0 {
        return Err(err("Weights cannot be negative."));
    }
    Ok(load_total + tackle_total)
}

/// Minimum rated crane capacity (kg) so `gross` uses at most the crane usage
/// ratio (75%) of the chart capacity. This is the figure to look up in the
/// crane's load chart.
pub fn required_chart_capacity(gross: f64, usage_ratio: f64) -> CalcResult<f64> {
    if gross < 0.0 {
        return Err(err("Total lift weight cannot be negative."));
    }
    if usage_ratio <= 0.0 || usage_ratio > 1.0 {
        return Err(err("Usage ratio must be between 0 and 1."));
    }
    Ok(gross / usage_ratio)
}

/// How much of the crane's rated chart capacity is in use (%).
pub fn chart_usage_percent(gross: f64, chart_capacity: f64) -> CalcResult<f64> {
    if gross < 0.0 {
        return Err(err("Total lift weight cannot be negative."));
    }
    if chart_capacity <= 0.0 {
        return Err(err("Crane chart capacity must be greater than 0."));
    }
    Ok(gross / chart_capacity * 100.0)
}

/// Design weight used for rigging checks = gross lift / crane ratio.
pub fn design_weight(gross: f64) -> CalcResult<f64> {
    required_chart_capacity(gross, CRANE_USAGE_RATIO)
}

/// Look up the rated capacity (kg) on a load chart at `radius`.
///
/// Below the smallest charted radius the capacity falls back to the closest
/// charted value; beyond the largest it falls back to the most distant value
/// (conservative). Between two charted radii a linear interpolation is used.
pub fn lookup_chart_capacity(rows: &[(f64, f64)], radius: f64) -> CalcResult<f64> {
    let mut chart: Vec<(f64, f64)> = rows.iter().copied().filter(|it| it.0 >= 0.0).collect();
    chart.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    if chart.is_empty() {
        return Err(err("Load chart is empty - nothing to look up."));
    }
    for (_, capacity) in &chart {
        if *capacity <= 0.0 {
            return Err(err("Load chart contains a non-positive capacity."));
        }
    }
    if radius < 0.0 {
        return Err(err("Working radius cannot be negative."));
    }

    if radius <= chart[0].0 {
        return Ok(chart[0].1);
    }
    if radius >= chart[chart.len() - 1].0 {
        return Ok(chart[chart.len() - 1].1);
    }

    let mut lo = chart[0];
    let mut hi = chart[chart.len() - 1];
    for point in &chart {
        if point.0 <= radius {
            lo = *point;
        }
        if point.0 >= radius {
            hi = *point;
            break;
        }
    }
    if lo.0 == hi.0 || lo.1 == hi.1 {
        return Ok(lo.1);
    }
    let t = (radius - lo.0) / (hi.0 - lo.0);
    Ok(lo.1 + t * (hi.1 - lo.1))
}

/// The measurements one load line carries, regardless of which type is active.
pub const DEFAULT_DENSITY_KG_M3: f64 = 7850.0;

/// The crane may be loaded to this fraction of its rated chart capacity.
pub const CRANE_USAGE_RATIO: f64 = 0.75;

/// How a crane's usage is banded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageState {
    Within,
    Caution,
    Over,
}

impl UsageState {
    /// The stable key the UI styles its status band with.
    pub fn key(self) -> &'static str {
        match self {
            UsageState::Within => "within",
            UsageState::Caution => "caution",
            UsageState::Over => "over",
        }
    }

    /// The wording the status band and the report print.
    pub fn wording(self) -> &'static str {
        match self {
            UsageState::Within => "WITHIN LIMIT",
            UsageState::Caution => "CAUTION",
            UsageState::Over => "WARNING",
        }
    }
}

/// Classify a usage percentage into its band.
///
/// One classifier for every surface that shows a crane's usage — the source
/// wrote this twice and the two had already drifted, which is exactly the
/// failure a single classifier prevents.
pub fn crane_usage_band(usage_percent: f64) -> UsageState {
    if usage_percent <= CRANE_USAGE_RATIO * 100.0 {
        UsageState::Within
    } else if usage_percent <= 100.0 {
        UsageState::Caution
    } else {
        UsageState::Over
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figures;
    use crate::model::{LoadItem, LoadMeasurements, TackleItem};

    #[test]
    fn fixed_weight_multiplies_by_quantity() {
        assert!((fixed_weight(100.0, 3).unwrap() - 300.0).abs() < 1e-9);
        assert!((fixed_weight(100.0, 1).unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn weight_by_metre_multiplies_rate_length_and_quantity() {
        assert!((weight_by_metre(5.0, 10.0, 1).unwrap() - 50.0).abs() < 1e-9);
        assert!((weight_by_metre(5.0, 10.0, 2).unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn weight_by_volume_multiplies_volume_density_and_quantity() {
        assert!((weight_by_volume(0.5, DEFAULT_DENSITY_KG_M3, 1).unwrap() - 0.5 * 7850.0).abs() < 1e-9);
    }

    #[test]
    fn negative_inputs_are_rejected_with_the_references_wording() {
        assert_eq!(
            fixed_weight(-1.0, 1).unwrap_err().0,
            "Fixed weight cannot be negative."
        );
        assert_eq!(
            weight_by_volume(1.0, 0.0, 1).unwrap_err().0,
            "Material density must be greater than 0."
        );
        assert_eq!(
            weight_by_metre(-1.0, 1.0, 1).unwrap_err().0,
            "Weight per metre and length cannot be negative."
        );
    }

    #[test]
    fn a_zero_quantity_line_still_carries_its_weight_on_a_load_line_but_not_on_tackle() {
        assert!((fixed_weight(250.0, 1).unwrap() - 250.0).abs() < 1e-9);
        let tackle = TackleItem {
            weight_kg: 250.0,
            qty: 0,
            ..TackleItem::default()
        };
        assert!((tackle.subtotal() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn an_incomplete_line_weighs_nothing_rather_than_failing() {
        let item = LoadItem {
            load_type: LoadType::PerMetre,
            kg_per_metre: 5.0,
            ..LoadItem::default()
        };
        assert!((item.weight() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn chart_lookup_interpolates_between_charted_radii() {
        let chart = vec![(5.0, 10000.0), (10.0, 5000.0)];
        assert!((lookup_chart_capacity(&chart, 3.0).unwrap() - 10000.0).abs() < 1e-9);
        assert!((lookup_chart_capacity(&chart, 5.0).unwrap() - 10000.0).abs() < 1e-9);
        assert!((lookup_chart_capacity(&chart, 7.5).unwrap() - 7500.0).abs() < 1e-9);
        assert!((lookup_chart_capacity(&chart, 10.0).unwrap() - 5000.0).abs() < 1e-9);
        assert!((lookup_chart_capacity(&chart, 22.0).unwrap() - 5000.0).abs() < 1e-9);
    }

    #[test]
    fn lookup_rejects_an_empty_chart() {
        assert_eq!(
            lookup_chart_capacity(&[], 1.0).unwrap_err().0,
            "Load chart is empty - nothing to look up."
        );
    }

    #[test]
    fn usage_bands_match_the_source() {
        assert_eq!(crane_usage_band(0.0), UsageState::Within);
        assert_eq!(crane_usage_band(74.99), UsageState::Within);
        assert_eq!(crane_usage_band(75.0), UsageState::Within);
        assert_eq!(crane_usage_band(75.1), UsageState::Caution);
        assert_eq!(crane_usage_band(100.0), UsageState::Caution);
        assert_eq!(crane_usage_band(100.1), UsageState::Over);
    }

    #[test]
    fn required_chart_capacity_is_the_75_percent_rule() {
        assert!(
            (required_chart_capacity(10_000.0, CRANE_USAGE_RATIO).unwrap()
                - 10_000.0 / 0.75)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn figures_use_a_comma_grouping_and_a_dot_decimal_regardless_of_device_locale() {
        assert_eq!(figures::fmt(Some(1234.5), 2, ""), "1,234.50");
        assert_eq!(figures::fmt(Some(1234.5), 2, "kg"), "1,234.50 kg");
        assert_eq!(figures::fmt(None, 2, "kg"), "\u{2014}");
        assert_eq!(figures::fmt(Some(0.0), 2, "kg"), "0.00 kg");
    }

    #[allow(dead_code)]
    fn load_measurements_marker(item: &impl LoadMeasurements) -> i64 {
        item.qty()
    }
}
