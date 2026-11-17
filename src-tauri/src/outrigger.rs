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
    pub pad_width_m: f64,
    pub thickness_material: String,
    pub crane_capacity_t: f64,
    pub foot_length_m: f64,
    pub foot_width_m: f64,
}

impl Default for OutriggerInputs {
    fn default() -> Self {
        Self {
            ground_key: String::new(),
            bearing_kpa: 0.0,
            crane_mass_kn: 0.0,
            lift_load_kn: 0.0,
            pad_shape: "square".to_string(),
            pad_width_m: 0.0,
            thickness_material: "steel".to_string(),
            crane_capacity_t: 0.0,
            foot_length_m: 0.0,
            foot_width_m: 0.0,
        }
    }
}

/// The thickness guidance for the chosen pad material.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ThicknessResult {
    pub material: String,
    pub crane_class: String,
    pub crane_capacity_t: f64,
    pub steel_range_mm: String,
    pub timber_range_mm: String,
    pub recommended_min_mm: f64,
    pub recommended_max_mm: f64,
    pub governing: String,
    pub cross_layer: bool,
    pub stiffeners: bool,
    pub notes: String,
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
    pub pad_width_m: Option<f64>,
    pub minimum_length_m: Option<f64>,
    pub thickness: ThicknessResult,
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

/// One row of the thickness guidance in the Thickness sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThicknessBand {
    pub key: &'static str,
    pub label: &'static str,
    pub crane_limit_t: f64,
    pub steel_min_mm: f64,
    pub steel_max_mm: f64,
    pub timber_min_mm: f64,
    pub timber_max_mm: f64,
}

/// The three crane classes in the supplied thickness research.
pub const THICKNESS_BANDS: [ThicknessBand; 3] = [
    ThicknessBand {
        key: "light",
        label: "Light (up to 25 t crane)",
        crane_limit_t: 25.0,
        steel_min_mm: 25.0,
        steel_max_mm: 32.0,
        timber_min_mm: 75.0,
        timber_max_mm: 100.0,
    },
    ThicknessBand {
        key: "medium",
        label: "Medium (25–80 t crane)",
        crane_limit_t: 80.0,
        steel_min_mm: 35.0,
        steel_max_mm: 50.0,
        timber_min_mm: 150.0,
        timber_max_mm: 200.0,
    },
    ThicknessBand {
        key: "heavy",
        label: "Heavy (80–200 t+ crane)",
        crane_limit_t: 1.0e9,
        steel_min_mm: 50.0,
        steel_max_mm: 75.0,
        timber_min_mm: 200.0,
        timber_max_mm: 300.0,
    },
];

fn thickness_band(crane_capacity_t: f64) -> Option<&'static ThicknessBand> {
    if !crane_capacity_t.is_finite() || crane_capacity_t <= 0.0 {
        return None;
    }
    THICKNESS_BANDS
        .iter()
        .find(|band| crane_capacity_t <= band.crane_limit_t)
}

fn thickness_range(min_mm: f64, max_mm: f64, plus: bool) -> String {
    if plus {
        format!("{min_mm:.0}–{max_mm:.0}+ mm")
    } else {
        format!("{min_mm:.0}–{max_mm:.0} mm")
    }
}

fn ceil5(value: f64) -> f64 {
    (value / 5.0).ceil() * 5.0
}

