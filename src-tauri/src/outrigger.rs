//! Outrigger pad sizing from the presumed allowable bearing values in
//! BS 8004 Table 1, adapted for Malaysian practice.
//!
//! The calculation is deliberately small and explicit:
//!
//! ```text
//! A = 0.65 x (CM + L) / V
//! ```
//!
//! where `CM` is the crane's operating mass (kN), `L` is the lifted load (kN)
//! and `V` is the maximum permissible safe bearing capacity (kPa). The result
//! sizes the bearing **area** only. Pad thickness, steel plate design and the
//! crane manufacturer's outrigger load are separate checks.

use std::fmt;

/// The constant in the published formula.
pub const OUTRIGGER_CONSTANT: f64 = 0.65;

/// One kilogram expressed as a kilonewton of weight (g = 9.80665 m/s²).
pub const KG_TO_KN: f64 = 9.80665 / 1000.0;

/// The four groups the ground table publishes.
pub const GROUND_CATEGORIES: [&str; 4] = [
    "Rocks",
    "Non-Cohesive Soils",
    "Cohesive Soils",
    "Organic & Fill",
];

/// One row of the BS 8004 Table 1 ground table.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundType {
    pub key: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub value_label: &'static str,
    pub v_default: Option<f64>,
    pub v_min: Option<f64>,
    pub v_max: Option<f64>,
    pub notes: &'static str,
    pub suitable: bool,
}

