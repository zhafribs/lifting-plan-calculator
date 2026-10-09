//! The sling arrangements, ported from `Sling.kt`.
//!
//! Units are metres, kilograms and degrees. `load_weight` is the **gross load
//! at hook**. The 75% allowable window is a crane load-chart rule and sizes the
//! lift against the crane only; it is deliberately *not* applied here, so the
//! sling is checked against the load the hook actually carries. (The 0.75 that
//! appears as a choke `base_reeve_factor` is the separate choker-derate, which
//! the reference documents carry as RF.)
//!
//! **It is a transcription, not a re-derivation.** Every formula, bound, early
//! return and error message is the source's. Where the source rounds a value
//! before using it, this rounds it too — that is not tidiness, it is the model:
//! the reference documents derive every printed force *from the factor they
//! print*.
//!
//! **`rint`, not "round".** Python's `round()` breaks a tie to even, and so
//! does `Math.rint`; [`f64::round_ties_even`] is used here for the same reason.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A sling input that cannot be solved, in words an operator can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlingInputError(pub String);

impl fmt::Display for SlingInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SlingInputError {}

fn err(message: &str) -> SlingInputError {
    SlingInputError(message.to_string())
}

/// Decimal places the reference documents use for a published sling angle factor.
pub const AF_DECIMALS: u32 = 3;

/// The reference documents print every length to the millimetre, so a length
/// that another printed line multiplies has to be rounded before it is used.
pub const LENGTH_DECIMALS: u32 = 3;

/// The leg count at which a tag has no working angle to enter.
pub const SINGLE_LEG: i64 = 1;

pub const ANGLE_BASIS_INTERNAL: &str = "internal";
pub const ANGLE_BASIS_EXTERNAL: &str = "external";

/// Round half to even, as Python's `round()` does.
fn round_half_even(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round_ties_even() / factor
}

/// A published angle factor, at the precision the reports use.
pub fn published_af(value: f64) -> f64 {
    round_half_even(value, AF_DECIMALS)
}

/// The first element yielding the largest value, as Kotlin's `maxByOrNull` does.
///
/// Rust's `Iterator::max_by` returns the *last* element on a tie, which would
/// pick a different governing leg group than the source on equal offsets.
pub(crate) fn max_by_first<T, F: Fn(&T) -> f64>(items: &[T], key: F) -> Option<&T> {
    let mut best: Option<&T> = None;
    let mut best_value = f64::NEG_INFINITY;
    for item in items {
        let value = key(item);
        if best.is_none() || value > best_value {
            best = Some(item);
            best_value = value;
        }
    }
    best
}

/// A published length, at the precision the reports use.
pub fn published_length(value: f64) -> f64 {
    round_half_even(value, LENGTH_DECIMALS)
}

/// The published product of a longitudinal and transverse factor.
///
/// The 2-leg basket and nested reference documents publish both factors and
/// then multiply the *published* pair to reach the combined factor.
pub fn published_combined_af(af_long: f64, af_trans: f64) -> f64 {
    published_af(published_af(af_long) * published_af(af_trans))
}

/// The leg count an installed tag rating is declared on.
///
/// The reference documents state the count under their note heading: *"If 4-Leg
/// Sling, use Sling Legs = 3"*, *"If 3-Leg Sling, use Sling Legs = 2"* and
/// *"If 2-Leg Sling, use Sling Legs = 2"*. A 3- or 4-leg bridle is rated for
/// the case where one leg does not take its share, so it is rated on one leg
/// fewer than it has; a 2-leg sling is rated on both.
pub fn rated_legs(legs: i64) -> i64 {
    let count = legs.max(1);
    if count > 2 {
        count - 1
    } else {
        count
    }
}

/// The per-leg capacity of an installed tag rating.
///
/// One sling leg carries the installed tag WLL divided by the rated leg count
/// and the tag's rated angle factor, e.g. 5,300 kg / (2 x 0.707) = 3,748.2 kg.
pub fn leg_capacity(tag_wll: f64, legs: i64, tag_af: f64) -> Option<f64> {
    if tag_wll <= 0.0 || legs < 1 || tag_af <= 0.0 {
        return None;
    }
    Some(tag_wll / (legs as f64 * tag_af))
}

/// The tag-rating utilisation as a percentage.
///
/// The reference documents divide the *printed* required WLL by the *printed*
/// per-leg capacity, so the published values are used here too.
pub fn tag_utilisation(required_wll: f64, leg_capacity: Option<f64>) -> Option<f64> {
    let capacity = leg_capacity?;
    if capacity <= 0.0 || required_wll <= 0.0 {
        return None;
    }
    Some(round_half_even(required_wll, 1) / round_half_even(capacity, 1) * 100.0)
}

/// One sling arrangement's declared shape.
///
/// `base_reeve_factor` is the denominator factor used in the documents' WLL
/// equation. It is not a generic "efficiency" multiplier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitchSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub shape: &'static str,
    pub effective_legs: i64,
    pub base_reeve_factor: f64,
    pub needs_angle: bool,
    pub angle_description: &'static str,
    pub round_load: bool,
    pub rectangular_load: bool,
    pub bridle: bool,
    pub leg_count: i64,
    pub needs_pick_distance: bool,
    pub nested: i64,
}

impl HitchSpec {
    /// Whether an installed tag is rated at an angle.
    ///
    /// A single leg hangs vertical, so it takes the tag's full rating and there
    /// is no per-leg derate to apply. A 2-leg or bridle arrangement splits the
    /// load between legs that splay out from the hook, so the tag's rating is
    /// derated by `cos(Tag WLL Angle)`. Derived from `leg_count` so it cannot
    /// drift.
    pub fn needs_tag_angle(&self) -> bool {
        self.leg_count >= 2
    }
}

const fn spec(
    key: &'static str,
    label: &'static str,
    shape: &'static str,
    effective_legs: i64,
    base_reeve_factor: f64,
    needs_angle: bool,
    angle_description: &'static str,
    round_load: bool,
    rectangular_load: bool,
    bridle: bool,
    leg_count: i64,
    needs_pick_distance: bool,
    nested: i64,
) -> HitchSpec {
    HitchSpec {
        key,
        label,
        shape,
        effective_legs,
        base_reeve_factor,
        needs_angle,
        angle_description,
        round_load,
        rectangular_load,
        bridle,
        leg_count,
        needs_pick_distance,
        nested,
    }
}

use std::f64::consts::PI;