/// The thickness guidance for the chosen material and the pad dimensions.
///
/// The class ranges are the supplied research; the 45-degree timber rule and
/// the steel-plate width limit can raise the minimum. The result is guidance
/// for the pad build-up, not a structural design.
pub fn thickness_for(
    inputs: &OutriggerInputs,
    pad_length_m: f64,
    pad_width_m: f64,
) -> ThicknessResult {
    let material = if inputs.thickness_material == "timber" {
        "timber"
    } else {
        "steel"
    };
    let Some(band) = thickness_band(inputs.crane_capacity_t) else {
        return ThicknessResult {
            material: material.to_string(),
            crane_class: "Not set".to_string(),
            crane_capacity_t: inputs.crane_capacity_t,
            steel_range_mm: "Enter the crane rated capacity".to_string(),
            timber_range_mm: "Enter the crane rated capacity".to_string(),
            recommended_min_mm: 0.0,
            recommended_max_mm: 0.0,
            governing: "Crane capacity".to_string(),
            cross_layer: false,
            stiffeners: false,
            notes: "Enter the crane rated capacity in the Crane tab to choose a thickness class."
                .to_string(),
        };
    };

    let heavy = band.key == "heavy";
    let steel_range = thickness_range(band.steel_min_mm, band.steel_max_mm, heavy);
    let timber_range = thickness_range(band.timber_min_mm, band.timber_max_mm, false);
    let (mut min_mm, max_mm) = if material == "timber" {
        (band.timber_min_mm, band.timber_max_mm)
    } else {
        (band.steel_min_mm, band.steel_max_mm)
    };
    let mut governing = "Crane class".to_string();
    let mut notes: Vec<String> = Vec::new();
    let mut cross_layer = false;
    let mut stiffeners = false;

    if material == "timber" {
        if pad_length_m > 1.2 || pad_width_m > 1.2 {
            cross_layer = true;
            notes.push(
                "A pad wider than 1.2 m should be cross-layered: two or more 100–150 mm hardwood layers at 90°, with a steel plate on top under the outrigger foot."
                    .to_string(),
            );
        }
        if inputs.foot_length_m > 0.0 && inputs.foot_width_m > 0.0 {
            let overhang_mm = ((pad_length_m - inputs.foot_length_m).max(0.0).max(
                (pad_width_m - inputs.foot_width_m).max(0.0),
            ) / 2.0)
                * 1000.0;
            if overhang_mm > min_mm {
                min_mm = ceil5(overhang_mm);
                governing = "45° load spread".to_string();
            }
            if overhang_mm > max_mm {
                notes.push(
                    "The edge overhang is larger than the class range; use a thicker mat or cross-layering."
                        .to_string(),
                );
            }
        } else {
            notes.push(
                "Enter the outrigger foot length and width to check the 45° timber load-spread rule."
                    .to_string(),
            );
        }
        notes.push(
            "Never use plywood, OSB, hollow blocks or softwoods for a crane pad.".to_string(),
        );
    } else if pad_length_m > 1.2 && pad_width_m > 1.2 {
        if min_mm < 40.0 {
            min_mm = 40.0;
            governing = "Steel plate width".to_string();
        }
        stiffeners = true;
        notes.push(
            "A steel plate wider than 1.2 m × 1.2 m needs at least 40–50 mm, or welded stiffening ribs."
                .to_string(),
        );
    }

    ThicknessResult {
        material: material.to_string(),
        crane_class: band.label.to_string(),
        crane_capacity_t: inputs.crane_capacity_t,
        steel_range_mm: steel_range,
        timber_range_mm: timber_range,
        recommended_min_mm: min_mm,
        recommended_max_mm: max_mm.max(min_mm),
        governing,
        cross_layer,
        stiffeners,
        notes: notes.join(" "),
    }
}