/// The table as the app presents it. Keys are stable so a selection survives
/// a solve cycle and can be carried into the report.
pub const GROUND_TYPES: [GroundType; 20] = [
    GroundType {
        key: "rocks_granite",
        category: "Rocks",
        description: "Strong igneous and gneissic rocks in sound condition (e.g., fresh Granite)",
        value_label: "10,000",
        v_default: Some(10_000.0),
        v_min: Some(10_000.0),
        v_max: Some(10_000.0),
        notes: "Requires pneumatic tools for excavation.",
        suitable: true,
    },
    GroundType {
        key: "rocks_limestone_sandstone",
        category: "Rocks",
        description: "Strong limestones and strong sandstones",
        value_label: "4,000",
        v_default: Some(4_000.0),
        v_min: Some(4_000.0),
        v_max: Some(4_000.0),
        notes: "Common in Kuala Lumpur (Kuala Lumpur Limestone) and Ipoh; check for cavities.",
        suitable: true,
    },
    GroundType {
        key: "rocks_schist_slate",
        category: "Rocks",
        description: "Schists and slates",
        value_label: "3,000",
        v_default: Some(3_000.0),
        v_min: Some(3_000.0),
        v_max: Some(3_000.0),
        notes: "Common in Kenny Hill Formation (lightly metamorphosed).",
        suitable: true,
    },
    GroundType {
        key: "rocks_shale_mudstone_siltstone",
        category: "Rocks",
        description: "Strong shales, strong mudstones, and strong siltstones",
        value_label: "2,000",
        v_default: Some(2_000.0),
        v_min: Some(2_000.0),
        v_max: Some(2_000.0),
        notes: "Requires careful inspection for bedding planes.",
        suitable: true,
    },
    GroundType {
        key: "rocks_clay_shale",
        category: "Rocks",
        description: "Clay shales",
        value_label: "1,000",
        v_default: Some(1_000.0),
        v_min: Some(1_000.0),
        v_max: Some(1_000.0),
        notes: "Subject to softening if exposed to water.",
        suitable: true,
    },
    GroundType {
        key: "rocks_weathered",
        category: "Rocks",
        description: "Soft or weathered rocks (e.g., Grade IV/V weathered granite or sandstone)",
        value_label: "600 – 1,000",
        v_default: Some(600.0),
        v_min: Some(600.0),
        v_max: Some(1_000.0),
        notes: "Very common in Malaysian residual soil profiles; verify via SPT/coring.",
        suitable: true,
    },
    GroundType {
        key: "rocks_chalk_fractured",
        category: "Rocks",
        description: "Chalk / heavily fractured weak rock",
        value_label: "600",
        v_default: Some(600.0),
        v_min: Some(600.0),
        v_max: Some(600.0),
        notes: "Assess on a site-by-site basis.",
        suitable: true,
    },
    GroundType {
        key: "granular_dense_gravel",
        category: "Non-Cohesive Soils",
        description: "Dense gravel, or dense sand and gravel",
        value_label: "> 600",
        v_default: Some(600.0),
        v_min: Some(600.0),
        v_max: None,
        notes: "Requires pick for excavation; high resistance to probing.",
        suitable: true,
    },
    GroundType {
        key: "granular_medium_gravel",
        category: "Non-Cohesive Soils",
        description: "Medium dense gravel, or medium dense sand and gravel",
        value_label: "200 – 600",
        v_default: Some(200.0),
        v_min: Some(200.0),
        v_max: Some(600.0),
        notes: "Moderate resistance to pick and shovel.",
        suitable: true,
    },
    GroundType {
        key: "granular_loose_gravel",
        category: "Non-Cohesive Soils",
        description: "Loose gravel, or loose sand and gravel",
        value_label: "< 200",
        v_default: None,
        v_min: Some(0.0),
        v_max: Some(200.0),
        notes: "Easily excavated with a shovel. Enter a site-specific V below 200 kPa.",
        suitable: true,
    },
    GroundType {
        key: "granular_dense_sand",
        category: "Non-Cohesive Soils",
        description: "Compact / dense sand",
        value_label: "> 300",
        v_default: Some(300.0),
        v_min: Some(300.0),
        v_max: None,
        notes: "High resistance to penetration (SPT N > 30).",
        suitable: true,
    },
    GroundType {
        key: "granular_medium_sand",
        category: "Non-Cohesive Soils",
        description: "Medium dense sand",
        value_label: "100 – 300",
        v_default: Some(100.0),
        v_min: Some(100.0),
        v_max: Some(300.0),
        notes: "Common in coastal plains; SPT N = 10 to 30.",
        suitable: true,
    },
    GroundType {
        key: "granular_loose_sand",
        category: "Non-Cohesive Soils",
        description: "Loose sand",
        value_label: "< 100",
        v_default: None,
        v_min: Some(0.0),
        v_max: Some(100.0),
        notes: "Easily excavated; SPT N < 10. Enter a site-specific V below 100 kPa.",
        suitable: true,
    },
    GroundType {
        key: "cohesive_very_stiff",
        category: "Cohesive Soils",
        description: "Very stiff boulder clays and hard clays",
        value_label: "300 – 600",
        v_default: Some(300.0),
        v_min: Some(300.0),
        v_max: Some(600.0),
        notes: "Indented only with difficulty by thumbnail.",
        suitable: true,
    },
    GroundType {
        key: "cohesive_stiff",
        category: "Cohesive Soils",
        description: "Stiff clays (e.g., stiff residual silty clay)",
        value_label: "150 – 300",
        v_default: Some(150.0),
        v_min: Some(150.0),
        v_max: Some(300.0),
        notes: "Indented by thumbnail, cannot be moulded by fingers.",
        suitable: true,
    },
    GroundType {
        key: "cohesive_firm",
        category: "Cohesive Soils",
        description: "Firm clays (e.g., firm residual sandy/silty clay)",
        value_label: "75 – 150",
        v_default: Some(75.0),
        v_min: Some(75.0),
        v_max: Some(150.0),
        notes: "Moulded only by strong finger pressure.",
        suitable: true,
    },
    GroundType {
        key: "cohesive_soft",
        category: "Cohesive Soils",
        description: "Soft clays and silts (e.g., Marine Clay along West Coast)",
        value_label: "< 75",
        v_default: None,
        v_min: Some(0.0),
        v_max: Some(75.0),
        notes: "Easily moulded by fingers; high consolidation settlement risk. Enter a site-specific V below 75 kPa.",
        suitable: true,
    },
    GroundType {
        key: "cohesive_very_soft",
        category: "Cohesive Soils",
        description: "Very soft clays and silts",
        value_label: "Not applicable",
        v_default: None,
        v_min: None,
        v_max: None,
        notes: "Extrudes between fingers; requires piling or ground improvement.",
        suitable: false,
    },
    GroundType {
        key: "organic_peat",
        category: "Organic & Fill",
        description: "Peat and organic soils (common in coastal Sarawak, Johor, Selangor)",
        value_label: "Not applicable",
        v_default: None,
        v_min: None,
        v_max: None,
        notes: "Extremely compressible; unsuitable for shallow foundations.",
        suitable: false,
    },
    GroundType {
        key: "organic_fill",
        category: "Organic & Fill",
        description: "Made ground or uncontrolled fill (e.g., ex-mining land / tin tailings)",
        value_label: "Not applicable",
        v_default: None,
        v_min: None,
        v_max: None,
        notes: "Common in Klang Valley & Kinta Valley; requires site-specific investigation.",
        suitable: false,
    },
];

/// The outrigger form.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct OutriggerInputs {
    pub ground_key: String,
    pub bearing_kpa: f64,
    pub crane_mass_kn: f64,
    pub lift_load_kn: f64,
    pub pad_shape: String,
    pub pad_length_m: f64,
    pub pad_width_m: f64,
}