/// The arrangements, in the order the 1-Leg reference folder uses.
pub const HITCH_SPECS: [HitchSpec; 19] = [
    spec("direct_hook", "Direct hook", "none", 1, 1.00, false, "", false, false, false, 1, false, 0),
    spec(
        "round_choke",
        "Round choke",
        "round",
        1,
        0.75,
        true,
        "Choke angle (inside/outside the loop)",
        true,
        false,
        false,
        1,
        false,
        0,
    ),
    spec(
        "rect_choke",
        "Rectangular choke",
        "rectangular",
        1,
        0.50,
        true,
        "Choke flare angle from vertical",
        false,
        true,
        false,
        1,
        false,
        0,
    ),
    spec("round_basket", "Round basket", "round", 2, 2.00, false, "", true, false, false, 1, false, 0),
    spec(
        "rect_basket",
        "Rectangular basket",
        "rectangular",
        2,
        1.00,
        false,
        "",
        false,
        true,
        false,
        1,
        false,
        0,
    ),
    spec(
        "vertical_round_basket",
        "Round vertical basket",
        "round",
        2,
        2.00,
        false,
        "",
        true,
        false,
        false,
        1,
        false,
        0,
    ),
    spec(
        "vertical_rect_basket",
        "Rectangular vertical basket",
        "rectangular",
        2,
        1.00,
        false,
        "",
        false,
        true,
        false,
        1,
        false,
        0,
    ),
    spec(
        "two_leg_direct",
        "2-Leg Direct",
        "two_direct",
        2,
        1.00,
        false,
        "",
        false,
        false,
        false,
        2,
        true,
        0,
    ),
    spec(
        "two_leg_round_choke",
        "2-Leg Round Choke",
        "round",
        2,
        0.75,
        true,
        "Choke angle (inside/outside the loop)",
        true,
        false,
        false,
        2,
        true,
        0,
    ),
    spec(
        "two_leg_rect_choke",
        "2-Leg Rectangular Choke",
        "rectangular",
        2,
        0.50,
        true,
        "Choke flare angle from vertical",
        false,
        true,
        false,
        2,
        true,
        0,
    ),
    spec(
        "two_leg_round_basket",
        "2-Leg Round Basket",
        "round",
        4,
        2.00,
        false,
        "",
        true,
        false,
        false,
        2,
        true,
        0,
    ),
    spec(
        "two_leg_rect_basket",
        "2-Leg Rectangular Basket",
        "rectangular",
        4,
        1.00,
        false,
        "",
        false,
        true,
        false,
        2,
        true,
        0,
    ),
    spec(
        "two_leg_vertical_round_basket",
        "2-Leg Round Vertical Basket",
        "round",
        4,
        2.00,
        false,
        "",
        true,
        false,
        false,
        2,
        true,
        0,
    ),
    spec(
        "two_leg_vertical_rect_basket",
        "2-Leg Rectangular Vertical Basket",
        "rectangular",
        4,
        1.00,
        false,
        "",
        false,
        true,
        false,
        2,
        true,
        0,
    ),
    spec(
        "two_leg_nested_2leg",
        "2-Leg Sling in Each Leg of 2-Leg Sling",
        "nested2",
        4,
        1.00,
        false,
        "",
        false,
        false,
        false,
        2,
        true,
        2,
    ),
    spec(
        "two_leg_nested_3leg",
        "3-Leg Sling in Each Leg of 2-Leg Sling",
        "nested3",
        6,
        1.00,
        false,
        "",
        false,
        false,
        false,
        2,
        true,
        3,
    ),
    spec(
        "two_leg_nested_4leg",
        "4-Leg Sling in Each Leg of 2-Leg Sling",
        "nested4",
        8,
        1.00,
        false,
        "",
        false,
        false,
        false,
        2,
        true,
        4,
    ),
    spec(
        "three_leg_direct",
        "3-Leg Bridle Direct",
        "bridle3",
        3,
        1.00,
        false,
        "",
        false,
        false,
        true,
        3,
        true,
        0,
    ),
    spec(
        "four_leg_direct",
        "4-Leg Bridle Direct",
        "bridle4",
        4,
        1.00,
        false,
        "",
        false,
        false,
        true,
        4,
        false,
        0,
    ),
];

/// The arrangement's declared shape, or a named failure.
pub fn hitch_spec(hitch: &str) -> Result<&'static HitchSpec, SlingInputError> {
    HITCH_SPECS
        .iter()
        .find(|spec| spec.key == hitch)
        .ok_or_else(|| err(&format!("Unknown sling arrangement: '{hitch}'")))
}

/// The uniform-load arrangements, in tab order (the first 17 entries; the two
/// bridles close the list).
pub fn uniform_hitch_keys() -> Vec<&'static str> {
    HITCH_SPECS.iter().map(|spec| spec.key).collect()
}

/// User-entered values for one continuous sling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OneLegInputs {
    pub hitch: String,
    pub sling_length: f64,
    pub diameter: f64,
    pub side: f64,
    pub bottom: f64,
    pub choke_angle: f64,
    pub angle_basis: String,
    pub tag_wll: f64,
    pub tag_wll_angle: f64,
    pub pick_distance: f64,
    pub pick_length: f64,
    pub pick_width: f64,
    pub secondary_length: f64,
    pub secondary_spread: f64,
    pub secondary_tag_wll: f64,
    pub secondary_tag_wll_angle: f64,
}

impl Default for OneLegInputs {
    fn default() -> Self {
        Self {
            hitch: "direct_hook".to_string(),
            sling_length: 0.0,
            diameter: 0.0,
            side: 0.0,
            bottom: 0.0,
            choke_angle: 60.0,
            angle_basis: ANGLE_BASIS_INTERNAL.to_string(),
            tag_wll: 0.0,
            tag_wll_angle: 45.0,
            pick_distance: 0.0,
            pick_length: 0.0,
            pick_width: 0.0,
            secondary_length: 0.0,
            secondary_spread: 0.0,
            secondary_tag_wll: 0.0,
            secondary_tag_wll_angle: 45.0,
        }
    }
}

/// The reference inputs for a supported sling arrangement.
pub fn example_inputs(hitch: &str) -> Result<OneLegInputs, SlingInputError> {
    let base = |hitch: &str| OneLegInputs {
        hitch: hitch.to_string(),
        sling_length: 7.0,
        ..OneLegInputs::default()
    };
    let inputs = match hitch {
        "direct_hook" => base(hitch),
        "round_choke" => OneLegInputs {
            diameter: 1.0,
            ..base(hitch)
        },
        "rect_choke" => OneLegInputs {
            side: 0.3,
            bottom: 0.4,
            ..base(hitch)
        },
        "round_basket" | "vertical_round_basket" => OneLegInputs {
            diameter: 1.0,
            ..base(hitch)
        },
        "rect_basket" | "vertical_rect_basket" => OneLegInputs {
            side: 0.3,
            bottom: 0.4,
            ..base(hitch)
        },
        "two_leg_direct" => OneLegInputs {
            pick_distance: 5.0,
            ..base(hitch)
        },
        "two_leg_round_choke" => OneLegInputs {
            diameter: 1.0,
            pick_distance: 5.0,
            ..base(hitch)
        },
        "two_leg_rect_choke" => OneLegInputs {
            side: 0.3,
            bottom: 0.4,
            pick_distance: 5.0,
            ..base(hitch)
        },
        "two_leg_round_basket" | "two_leg_vertical_round_basket" => OneLegInputs {
            diameter: 1.0,
            pick_distance: 5.0,
            ..base(hitch)
        },
        "two_leg_rect_basket" | "two_leg_vertical_rect_basket" => OneLegInputs {
            side: 0.3,
            bottom: 0.4,
            pick_distance: 5.0,
            ..base(hitch)
        },
        "two_leg_nested_2leg" | "two_leg_nested_3leg" | "two_leg_nested_4leg" => OneLegInputs {
            pick_distance: 5.4,
            secondary_length: 0.6,
            secondary_spread: 0.2,
            ..base(hitch)
        },
        "three_leg_direct" => OneLegInputs {
            pick_distance: 5.4,
            ..base(hitch)
        },
        "four_leg_direct" => OneLegInputs {
            pick_length: 5.0,
            pick_width: 2.0,
            ..base(hitch)
        },
        _ => return Err(err(&format!("Unknown sling arrangement: '{hitch}'"))),
    };
    Ok(inputs)
}

/// A number, or a named failure an operator can act on.
fn number(
    value: f64,
    name: &str,
    positive: bool,
    non_negative: bool,
) -> Result<f64, SlingInputError> {
    if !value.is_finite() {
        return Err(err(&format!("{name} must be finite.")));
    }
    if positive && value <= 0.0 {
        return Err(err(&format!("{name} must be greater than 0.")));
    }
    if non_negative && value < 0.0 {
        return Err(err(&format!("{name} cannot be negative.")));
    }
    Ok(value)
}