/// The example the tab loads on request.
pub fn example_outrigger_inputs() -> OutriggerInputs {
    OutriggerInputs {
        ground_key: "cohesive_stiff".to_string(),
        bearing_kpa: 150.0,
        crane_mass_kn: 400.0,
        lift_load_kn: 50.0,
        pad_shape: "square".to_string(),
        pad_width_m: 0.0,
        thickness_material: "steel".to_string(),
        crane_capacity_t: 50.0,
        foot_length_m: 0.0,
        foot_width_m: 0.0,
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

    let pad_shape = if inputs.pad_shape == "rectangular" {
        "rectangular"
    } else {
        "square"
    };
    let (pad_width, minimum_length) = if pad_shape == "rectangular" {
        if !inputs.pad_width_m.is_finite() || inputs.pad_width_m <= 0.0 {
            return Err(err("Enter the pad width for a rectangular pad."));
        }
        let length = (required_area / inputs.pad_width_m / 0.05).ceil() * 0.05;
        (Some(inputs.pad_width_m), Some(length))
    } else {
        (None, None)
    };

    let (thickness_length, thickness_width) = if pad_shape == "rectangular" {
        (minimum_length.unwrap_or(0.0), pad_width.unwrap_or(0.0))
    } else {
        (recommended_square_rounded, recommended_square_rounded)
    };
    let thickness = thickness_for(inputs, thickness_length, thickness_width);

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
        pad_shape: pad_shape.to_string(),
        pad_width_m: pad_width,
        minimum_length_m: minimum_length,
        thickness,
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
        assert_eq!(result.pad_shape, "square");
        assert!(result.minimum_length_m.is_none());
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
    fn a_rectangular_pad_gets_the_minimum_length_for_its_width() {
        let mut input = example();
        input.pad_shape = "rectangular".to_string();
        input.pad_width_m = 2.0;
        let result = solve_outrigger(&input).unwrap();
        assert_eq!(result.pad_shape, "rectangular");
        assert_eq!(result.pad_width_m, Some(2.0));
        // 1.95 m2 / 2.00 m = 0.975 m, rounded up to the next 50 mm = 1.00 m.
        assert_eq!(result.minimum_length_m, Some(1.0));
    }

    #[test]
    fn a_rectangular_pad_without_a_width_is_refused() {
        let mut input = example();
        input.pad_shape = "rectangular".to_string();
        input.pad_width_m = 0.0;
        assert!(solve_outrigger(&input).is_err());
    }

    #[test]
    fn a_light_steel_pad_uses_the_light_steel_range() {
        let mut input = example();
        input.crane_capacity_t = 20.0;
        input.thickness_material = "steel".to_string();
        input.pad_shape = "rectangular".to_string();
        input.pad_width_m = 1.0;
        let result = solve_outrigger(&input).unwrap();
        assert_eq!(result.thickness.crane_class, "Light (up to 25 t crane)");
        assert!((result.thickness.recommended_min_mm - 25.0).abs() < 1e-9);
        assert!((result.thickness.recommended_max_mm - 32.0).abs() < 1e-9);
    }

    #[test]
    fn a_wide_steel_pad_raises_the_minimum_and_flags_stiffeners() {
        let mut input = example();
        input.crane_capacity_t = 20.0;
        input.thickness_material = "steel".to_string();
        input.pad_width_m = 1.4;
        input.pad_shape = "rectangular".to_string();
        let result = solve_outrigger(&input).unwrap();
        assert!(result.thickness.stiffeners);
        assert!((result.thickness.recommended_min_mm - 40.0).abs() < 1e-9);
        assert_eq!(result.thickness.governing, "Steel plate width");
    }

    #[test]
    fn the_timber_overhang_can_raise_the_thickness_above_the_class_range() {
        let mut input = example();
        input.crane_capacity_t = 50.0;
        input.thickness_material = "timber".to_string();
        input.pad_width_m = 2.0;
        input.pad_shape = "rectangular".to_string();
        input.foot_length_m = 0.6;
        input.foot_width_m = 0.6;
        let result = solve_outrigger(&input).unwrap();
        assert_eq!(result.thickness.governing, "45° load spread");
        assert!((result.thickness.recommended_min_mm - 700.0).abs() < 1e-9);
        assert!(result.thickness.cross_layer);
    }

    #[test]
    fn a_missing_crane_capacity_leaves_thickness_unset() {
        let mut input = example();
        input.crane_capacity_t = 0.0;
        let result = solve_outrigger(&input).unwrap();
        assert_eq!(result.thickness.crane_class, "Not set");
        assert_eq!(result.thickness.recommended_min_mm, 0.0);
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