impl Default for OutriggerInputs {
    fn default() -> Self {
        Self {
            ground_key: String::new(),
            bearing_kpa: 0.0,
            crane_mass_kn: 0.0,
            lift_load_kn: 0.0,
            pad_shape: "square".to_string(),
            pad_length_m: 0.0,
            pad_width_m: 0.0,
        }
    }
}

/// The solved pad, or the reason it could not be sized.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct OutriggerResult {
    pub ground_key: String,
    pub ground_category: String,
    pub ground_description: String,
    pub ground_value_label: String,
    pub ground_notes: String,
    pub bearing_kpa: f64,
    pub crane_mass_kn: f64,
    pub lift_load_kn: f64,
    pub design_load_kn: f64,
    pub required_area_m2: f64,
    pub recommended_square_m: f64,
    pub recommended_square_rounded_m: f64,
    pub pad_shape: String,
    pub pad_length_m: f64,
    pub pad_width_m: f64,
    pub pad_area_m2: Option<f64>,
    pub applied_pressure_kpa: Option<f64>,
    pub utilisation_percent: Option<f64>,
    /// `ok`, `caution`, `over` or `size_required`.
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutriggerError(pub String);

impl fmt::Display for OutriggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for OutriggerError {}

fn err(message: &str) -> OutriggerError {
    OutriggerError(message.to_string())
}

/// Find one ground row by its stable key.
pub fn ground_type(key: &str) -> Option<&'static GroundType> {
    GROUND_TYPES.iter().find(|ground| ground.key == key)
}

/// Choose the first row in a category, used by the tab when the category
/// changes.
pub fn first_in_category(category: &str) -> Option<&'static GroundType> {
    GROUND_TYPES.iter().find(|ground| ground.category == category)
}

/// The example the tab loads on request.
pub fn example_outrigger_inputs() -> OutriggerInputs {
    OutriggerInputs {
        ground_key: "cohesive_stiff".to_string(),
        bearing_kpa: 150.0,
        crane_mass_kn: 400.0,
        lift_load_kn: 50.0,
        pad_shape: "square".to_string(),
        pad_length_m: 2.0,
        pad_width_m: 2.0,
    }
}