/// `(external_degrees, internal_degrees)` for a choke angle.
pub fn normalise_angle(value: f64, basis: &str) -> Result<(f64, f64), SlingInputError> {
    let angle = number(value, "Choke angle", false, false)?;
    if basis != ANGLE_BASIS_INTERNAL && basis != ANGLE_BASIS_EXTERNAL {
        return Err(err("Angle basis must be internal or external."));
    }
    if !(angle > 0.0 && angle < 180.0) {
        return Err(err("Choke angle must be between 0 and 180 degrees."));
    }
    Ok(if basis == ANGLE_BASIS_EXTERNAL {
        (angle, 180.0 - angle)
    } else {
        (180.0 - angle, angle)
    })
}

/// The rating multiplier from the example choke-angle table.
///
/// The boundary at 120 degrees is treated as the 100% row.
pub fn choke_reduction_factor(external_angle: f64) -> Result<f64, SlingInputError> {
    let angle = number(external_angle, "External choke angle", false, false)?;
    if !(angle > 0.0 && angle <= 180.0) {
        return Err(err("External choke angle must be between 0 and 180 degrees."));
    }
    Ok(if angle >= 120.0 {
        1.00
    } else if angle >= 90.0 {
        0.87
    } else if angle >= 60.0 {
        0.74
    } else if angle >= 30.0 {
        0.62
    } else {
        0.49
    })
}

fn validate_shape(spec: &HitchSpec, inputs: &OneLegInputs) -> Result<(), SlingInputError> {
    if spec.round_load {
        number(inputs.diameter, "Load diameter", true, false)?;
    }
    if spec.rectangular_load {
        number(inputs.side, "Load side/height", true, false)?;
        number(inputs.bottom, "Load bottom/length", true, false)?;
    }
    if spec.needs_pick_distance {
        number(inputs.pick_distance, "Pick-point distance", true, false)?;
    }
    if spec.nested != 0 {
        number(inputs.secondary_length, "Secondary sling length", true, false)?;
        number(inputs.secondary_spread, "Secondary pick spacing", false, true)?;
    }
    if spec.shape == "bridle4" {
        number(inputs.pick_length, "Pick-point length", true, false)?;
        number(inputs.pick_width, "Pick-point width", true, false)?;
    }
    Ok(())
}

/// One leg group of a nested system, outermost first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecondaryGroup {
    pub offset: f64,
    pub angle: f64,
    pub af: f64,
    pub combined_af: f64,
    pub tension: f64,
    pub governing: bool,
}

/// The figures a report derives from, kept together so they cannot drift apart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlingFormula {
    pub wrap: f64,
    pub free_leg: f64,
    pub angle: f64,
    pub hook_to_load: f64,
    pub tension_each: f64,
    pub max_tension: f64,
    pub required_wll: f64,
}

/// The solved arrangement. The field set mirrors the source's returned values
/// one-for-one, so a reader comparing the two can follow a figure across.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlingResult {
    pub hitch: String,
    pub hitch_label: String,
    pub shape: String,
    pub diameter: f64,
    pub side: f64,
    pub bottom: f64,
    pub pick_distance: f64,
    pub pick_length: f64,
    pub pick_width: f64,
    pub secondary_length: f64,
    pub secondary_spread: f64,
    pub nested: i64,
    pub choke_angle: f64,
    pub angle_basis: String,
    pub load_weight: f64,
    pub sling_length: f64,
    pub wrap_length: f64,
    pub free_leg_length: f64,
    pub angle: f64,
    pub angle_long: Option<f64>,
    pub angle_trans: Option<f64>,
    pub af_long: Option<f64>,
    pub af_trans: Option<f64>,
    pub angle_factor: f64,
    pub base_reeve_factor: f64,
    pub choke_reduction_factor: f64,
    pub combined_reeve_factor: f64,
    pub external_angle: Option<f64>,
    pub internal_angle: Option<f64>,
    pub effective_legs: i64,
    pub sling_legs: i64,
    pub tension_each: f64,
    pub max_tension: f64,
    pub max_tension_basis: String,
    pub required_wll: f64,
    pub rating_legs: i64,
    pub tag_wll_angle: f64,
    pub tag_angle_factor: f64,
    pub leg_capacity: Option<f64>,
    pub secondary_tag_wll: f64,
    pub secondary_tag_wll_angle: f64,
    pub secondary_tag_angle_factor: f64,
    pub secondary_legs: Option<i64>,
    pub secondary_rating_legs: Option<i64>,
    pub secondary_leg_capacity: Option<f64>,
    pub secondary_utilisation: Option<f64>,
    pub secondary_tension: Option<f64>,
    pub secondary_groups: Option<Vec<SecondaryGroup>>,
    pub secondary_required_wll: Option<f64>,
    pub secondary_angle: Option<f64>,
    pub secondary_af: Option<f64>,
    pub secondary_height: Option<f64>,
    pub secondary_height_angle: Option<f64>,
    pub pile_level_difference: Option<f64>,
    pub secondary_height_governing_offset: Option<f64>,
    pub total_combined_length: Option<f64>,
    pub secondary_combined_af: Option<f64>,
    pub document_max_tension: Option<f64>,
    pub document_required_wll: f64,
    pub tag_wll: f64,
    pub utilisation: Option<f64>,
    pub secondary_equipment_status: String,
    pub secondary_equipment_ok: Option<bool>,
    pub equipment_status: String,
    pub equipment_ok: Option<bool>,
    pub choke_to_load: Option<f64>,
    pub hook_to_load: f64,
    pub lead: Option<f64>,
    pub corner_length: Option<f64>,
    pub shape_note: String,
    pub formula: SlingFormula,
}