/// The outrigger pad calculation.
pub fn solve_outrigger(inputs: &OutriggerInputs) -> Result<OutriggerResult, OutriggerError> {
    let ground = ground_type(&inputs.ground_key)
        .ok_or_else(|| err("Choose a ground category and description."))?;
    if !ground.suitable {
        return Err(err(
            "This ground is not suitable for shallow foundations. Use piling or ground improvement, or obtain a site-specific bearing value from the geotechnical assessment.",
        ));
    }

    let v = inputs.bearing_kpa;
    if !v.is_finite() || v <= 0.0 {
        return Err(err("Enter the maximum permissible bearing value V in kPa."));
    }
    let cm = inputs.crane_mass_kn;
    if !cm.is_finite() || cm <= 0.0 {
        return Err(err("Enter the total crane mass CM in kN."));
    }
    let l = inputs.lift_load_kn;
    if !l.is_finite() || l < 0.0 {
        return Err(err("The lift load cannot be negative."));
    }

    let design_load = OUTRIGGER_CONSTANT * (cm + l);
    let required_area = design_load / v;
    if !required_area.is_finite() || required_area <= 0.0 {
        return Err(err("The design load and bearing value do not produce a pad area."));
    }
    let recommended_square = required_area.sqrt();
    // Round up to the next 50 mm, the practical increment used on site.
    let recommended_square_rounded = (recommended_square / 0.05).ceil() * 0.05;

    let (pad_area, applied_pressure, utilisation, status, message) = if inputs.pad_length_m > 0.0
        || inputs.pad_width_m > 0.0
    {
        if inputs.pad_length_m <= 0.0 || inputs.pad_width_m <= 0.0 {
            return Err(err("Enter both the pad length and the pad width."));
        }
        let area = inputs.pad_length_m * inputs.pad_width_m;
        if !area.is_finite() || area <= 0.0 {
            return Err(err("The pad length and width do not produce an area."));
        }
        let pressure = design_load / area;
        let use_percent = pressure / v * 100.0;
        let (status, message) = if use_percent > 100.0 {
            ("over", "Pad is too small for the design load. Increase the bearing area.")
        } else if use_percent >= 90.0 {
            ("caution", "Pad is close to the limit. Check the ground and the pad build-up.")
        } else {
            ("ok", "Pad size is adequate for the design load.")
        };
        (
            Some(area),
            Some(pressure),
            Some(use_percent),
            status,
            message,
        )
    } else {
        (
            None,
            None,
            None,
            "size_required",
            "Enter a pad size, or use the recommended square pad.",
        )
    };

    Ok(OutriggerResult {
        ground_key: ground.key.to_string(),
        ground_category: ground.category.to_string(),
        ground_description: ground.description.to_string(),
        ground_value_label: ground.value_label.to_string(),
        ground_notes: ground.notes.to_string(),
        bearing_kpa: v,
        crane_mass_kn: cm,
        lift_load_kn: l,
        design_load_kn: design_load,
        required_area_m2: required_area,
        recommended_square_m: recommended_square,
        recommended_square_rounded_m: recommended_square_rounded,
        pad_shape: if inputs.pad_shape == "rectangular" {
            "rectangular".to_string()
        } else {
            "square".to_string()
        },
        pad_length_m: inputs.pad_length_m,
        pad_width_m: inputs.pad_width_m,
        pad_area_m2: pad_area,
        applied_pressure_kpa: applied_pressure,
        utilisation_percent: utilisation,
        status: status.to_string(),
        message: message.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> OutriggerInputs {
        example_outrigger_inputs()
    }

    #[test]
    fn the_table_has_twenty_rows_and_all_four_categories() {
        assert_eq!(GROUND_TYPES.len(), 20);
        for category in GROUND_CATEGORIES {
            assert!(GROUND_TYPES.iter().any(|ground| ground.category == category));
        }
    }

    #[test]
    fn the_example_matches_the_published_formula() {
        let result = solve_outrigger(&example()).unwrap();
        assert!((result.design_load_kn - 292.5).abs() < 1e-9);
        assert!((result.required_area_m2 - 1.95).abs() < 1e-9);
        assert!((result.recommended_square_m - 1.95_f64.sqrt()).abs() < 1e-9);
        assert!((result.recommended_square_rounded_m - 1.40).abs() < 1e-9);
        assert_eq!(result.status, "ok");
        assert!((result.applied_pressure_kpa.unwrap() - 73.125).abs() < 1e-9);
        assert!((result.utilisation_percent.unwrap() - 48.75).abs() < 1e-9);
    }

    #[test]
    fn a_weathered_rock_range_uses_its_lower_bound_by_default() {
        let mut input = example();
        input.ground_key = "rocks_weathered".to_string();
        input.bearing_kpa = 600.0;
        let result = solve_outrigger(&input).unwrap();
        assert!((result.required_area_m2 - 0.4875).abs() < 1e-9);
        assert!((result.recommended_square_rounded_m - 0.70).abs() < 1e-9);
    }

    #[test]
    fn a_pad_below_the_required_area_is_over() {
        let mut input = example();
        input.pad_length_m = 1.0;
        input.pad_width_m = 1.0;
        let result = solve_outrigger(&input).unwrap();
        assert_eq!(result.status, "over");
        assert!((result.utilisation_percent.unwrap() - 195.0).abs() < 1e-9);
    }

    #[test]
    fn a_pad_at_ninety_percent_is_caution() {
        let mut input = example();
        // 292.5 / (1.43 * 1.44) / 150 = 94.6%
        input.pad_length_m = 1.43;
        input.pad_width_m = 1.44;
        let result = solve_outrigger(&input).unwrap();
        assert_eq!(result.status, "caution");
    }

    #[test]
    fn no_pad_dimensions_ask_for_a_size_instead_of_failing() {
        let mut input = example();
        input.pad_length_m = 0.0;
        input.pad_width_m = 0.0;
        let result = solve_outrigger(&input).unwrap();
        assert_eq!(result.status, "size_required");
        assert!(result.pad_area_m2.is_none());
    }

    #[test]
    fn one_missing_dimension_is_refused() {
        let mut input = example();
        input.pad_length_m = 2.0;
        input.pad_width_m = 0.0;
        assert!(solve_outrigger(&input).is_err());
    }

    #[test]
    fn unsuitable_ground_is_refused_with_the_reason() {
        let mut input = example();
        input.ground_key = "organic_peat".to_string();
        input.bearing_kpa = 50.0;
        let error = solve_outrigger(&input).unwrap_err();
        assert!(error.0.contains("not suitable for shallow foundations"));
    }

    #[test]
    fn a_missing_bearing_value_is_refused() {
        let mut input = example();
        input.bearing_kpa = 0.0;
        assert!(solve_outrigger(&input).is_err());
    }

    #[test]
    fn a_missing_crane_mass_is_refused() {
        let mut input = example();
        input.crane_mass_kn = 0.0;
        assert!(solve_outrigger(&input).is_err());
    }
}