/// A figure at a fixed number of decimals, with a dot decimal separator.
fn fixed(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// Solve one of the reference sling arrangements.
#[allow(clippy::too_many_lines)]
pub fn solve_one_leg(
    inputs: &OneLegInputs,
    load_weight: f64,
) -> Result<SlingResult, SlingInputError> {
    let spec = hitch_spec(&inputs.hitch)?;
    let weight = number(load_weight, "Gross load at hook", false, true)?;
    let length = number(inputs.sling_length, "Sling total length", true, false)?;
    validate_shape(spec, inputs)?;

    // Normalise every numeric value once, so the geometry below cannot leak a
    // bad value when the model is used outside a form.
    let normalised = OneLegInputs {
        hitch: inputs.hitch.clone(),
        sling_length: length,
        diameter: number(inputs.diameter, "Load diameter", false, true)?,
        side: number(inputs.side, "Load side/height", false, true)?,
        bottom: number(inputs.bottom, "Load bottom/length", false, true)?,
        choke_angle: number(inputs.choke_angle, "Choke angle", false, false)?,
        angle_basis: inputs.angle_basis.clone(),
        tag_wll: number(inputs.tag_wll, "Configured tag WLL", false, true)?,
        tag_wll_angle: number(inputs.tag_wll_angle, "Tag WLL angle", false, false)?,
        secondary_tag_wll: number(inputs.secondary_tag_wll, "Secondary tag WLL", false, true)?,
        secondary_tag_wll_angle: number(
            inputs.secondary_tag_wll_angle,
            "Secondary tag WLL angle",
            false,
            false,
        )?,
        pick_distance: number(inputs.pick_distance, "Pick-point distance", false, true)?,
        pick_length: number(inputs.pick_length, "Pick-point length", false, true)?,
        pick_width: number(inputs.pick_width, "Pick-point width", false, true)?,
        secondary_length: number(
            inputs.secondary_length,
            "Secondary sling length",
            false,
            true,
        )?,
        secondary_spread: number(
            inputs.secondary_spread,
            "Secondary pick spacing",
            false,
            true,
        )?,
    };

    let mut external: Option<f64> = None;
    let mut internal: Option<f64> = None;
    if spec.needs_angle {
        let (ext, int) = normalise_angle(normalised.choke_angle, &normalised.angle_basis)?;
        external = Some(ext);
        internal = Some(int);
        if (normalised.hitch == "rect_choke" || normalised.hitch == "two_leg_rect_choke")
            && !(int > 0.0 && int < 90.0)
        {
            return Err(err(
                "Rectangular choke flare must be between 0 and 90 degrees \
                 from vertical (the external angle must be above 90 degrees).",
            ));
        }
    }
    let reduction = match external {
        Some(angle) => choke_reduction_factor(angle)?,
        None => 1.0,
    };

    let mut wrap = 0.0;
    let mut free_leg = length;
    let mut angle = 0.0;
    let mut hook_to_load = length;
    let mut choke_to_load: Option<f64> = None;
    let mut corner_length: Option<f64> = None;
    let mut angle_trans: Option<f64> = None;
    let mut angle_long: Option<f64> = None;
    let mut af_long: Option<f64> = None;
    let mut af_trans: Option<f64> = None;
    let mut precomputed_angle_factor: Option<f64> = None;
    let mut tension_override: Option<f64> = None;
    let mut secondary_tension: Option<f64> = None;
    let mut secondary_required_wll: Option<f64> = None;
    let mut secondary_angle: Option<f64> = None;
    let mut secondary_af: Option<f64> = None;
    let mut secondary_groups: Option<Vec<SecondaryGroup>> = None;
    let mut pile_level_difference: Option<f64> = None;
    let mut governing_index = 0usize;
    // A lead/contact length is meaningful only for the two choke arrangements.
    let mut lead: Option<f64> = None;
    let mut shape_note = String::new();
    // Set only by the nested branch, and read back under `spec.nested != 0`.
    let mut offsets: Option<Vec<f64>> = None;
    let mut height_sling2 = 0.0;
    let mut height_angle = 0.0;
    let mut total_combined = 0.0;

    if normalised.hitch == "direct_hook" {
        shape_note = "A single vertical leg runs from the hook to the load.".to_string();
    } else if normalised.hitch == "three_leg_direct" {
        let radius_spread = normalised.pick_distance / 3.0f64.sqrt();
        let ratio = radius_spread / length;
        if ratio > 1.0 {
            return Err(err(
                "The 3-leg bridle cannot reach the hook with this sling length \
                 and pick-point spacing.",
            ));
        }
        angle = ratio.asin().to_degrees();
        hook_to_load = length * published_af(angle.to_radians().cos());
        if hook_to_load <= 0.0 {
            return Err(err(
                "The 3-leg bridle needs a shorter pick-point spacing or a \
                 longer sling to provide headroom.",
            ));
        }
        shape_note = "Three legs share the load from a uniform pick-point circle.".to_string();
    } else if normalised.hitch == "four_leg_direct" {
        let radius_spread = (normalised.pick_length / 2.0)
            .hypot(normalised.pick_width / 2.0);
        let ratio = radius_spread / length;
        if ratio > 1.0 {
            return Err(err(
                "The 4-leg bridle cannot reach the hook with this sling length \
                 and pick-point rectangle.",
            ));
        }
        angle = ratio.asin().to_degrees();
        hook_to_load = length * published_af(angle.to_radians().cos());
        if hook_to_load <= 0.0 {
            return Err(err(
                "The 4-leg bridle needs a smaller pick-point rectangle or a \
                 longer sling to provide headroom.",
            ));
        }
        shape_note = "Four legs share the load from the four corners of the \
                      pick-point rectangle."
            .to_string();
    } else if spec.nested != 0 {
        let n = spec.nested;
        let free2 = normalised.secondary_length;
        let spread = normalised.secondary_spread;
        let current_offsets: Vec<f64> = match n {
            2 => vec![spread / 2.0],
            3 => vec![0.0, spread],
            _ => vec![spread / 2.0, 1.5 * spread],
        };
        offsets = Some(current_offsets.clone());
        let mut trans_angles: Vec<f64> = Vec::new();
        let mut trans_factors: Vec<f64> = Vec::new();
        for offset in &current_offsets {
            let ratio_t = offset / free2;
            if ratio_t > 1.0 {
                return Err(err(
                    "The secondary sling cannot reach the pile spacing entered.",
                ));
            }
            trans_angles.push(ratio_t.asin().to_degrees());
            trans_factors.push(published_af(trans_angles.last().unwrap().to_radians().cos()));
        }
        // Every secondary leg is cut to the same free length and they all meet
        // at one master link, so the pile tops they land on cannot be level. The
        // leg with the largest offset carries the highest tension, and is the
        // conservative reference for the master legs.
        governing_index = current_offsets
            .iter()
            .enumerate()
            .fold(None::<(usize, f64)>, |best, (index, offset)| match best {
                None => Some((index, *offset)),
                Some((_, value)) if *offset > value => Some((index, *offset)),
                Some(existing) => Some(existing),
            })
            .map(|(index, _)| index)
            .unwrap_or(0);
        height_sling2 = free2 * trans_factors[governing_index];
        height_angle = trans_angles[governing_index];
        // Spread between the highest and the lowest pile top under the link.
        let max_factor = trans_factors.iter().copied().fold(f64::MIN, f64::max);
        let min_factor = trans_factors.iter().copied().fold(f64::MAX, f64::min);
        pile_level_difference = Some(free2 * (max_factor - min_factor));
        total_combined = length + height_sling2;
        let ratio = (normalised.pick_distance / 2.0) / total_combined;
        if ratio > 1.0 {
            return Err(err(
                "The nested sling system cannot reach the hook with this \
                 master length and pick-point spread.",
            ));
        }
        angle_long = Some(ratio.asin().to_degrees());
        af_long = Some(published_af(angle_long.unwrap().to_radians().cos()));
        // The reference documents multiply the published longitudinal and
        // transverse factors to reach the combined secondary-leg factor.
        let combined: Vec<f64> = trans_factors
            .iter()
            .map(|factor| published_combined_af(af_long.unwrap(), *factor))
            .collect();
        let worst_index = combined
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index)
            .unwrap_or(0);
        precomputed_angle_factor = af_long;
        tension_override = Some(weight / (2.0 * af_long.unwrap()));
        secondary_tension = Some(
            weight / (spec.effective_legs as f64 * combined[worst_index]),
        );
        secondary_required_wll = secondary_tension;
        secondary_angle = Some(trans_angles[worst_index]);
        secondary_af = Some(trans_factors[worst_index]);
        // A nested system with more than one transverse offset reports every leg
        // group, outermost first, as the reference documents do.
        let mut order: Vec<usize> = (0..current_offsets.len()).collect();
        order.sort_by(|a, b| {
            current_offsets[*b]
                .partial_cmp(&current_offsets[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        secondary_groups = Some(
            order
                .iter()
                .map(|&i| SecondaryGroup {
                    offset: current_offsets[i],
                    angle: trans_angles[i],
                    af: trans_factors[i],
                    combined_af: combined[i],
                    tension: weight / (spec.effective_legs as f64 * combined[i]),
                    governing: i == worst_index,
                })
                .collect(),
        );
        angle = angle_long.unwrap();
        angle_trans = Some(trans_angles[worst_index]);
        af_trans = Some(trans_factors[worst_index]);
        // The headroom is a printed line multiplied by a printed factor.
        hook_to_load = published_length(total_combined) * af_long.unwrap();
        shape_note = "A 2-leg master bridle splits into two secondary slings; \
                      the highest secondary-leg tension is shown."
            .to_string();
    } else if normalised.hitch == "two_leg_direct" {
        let half_span = normalised.pick_distance / 2.0;
        let ratio = half_span / length;
        if ratio > 1.0 {
            return Err(err(
                "The 2-leg bridle cannot reach the hook with this sling length \
                 and pick-point distance.",
            ));
        }
        angle = ratio.asin().to_degrees();
        hook_to_load = length * published_af(angle.to_radians().cos());
        if hook_to_load <= 0.0 {
            return Err(err(
                "The 2-leg bridle needs a shorter pick-point distance or a \
                 longer sling to provide headroom.",
            ));
        }
        shape_note = "Two legs share the load from the pick points.".to_string();
    } else if normalised.hitch == "two_leg_round_choke"
        || normalised.hitch == "two_leg_rect_choke"
    {
        let (Some(ext), Some(int)) = (external, internal) else {
            return Err(err("A 2-leg choke needs a choke angle."));
        };
        if normalised.hitch == "two_leg_round_choke" {
            let radius = normalised.diameter / 2.0;
            wrap = PI * normalised.diameter;
            lead = Some(length - wrap);
            free_leg = lead.unwrap();
            if lead.unwrap() <= 0.0 {
                return Err(err(&format!(
                    "The sling is too short to complete the 2-leg round choke; \
                     at least {} m is required.",
                    fixed(wrap, 3)
                )));
            }
            choke_to_load = Some(if ext >= 90.0 {
                (radius / int.to_radians().sin() - radius).max(0.0)
            } else {
                0.0
            });
            shape_note = "Two legs choke the round load on a full wrap.".to_string();
        } else {
            corner_length = Some((normalised.bottom / 2.0) / int.to_radians().sin());
            wrap = normalised.bottom + 2.0 * normalised.side + 2.0 * corner_length.unwrap();
            lead = Some(length - wrap);
            free_leg = lead.unwrap();
            if lead.unwrap() <= 0.0 {
                return Err(err(&format!(
                    "The sling is too short for the 2-leg rectangular choke \
                     and flare; at least {} m is required.",
                    fixed(wrap, 3)
                )));
            }
            let mut gap = (normalised.bottom / 2.0) / int.to_radians().tan();
            if ext < 90.0 {
                gap = 0.0;
            }
            choke_to_load = Some(gap);
            shape_note =
                "Two legs choke the rectangular load at the corner strands.".to_string();
        }
        let vertical_span = free_leg + choke_to_load.unwrap_or(0.0);
        let ratio = (normalised.pick_distance / 2.0) / vertical_span;
        if ratio > 1.0 {
            return Err(err(
                "The 2-leg choke cannot reach the hook with this sling length \
                 and pick-point distance.",
            ));
        }
        angle = ratio.asin().to_degrees();
        hook_to_load = free_leg * angle.to_radians().cos();
        if hook_to_load <= 0.0 {
            return Err(err(
                "The 2-leg choke needs more free leg to provide headroom above \
                 the load.",
            ));
        }
    } else if normalised.hitch == "two_leg_round_basket"
        || normalised.hitch == "two_leg_vertical_round_basket"
        || normalised.hitch == "two_leg_rect_basket"
        || normalised.hitch == "two_leg_vertical_rect_basket"
    {
        let vertical = normalised.hitch.starts_with("two_leg_vertical");
        let headroom: f64;
        if normalised.hitch.ends_with("round_basket") {
            let radius = normalised.diameter / 2.0;
            wrap = PI * radius;
            let remaining = length - wrap;
            if remaining <= 0.0 {
                return Err(err(&format!(
                    "The sling is too short to wrap around the round load; \
                     at least {} m is required.",
                    fixed(wrap, 3)
                )));
            }
            free_leg = remaining / (spec.effective_legs as f64 / spec.leg_count as f64);
            angle_long = Some(if vertical {
                0.0
            } else {
                ((normalised.pick_distance / 2.0) / free_leg).asin().to_degrees()
            });
            angle_trans = Some((radius / free_leg).asin().to_degrees());
            headroom = free_leg
                * published_combined_af(
                    angle_long.unwrap().to_radians().cos(),
                    angle_trans.unwrap().to_radians().cos(),
                )
                - radius;
            if headroom <= 0.0 {
                return Err(err(
                    "The 2-leg round basket needs more free leg to provide \
                     headroom above the load.",
                ));
            }
            shape_note = "Two basket slings cradle the round load.".to_string();
        } else {
            wrap = normalised.bottom + 2.0 * normalised.side;
            let remaining = length - wrap;
            if remaining <= 0.0 {
                return Err(err(&format!(
                    "The sling is too short to wrap around the rectangular \
                     load; at least {} m is required.",
                    fixed(wrap, 3)
                )));
            }
            free_leg = remaining / (spec.effective_legs as f64 / spec.leg_count as f64);
            angle_long = Some(if vertical {
                0.0
            } else {
                ((normalised.pick_distance / 2.0) / free_leg).asin().to_degrees()
            });
            angle_trans = Some(((normalised.bottom / 2.0) / free_leg).asin().to_degrees());
            headroom = free_leg
                * published_combined_af(
                    angle_long.unwrap().to_radians().cos(),
                    angle_trans.unwrap().to_radians().cos(),
                )
                - if vertical { normalised.side } else { 0.0 };
            if headroom <= 0.0 {
                return Err(err(
                    "The 2-leg rectangular basket needs more free leg to \
                     provide headroom above the load.",
                ));
            }
            shape_note = "Two basket slings cradle the rectangular load.".to_string();
        }
        angle = angle_long.unwrap();
        hook_to_load = headroom;
        af_long = Some(published_af(angle_long.unwrap().to_radians().cos()));
        af_trans = Some(published_af(angle_trans.unwrap().to_radians().cos()));
        precomputed_angle_factor = Some(published_combined_af(af_long.unwrap(), af_trans.unwrap()));
        if precomputed_angle_factor.unwrap() <= 0.0 {
            return Err(err("The sling angle is too close to horizontal."));
        }
    } else if normalised.hitch == "round_basket"
        || normalised.hitch == "vertical_round_basket"
    {
        let radius = normalised.diameter / 2.0;
        wrap = PI * radius;
        let remaining = length - wrap;
        if remaining <= 0.0 {
            return Err(err(&format!(
                "The sling is too short to wrap around the round load; at \
                 least {} m is required.",
                fixed(wrap, 3)
            )));
        }
        free_leg = remaining / 2.0;
        if normalised.hitch == "round_basket" {
            let ratio = radius / free_leg;
            if ratio > 1.0 {
                return Err(err(
                    "The round basket cannot reach the hook with this length.",
                ));
            }
            angle = ratio.asin().to_degrees();
            hook_to_load = free_leg * published_af(angle.to_radians().cos()) - radius;
            if hook_to_load <= 0.0 {
                return Err(err(
                    "The round basket needs more free leg to provide headroom \
                     above the load.",
                ));
            }
            shape_note = "The sling cradles half of the round load.".to_string();
        } else {
            angle = 0.0;
            hook_to_load = free_leg - radius;
            if hook_to_load <= 0.0 {
                return Err(err(
                    "The vertical round basket needs more free leg to clear the load.",
                ));
            }
            shape_note = "The two legs rise vertically to a spreader bar.".to_string();
        }
    } else if normalised.hitch == "rect_basket"
        || normalised.hitch == "vertical_rect_basket"
    {
        wrap = normalised.bottom + 2.0 * normalised.side;
        let remaining = length - wrap;
        if remaining <= 0.0 {
            return Err(err(&format!(
                "The sling is too short to wrap around the rectangular load; \
                 at least {} m is required.",
                fixed(wrap, 3)
            )));
        }
        free_leg = remaining / 2.0;
        if normalised.hitch == "rect_basket" {
            let ratio = (normalised.bottom / 2.0) / free_leg;
            if ratio > 1.0 {
                return Err(err(
                    "The rectangular basket cannot reach the hook with this length.",
                ));
            }
            angle = ratio.asin().to_degrees();
            hook_to_load = free_leg * published_af(angle.to_radians().cos());
            if hook_to_load <= 0.0 {
                return Err(err(
                    "The rectangular basket needs more free leg to provide \
                     headroom above the load.",
                ));
            }
            shape_note = "The sling cradles the bottom and two sides of the load.".to_string();
        } else {
            angle = 0.0;
            hook_to_load = free_leg - normalised.side;
            if hook_to_load <= 0.0 {
                return Err(err(
                    "The vertical rectangular basket needs more free leg to \
                     clear the load.",
                ));
            }
            shape_note = "The two legs rise vertically to a spreader bar.".to_string();
        }
    } else if normalised.hitch == "round_choke" {
        // The round choke uses a complete circumference. The main lifting line
        // remains vertical, even when the choke's contact angle changes.
        let radius = normalised.diameter / 2.0;
        wrap = PI * normalised.diameter;
        lead = Some(length - wrap);
        free_leg = lead.unwrap();
        if lead.unwrap() <= 0.0 {
            return Err(err(&format!(
                "The sling is too short to complete the round choke; at least \
                 {} m is required.",
                fixed(wrap, 3)
            )));
        }
        // For an external angle below 90 degrees the field rule in the
        // reference document declares the choke flush against the load.
        choke_to_load = Some(if external.unwrap() >= 90.0 {
            (radius / internal.unwrap().to_radians().sin() - radius).max(0.0)
        } else {
            0.0
        });
        hook_to_load = lead.unwrap() + choke_to_load.unwrap_or(0.0);
        angle = 0.0;
        shape_note = "The noose completes one full circumference around the load.".to_string();
    } else if normalised.hitch == "rect_choke" {
        let (Some(int), Some(ext)) = (internal, external) else {
            return Err(err("A rectangular choke needs a choke angle."));
        };
        corner_length = Some((normalised.bottom / 2.0) / int.to_radians().sin());
        wrap = normalised.bottom + 2.0 * normalised.side + 2.0 * corner_length.unwrap();
        lead = Some(length - wrap);
        free_leg = lead.unwrap();
        if lead.unwrap() <= 0.0 {
            return Err(err(&format!(
                "The sling is too short for the rectangular choke and flare; \
                 at least {} m is required.",
                fixed(wrap, 3)
            )));
        }
        let mut gap = (normalised.bottom / 2.0) / int.to_radians().tan();
        if ext < 90.0 {
            gap = 0.0;
        }
        choke_to_load = Some(gap);
        hook_to_load = lead.unwrap() + choke_to_load.unwrap_or(0.0);
        angle = 0.0;
        shape_note =
            "The two corner strands flare from the load to the choke point.".to_string();
    }

    // The report prints the angle factor at three decimals, and the reference
    // documents derive every printed force from that published factor, so the
    // same factor drives the results returned here.
    let angle_factor = published_af(precomputed_angle_factor.unwrap_or(angle.to_radians().cos()));
    if angle_factor <= 0.0 {
        return Err(err("The sling angle is too close to horizontal."));
    }

    let effective_legs = spec.effective_legs;
    // One-leg arrangements are rated as one continuous sling and a 2-leg
    // arrangement as two slings. A 3- or 4-leg bridle is rated for the case
    // where one leg does not take its share, so its tag rating is the N-1 case.
    let sling_legs = if spec.leg_count > 1 { spec.leg_count } else { 1 };
    let rating_legs = if spec.bridle { rated_legs(sling_legs) } else { sling_legs };
    let tension_each = tension_override.unwrap_or(weight / (effective_legs as f64 * angle_factor));
    let combined_factor = spec.base_reeve_factor * reduction;
    let required_wll = weight / (rating_legs as f64 * combined_factor * angle_factor);

    // In the reference reports the displayed applied tension is the resolved
    // force path. The required tag WLL is a rating requirement, not a second
    // force path, so the two values are kept separate and explicit.
    let max_tension = match secondary_tension {
        Some(secondary) => tension_each.max(secondary),
        None => tension_each,
    };

    let tag_wll = number(normalised.tag_wll, "Configured tag WLL", false, true)?;
    let mut tag_angle = number(normalised.tag_wll_angle, "Tag WLL angle", false, false)?;
    // A single leg hangs vertical and takes the tag's full rating, so it is
    // rated at no angle. Enforced here rather than only in the form, so a
    // caller cannot derate a one-leg sling by a leftover 2-leg angle.
    if !spec.needs_tag_angle() {
        tag_angle = 0.0;
    }

    let secondary_tag_wll = number(normalised.secondary_tag_wll, "Secondary tag WLL", false, true)?;
    let secondary_tag_angle = number(
        normalised.secondary_tag_wll_angle,
        "Secondary tag WLL angle",
        false,
        false,
    )?;
    let tag_af = published_af(tag_angle.to_radians().cos());
    let secondary_tag_af = published_af(secondary_tag_angle.to_radians().cos());
    let leg_capacity_value = leg_capacity(tag_wll, rating_legs, tag_af);

    // The secondary slings of a nested system have their own tag rating. Each
    // secondary sling is itself a 2-, 3- or 4-leg sling closing onto a master
    // leg, so the note's rule applies to it exactly as it does to a bridle of
    // the same size: a 3-leg second sling is rated on 2 legs and a 4-leg on 3.
    let secondary_legs = if spec.nested != 0 {
        Some(effective_legs / spec.leg_count)
    } else {
        None
    };
    let secondary_rating_legs = if spec.nested != 0 {
        Some(rated_legs(spec.nested))
    } else {
        None
    };

    let secondary_capacity = leg_capacity(
        secondary_tag_wll,
        secondary_rating_legs.unwrap_or(0),
        secondary_tag_af,
    );
    let utilisation = tag_utilisation(required_wll, leg_capacity_value);
    let secondary_utilisation = tag_utilisation(secondary_required_wll.unwrap_or(0.0), secondary_capacity);

    let equipment_ok: Option<bool> = if leg_capacity_value.is_some() {
        Some(utilisation.map(|value| value <= 100.0).unwrap_or(false))
    } else {
        None
    };
    let equipment_status = if leg_capacity_value.is_none() {
        "No tag WLL entered".to_string()
    } else if equipment_ok == Some(true) {
        "PASS \u{2014} configured WLL meets the requirement".to_string()
    } else {
        "FAIL \u{2014} configured WLL is below the requirement".to_string()
    };

    let secondary_equipment_ok: Option<bool> = match secondary_capacity {
        None => None,
        Some(_) => Some(secondary_utilisation.unwrap_or(0.0) <= 100.0),
    };
    let secondary_equipment_status = if secondary_capacity.is_none() {
        "No secondary tag WLL entered".to_string()
    } else if secondary_utilisation.unwrap_or(0.0) <= 100.0 {
        "PASS \u{2014} secondary tag WLL meets the requirement".to_string()
    } else {
        "FAIL \u{2014} secondary tag WLL is below the requirement".to_string()
    };

    let secondary_combined_af = match (secondary_af, af_long) {
        (Some(secondary), Some(longitudinal)) => {
            Some(published_combined_af(longitudinal, secondary))
        }
        _ => None,
    };

    Ok(SlingResult {
        hitch: normalised.hitch.clone(),
        hitch_label: spec.label.to_string(),
        shape: spec.shape.to_string(),
        diameter: normalised.diameter,
        side: normalised.side,
        bottom: normalised.bottom,
        pick_distance: normalised.pick_distance,
        pick_length: normalised.pick_length,
        pick_width: normalised.pick_width,
        secondary_length: normalised.secondary_length,
        secondary_spread: normalised.secondary_spread,
        nested: spec.nested,
        choke_angle: normalised.choke_angle,
        angle_basis: normalised.angle_basis.clone(),
        load_weight: weight,
        sling_length: length,
        wrap_length: wrap,
        free_leg_length: free_leg,
        angle,
        angle_long,
        angle_trans,
        af_long,
        af_trans,
        angle_factor,
        base_reeve_factor: spec.base_reeve_factor,
        choke_reduction_factor: reduction,
        combined_reeve_factor: combined_factor,
        external_angle: external,
        internal_angle: internal,
        effective_legs,
        sling_legs,
        tension_each,
        max_tension,
        max_tension_basis: "equal_to_tension_each".to_string(),
        required_wll,
        rating_legs,
        tag_wll_angle: tag_angle,
        tag_angle_factor: tag_af,
        leg_capacity: leg_capacity_value,
        secondary_tag_wll,
        secondary_tag_wll_angle: secondary_tag_angle,
        secondary_tag_angle_factor: secondary_tag_af,
        secondary_legs,
        secondary_rating_legs,
        secondary_leg_capacity: secondary_capacity,
        secondary_utilisation,
        secondary_tension,
        secondary_groups,
        secondary_required_wll,
        secondary_angle,
        secondary_af,
        secondary_height: if spec.nested != 0 {
            Some(height_sling2)
        } else {
            None
        },
        secondary_height_angle: if spec.nested != 0 {
            Some(height_angle)
        } else {
            None
        },
        pile_level_difference: if spec.nested != 0 && spec.nested > 2 {
            pile_level_difference
        } else {
            None
        },
        secondary_height_governing_offset: if spec.nested != 0 && spec.nested > 2 {
            offsets.map(|values| values[governing_index])
        } else {
            None
        },
        total_combined_length: if spec.nested != 0 {
            Some(total_combined)
        } else {
            None
        },
        secondary_combined_af,
        // Reserved distinction between a value printed by a reference document
        // and a value calculated by the model.
        document_max_tension: None,
        document_required_wll: required_wll,
        tag_wll,
        utilisation,
        secondary_equipment_status,
        secondary_equipment_ok,
        equipment_status,
        equipment_ok,
        choke_to_load,
        hook_to_load,
        lead,
        corner_length,
        shape_note,
        formula: SlingFormula {
            wrap,
            free_leg,
            angle,
            hook_to_load,
            tension_each,
            max_tension,
            required_wll,
        },
    })
}

/// Convenience helper used by tests and the example loader.
pub fn solve_defaults(hitch: &str, load_weight: f64) -> Result<SlingResult, SlingInputError> {
    solve_one_leg(&example_inputs(hitch)?, load_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-6;

    #[test]
    fn round_half_to_even_matches_python() {
        assert_eq!(round_half_even(0.5, 0), 0.0);
        assert_eq!(round_half_even(1.5, 0), 2.0);
        assert_eq!(round_half_even(2.675, 2), 2.68);
    }

    #[test]
    fn unknown_hitch_is_a_named_failure() {
        let inputs = OneLegInputs {
            hitch: "teleport".to_string(),
            ..OneLegInputs::default()
        };
        assert_eq!(
            solve_one_leg(&inputs, 100.0).unwrap_err().0,
            "Unknown sling arrangement: 'teleport'"
        );
    }

    #[test]
    fn zero_length_is_refused() {
        let inputs = OneLegInputs {
            hitch: "direct_hook".to_string(),
            sling_length: 0.0,
            ..OneLegInputs::default()
        };
        assert_eq!(
            solve_one_leg(&inputs, 100.0).unwrap_err().0,
            "Sling total length must be greater than 0."
        );
    }

    #[test]
    fn every_arrangement_matches_the_reference() {
        let expected: [(&str, f64, f64, f64, f64, i64, i64, f64); 19] = [
            ("direct_hook", 1.0, 2000.0, 2000.0, 7.0, 1, 1, 0.0),
            ("round_choke", 1.0, 2000.0, 2666.6666666666665, 3.9357576155998326, 1, 1, 0.0),
            ("rect_choke", 1.0, 2000.0, 4000.0, 5.653589838486225, 1, 1, 0.0),
            ("round_basket", 0.983, 1017.293997965412, 1017.293997965412, 2.1684536053803085, 2, 1, 10.613857356461029),
            ("rect_basket", 0.998, 1002.0040080160321, 2004.0080160320642, 2.9939999999999998, 2, 1, 3.822553729274344),
            ("vertical_round_basket", 1.0, 1000.0, 1000.0, 2.2146018366025517, 2, 1, 0.0),
            ("vertical_rect_basket", 1.0, 1000.0, 2000.0, 2.7, 2, 1, 0.0),
            ("two_leg_direct", 0.934, 1070.663811563169, 1070.663811563169, 6.538, 2, 2, 20.924832427638318),
            ("two_leg_round_choke", 0.772, 1295.3367875647668, 1727.1157167530225, 2.980026563014584, 2, 2, 39.43494713747446),
            ("two_leg_rect_choke", 0.897, 1114.8272017837235, 2229.654403567447, 4.967239281902605, 2, 2, 26.24413735040033),
            ("two_leg_round_basket", 0.383, 1305.4830287206266, 1305.4830287206266, 0.5396925034187774, 4, 2, 67.06468259794082),
            ("two_leg_rect_basket", 0.552, 905.7971014492753, 1811.5942028985505, 1.6560000000000001, 4, 2, 56.44269023807929),
            ("two_leg_vertical_round_basket", 0.983, 508.646998982706, 508.646998982706, 2.1684536053803085, 4, 2, 0.0),
            ("two_leg_vertical_rect_basket", 0.998, 501.00200400801606, 1002.0040080160321, 2.694, 4, 2, 0.0),
            ("two_leg_nested_2leg", 0.935, 1069.51871657754, 1069.51871657754, 7.09852, 4, 2, 20.833669881768763),
            ("two_leg_nested_3leg", 0.934, 1070.663811563169, 1070.663811563169, 7.066644, 6, 2, 20.90803884369347),
            ("two_leg_nested_4leg", 0.933, 1071.8113612004286, 1071.8113612004286, 7.01616, 8, 2, 21.042580058290127),
            ("three_leg_direct", 0.895, 744.8789571694599, 1117.31843575419, 6.265000000000001, 3, 2, 26.447941163157644),
            ("four_leg_direct", 0.923, 541.7118093174431, 722.2824124232575, 6.461, 4, 3, 22.622300890389113),
        ];

        let order: Vec<&str> = HITCH_SPECS.iter().map(|spec| spec.key).collect();
        let expected_order: Vec<&str> = expected.iter().map(|row| row.0).collect();
        assert_eq!(order, expected_order);

        for (hitch, af, tension, required, hook, eff, rating, angle) in expected {
            let result = solve_defaults(hitch, 2000.0).unwrap();
            assert!((result.angle_factor - af).abs() < TOL, "{hitch} AF");
            assert!((result.tension_each - tension).abs() < TOL, "{hitch} tension");
            assert!((result.required_wll - required).abs() < TOL, "{hitch} required");
            assert!((result.hook_to_load - hook).abs() < TOL, "{hitch} headroom");
            assert_eq!(result.effective_legs, eff, "{hitch} effective legs");
            assert_eq!(result.rating_legs, rating, "{hitch} rating legs");
            assert!((result.angle - angle).abs() < TOL, "{hitch} angle");
        }
    }

    #[test]
    fn the_nested_systems_report_their_secondary_legs() {
        let two = solve_defaults("two_leg_nested_2leg", 2000.0).unwrap();
        assert!((two.secondary_af.unwrap() - 0.986).abs() < TOL);
        assert!((two.secondary_combined_af.unwrap() - 0.922).abs() < TOL);
        assert!((two.secondary_tension.unwrap() - 542.2993492407809).abs() < TOL);
        assert!((two.secondary_angle.unwrap() - 9.594068226860463).abs() < TOL);
        assert_eq!(two.secondary_legs, Some(2));
        assert_eq!(two.secondary_rating_legs, Some(2));
        assert!((two.secondary_height.unwrap() - 0.5916).abs() < TOL);
        assert!((two.total_combined_length.unwrap() - 7.5916).abs() < TOL);
        assert_eq!(two.pile_level_difference, None);
        assert_eq!(two.secondary_groups.as_ref().unwrap().len(), 1);

        let three = solve_defaults("two_leg_nested_3leg", 2000.0).unwrap();
        assert!((three.secondary_combined_af.unwrap() - 0.881).abs() < TOL);
        assert!((three.secondary_tension.unwrap() - 378.35792659856224).abs() < TOL);
        assert_eq!(three.secondary_legs, Some(3));
        assert_eq!(three.secondary_rating_legs, Some(2));
        assert!((three.secondary_height.unwrap() - 0.5658).abs() < TOL);
        assert!((three.pile_level_difference.unwrap() - 0.03420000000000003).abs() < TOL);
        let groups = three.secondary_groups.as_ref().unwrap();
        assert_eq!(groups.len(), 2);
        assert!((groups[0].offset - 0.2).abs() < TOL);
        assert!(groups[0].governing);
        assert!(!groups[1].governing);

        let four = solve_defaults("two_leg_nested_4leg", 2000.0).unwrap();
        assert!((four.secondary_combined_af.unwrap() - 0.808).abs() < TOL);
        assert!((four.secondary_tension.unwrap() - 309.40594059405936).abs() < TOL);
        assert_eq!(four.secondary_legs, Some(4));
        assert_eq!(four.secondary_rating_legs, Some(3));
        assert!((four.pile_level_difference.unwrap() - 0.072).abs() < TOL);
        assert!((four.secondary_height_governing_offset.unwrap() - 0.3).abs() < TOL);
    }

    #[test]
    fn a_one_leg_sling_is_never_derated_by_a_leftover_tag_angle() {
        let inputs = OneLegInputs {
            hitch: "direct_hook".to_string(),
            sling_length: 7.0,
            tag_wll: 5000.0,
            tag_wll_angle: 45.0,
            ..OneLegInputs::default()
        };
        let result = solve_one_leg(&inputs, 2000.0).unwrap();
        assert!((result.tag_wll_angle - 0.0).abs() < TOL);
        assert!((result.tag_angle_factor - 1.0).abs() < TOL);
        assert!((result.leg_capacity.unwrap() - 5000.0).abs() < TOL);
        assert!((result.utilisation.unwrap() - 40.0).abs() < TOL);
        assert_eq!(
            result.equipment_status,
            "PASS \u{2014} configured WLL meets the requirement"
        );
    }

    #[test]
    fn a_two_leg_sling_does_take_its_rated_angle() {
        let inputs = OneLegInputs {
            hitch: "two_leg_direct".to_string(),
            sling_length: 7.0,
            pick_distance: 5.0,
            tag_wll: 5300.0,
            tag_wll_angle: 45.0,
            ..OneLegInputs::default()
        };
        let result = solve_one_leg(&inputs, 2000.0).unwrap();
        assert!((result.tag_wll_angle - 45.0).abs() < TOL);
        assert!((result.tag_angle_factor - 0.707).abs() < TOL);
        assert!((result.leg_capacity.unwrap() - 3748.2319660537482).abs() < TOL);
        assert!((result.utilisation.unwrap() - 28.56571154153994).abs() < TOL);
    }

    #[test]
    fn a_bridle_is_rated_on_one_leg_fewer_than_it_has() {
        assert_eq!(rated_legs(1), 1);
        assert_eq!(rated_legs(2), 2);
        assert_eq!(rated_legs(3), 2);
        assert_eq!(rated_legs(4), 3);
        assert_eq!(rated_legs(5), 4);
        assert!(!hitch_spec("direct_hook").unwrap().needs_tag_angle());
        assert!(!hitch_spec("round_choke").unwrap().needs_tag_angle());
        assert!(hitch_spec("two_leg_direct").unwrap().needs_tag_angle());
        assert!(hitch_spec("four_leg_direct").unwrap().needs_tag_angle());
    }

    #[test]
    fn the_choke_reduction_table_keeps_its_boundaries() {
        assert!((choke_reduction_factor(150.0).unwrap() - 1.00).abs() < TOL);
        assert!((choke_reduction_factor(120.0).unwrap() - 1.00).abs() < TOL);
        assert!((choke_reduction_factor(119.0).unwrap() - 0.87).abs() < TOL);
        assert!((choke_reduction_factor(90.0).unwrap() - 0.87).abs() < TOL);
        assert!((choke_reduction_factor(89.0).unwrap() - 0.74).abs() < TOL);
        assert!((choke_reduction_factor(60.0).unwrap() - 0.74).abs() < TOL);
        assert!((choke_reduction_factor(59.0).unwrap() - 0.62).abs() < TOL);
        assert!((choke_reduction_factor(30.0).unwrap() - 0.62).abs() < TOL);
        assert!((choke_reduction_factor(29.0).unwrap() - 0.49).abs() < TOL);
        assert!((choke_reduction_factor(10.0).unwrap() - 0.49).abs() < TOL);
    }

    #[test]
    fn the_two_angle_bases_are_two_readings_of_one_angle() {
        let (first, second) = normalise_angle(60.0, ANGLE_BASIS_INTERNAL).unwrap();
        assert!((first - 120.0).abs() < TOL);
        assert!((second - 60.0).abs() < TOL);
        let (first, second) = normalise_angle(120.0, ANGLE_BASIS_EXTERNAL).unwrap();
        assert!((first - 120.0).abs() < TOL);
        assert!((second - 60.0).abs() < TOL);
        assert!(normalise_angle(60.0, "sideways").is_err());
        assert!(normalise_angle(0.0, ANGLE_BASIS_INTERNAL).is_err());
        assert!(normalise_angle(180.0, ANGLE_BASIS_INTERNAL).is_err());
    }

    #[test]
    fn an_arrangement_that_cannot_reach_the_hook_says_so() {
        let inputs = OneLegInputs {
            hitch: "three_leg_direct".to_string(),
            sling_length: 1.0,
            pick_distance: 5.4,
            ..OneLegInputs::default()
        };
        assert_eq!(
            solve_one_leg(&inputs, 2000.0).unwrap_err().0,
            "The 3-leg bridle cannot reach the hook with this sling length and pick-point spacing."
        );
    }

    #[test]
    fn a_sling_too_short_for_the_wrap_names_the_length_it_needs() {
        let inputs = OneLegInputs {
            hitch: "round_choke".to_string(),
            sling_length: 3.0,
            diameter: 1.0,
            ..OneLegInputs::default()
        };
        assert_eq!(
            solve_one_leg(&inputs, 2000.0).unwrap_err().0,
            "The sling is too short to complete the round choke; at least 3.142 m is required."
        );
    }

    #[test]
    fn a_rectangular_choke_flare_must_stay_inside_the_vertical() {
        let inputs = OneLegInputs {
            hitch: "rect_choke".to_string(),
            sling_length: 7.0,
            side: 0.3,
            bottom: 0.4,
            choke_angle: 90.0,
            ..OneLegInputs::default()
        };
        assert!(solve_one_leg(&inputs, 2000.0).is_err());
    }

    #[test]
    fn a_missing_shape_measurement_is_named_rather_than_treated_as_zero() {
        let inputs = OneLegInputs {
            hitch: "round_basket".to_string(),
            sling_length: 7.0,
            ..OneLegInputs::default()
        };
        assert_eq!(
            solve_one_leg(&inputs, 2000.0).unwrap_err().0,
            "Load diameter must be greater than 0."
        );
    }
}
