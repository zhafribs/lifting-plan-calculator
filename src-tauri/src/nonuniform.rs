//! Nonuniform lifts, ported from `Nonuniform.kt`.
//!
//! A *nonuniform* lift is one where the two sling legs do not share the load
//! equally. Three scenarios are covered: the 2-leg asymmetric bridle with a
//! shortening grab hook, and the two two-crane tandem methods.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::calc::{self, UsageState};
use crate::sling::{
    leg_capacity, max_by_first, published_af, published_length, rated_legs,
};

/// A nonuniform input that cannot be solved, in words an operator can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonuniformInputError(pub String);

impl fmt::Display for NonuniformInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for NonuniformInputError {}

fn err(message: &str) -> NonuniformInputError {
    NonuniformInputError(message.to_string())
}

/// A scenario's identity in the tab's list.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScenarioSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

/// Where crane 2's lifting lug sits, and what that changes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TandemMethod {
    pub key: &'static str,
    pub label: &'static str,
    pub full_label: &'static str,
    /// The second method's projected distance, in words, for the report.
    pub rule: &'static str,
    pub lug2: &'static str,
    pub span_behaviour: &'static str,
    pub max_rotation: &'static str,
    pub use_for: &'static str,
    /// Whether the method can be carried to the full 90 degrees of vertical
    /// rotation.
    pub full_rotation: bool,
}

/// The two tandem methods, in the order the tab offers them.
pub const TANDEM_METHODS: [TandemMethod; 2] = [
    TandemMethod {
        key: "tandem_aligned_pivot",
        label: "1. Aligned COG Pivot",
        full_label: "Tandem Lift (Aligned COG Pivot)",
        rule: "ND2 = D2 x cos(theta)",
        lug2: "Welded on the side or inside of the frame, inline with the \
               horizontal COG axis.",
        span_behaviour: "Stays wide and stable; the hooks do not have to \
                         migrate inward as the load rotates.",
        max_rotation: "Full 90 degrees vertical rotation.",
        use_for: "Up-ending and load rotation manoeuvres: precast concrete \
                  columns, vessels, storage tanks, wind tower sections.",
        full_rotation: true,
    },
    TandemMethod {
        key: "tandem_top_lug",
        label: "2. Repositioned Top Lifting Point",
        full_label: "Tandem Lift (Repositioned Top Lifting Point)",
        // The document's header writes D2 - Y*tan(theta)*cos(theta), but its own
        // working is (D2 - Y*tan(theta))*cos(theta). Only the second reproduces
        // its printed span, tensions and required WLL, so that is what is solved.
        rule: "ND2 = (D2 - Y x tan(theta)) x cos(theta)",
        lug2: "Welded on the top surface face of the cargo, sitting above the \
               COG level.",
        span_behaviour: "Compresses and narrows as the load tilts, which \
                         shortens crane 2's balancing lever arm quickly.",
        max_rotation: "Limited; the span reaches zero at the angle where \
                       tan(theta) = D2 / Y.",
        use_for: "Level-shifting and drift lifts where the load can never go \
                  vertical: skids, bridge trusses, pipe rack modules.",
        full_rotation: false,
    },
];

/// The tandem methods, in the order the tab lists them.
pub fn tandem_method_order() -> Vec<&'static str> {
    TANDEM_METHODS.iter().map(|method| method.key).collect()
}

/// Every scenario the model can solve, in tab order.
pub const SCENARIO_SPECS: [ScenarioSpec; 3] = [
    ScenarioSpec {
        key: "asym_2leg_shortening",
        label: "2-Leg Asymmetric (Shortening Grab Hook)",
        description: "Two legs sharing one headroom with the COG off the \
                      midpoint; the shorter leg is shortened with a clevis grab hook.",
    },
    ScenarioSpec {
        key: "tandem_aligned_pivot",
        label: "Tandem Lift (Aligned COG Pivot)",
        description: "Two independent cranes sharing the load. Lug 2 is inline with \
                      the COG, so the span holds while the load rotates and can be \
                      carried to the full 90 degrees of vertical rotation.",
    },
    ScenarioSpec {
        key: "tandem_top_lug",
        label: "Tandem Lift (Repositioned Top Lifting Point)",
        description: "Two independent cranes sharing the load. Lug 2 is on the top \
                      surface above the COG, so the span closes as the load tilts and \
                      the load runs onto crane 2 far faster.",
    },
];

/// Every scenario the model can solve, in tab order.
pub fn scenario_order() -> Vec<&'static str> {
    SCENARIO_SPECS.iter().map(|spec| spec.key).collect()
}

/// The tandem method a scenario key names.
pub fn tandem_method(scenario: &str) -> Result<&'static TandemMethod, NonuniformInputError> {
    TANDEM_METHODS
        .iter()
        .find(|method| method.key == scenario)
        .ok_or_else(|| {
            err(&format!(
                "'{scenario}' is not a tandem method: it is {}",
                tandem_method_order().join(", ")
            ))
        })
}

/// The leg counts a sling tag can be built with, and so the choices the form
/// offers. **1 is a single sling leg, not a one-leg bridle**, and it is first
/// because it is the ordinary case.
pub const SLING_LEG_OPTIONS: [i64; 4] = [1, 2, 3, 4];

/// The tag angle that applies to a tag of `legs` legs.
///
/// A single leg hangs vertical, so there is no angle at which to rate it: it
/// takes its tag's full rating. Forcing zero here rather than only hiding the
/// field in the UI is load-bearing — a 45 deg left over from before the
/// operator chose a single leg would quietly derate the capacity by a third.
pub fn effective_tag_angle(legs: i64, angle: f64) -> f64 {
    if legs == crate::sling::SINGLE_LEG {
        0.0
    } else {
        angle
    }
}

/// The scenario's identity, or a named failure.
pub fn scenario_spec(scenario: &str) -> Result<&'static ScenarioSpec, NonuniformInputError> {
    SCENARIO_SPECS
        .iter()
        .find(|spec| spec.key == scenario)
        .ok_or_else(|| err(&format!("Unknown nonuniform scenario: {scenario}")))
}

/// User-entered values for a nonuniform lift.
///
/// `pick_point2_height` is the step between the two pick points, and it is
/// signed: **positive means pick point 2 sits higher than pick point 1**,
/// negative means it sits lower. The sign is not cosmetic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NonuniformInputs {
    pub scenario: String,
    pub sling_length: f64,
    pub pick_distance: f64,
    pub cog_from_pick1: f64,
    pub pick_point2_height: f64,
    /// 1.0 rather than 0.0 like the rest: the reeve factor **divides** the
    /// required rating, so zero is not a value the solver accepts at all.
    pub base_reeve_factor: f64,
    pub tag_wll1: f64,
    pub tag_wll1_angle: f64,
    pub sling_legs1: i64,
    pub tag_wll2: f64,
    pub tag_wll2_angle: f64,
    pub sling_legs2: i64,
    // Tandem lift (two independent cranes).
    pub dist_a: f64,
    pub dist_b: f64,
    pub cog_height: f64,
    pub tilt_angle: f64,
    pub tag_wll_a: f64,
    pub tag_wll_a_angle: f64,
    pub sling_legs_a: i64,
    pub tag_wll_b: f64,
    pub tag_wll_b_angle: f64,
    pub sling_legs_b: i64,
    pub crane_rated_a: f64,
    pub crane_rated_b: f64,
}

impl Default for NonuniformInputs {
    fn default() -> Self {
        Self {
            scenario: "asym_2leg_shortening".to_string(),
            sling_length: 0.0,
            pick_distance: 0.0,
            cog_from_pick1: 0.0,
            pick_point2_height: 0.0,
            base_reeve_factor: 1.0,
            tag_wll1: 0.0,
            tag_wll1_angle: 45.0,
            sling_legs1: crate::sling::SINGLE_LEG,
            tag_wll2: 0.0,
            tag_wll2_angle: 45.0,
            sling_legs2: crate::sling::SINGLE_LEG,
            dist_a: 0.0,
            dist_b: 0.0,
            cog_height: 0.0,
            tilt_angle: 0.0,
            tag_wll_a: 0.0,
            tag_wll_a_angle: 45.0,
            sling_legs_a: crate::sling::SINGLE_LEG,
            tag_wll_b: 0.0,
            tag_wll_b_angle: 45.0,
            sling_legs_b: crate::sling::SINGLE_LEG,
            crane_rated_a: 0.0,
            crane_rated_b: 0.0,
        }
    }
}

/// The documents' inputs for a scenario.
pub fn example_nonuniform_inputs(
    scenario: &str,
) -> Result<NonuniformInputs, NonuniformInputError> {
    match scenario {
        // The document's worked case: 2 t over a 5.0 m span with the COG
        // 3.5 m from pick point 1, and pick point 2 sitting 0.5 m higher.
        "asym_2leg_shortening" => Ok(NonuniformInputs {
            scenario: "asym_2leg_shortening".to_string(),
            sling_length: 7.0,
            pick_distance: 5.0,
            cog_from_pick1: 3.5,
            pick_point2_height: 0.5,
            tilt_angle: 35.0,
            ..NonuniformInputs::default()
        }),
        // Both tandem documents work the same case: a 60 t load, the COG 3 m
        // from crane 1 and 5 m from crane 2, 2 m below the lifting point
        // baseline, at a 35 degree tilt.
        "tandem_aligned_pivot" | "tandem_top_lug" => Ok(NonuniformInputs {
            scenario: scenario.to_string(),
            sling_length: 7.0,
            pick_distance: 5.0,
            cog_from_pick1: 3.5,
            dist_a: 3.0,
            dist_b: 5.0,
            cog_height: 2.0,
            tilt_angle: 35.0,
            ..NonuniformInputs::default()
        }),
        _ => Err(err(&format!("No example for scenario: {scenario}"))),
    }
}

/// A number, or a named failure an operator can act on.
fn number(value: f64, name: &str, positive: bool) -> Result<f64, NonuniformInputError> {
    if !value.is_finite() {
        return Err(err(&format!("{name} must be a finite number.")));
    }
    if positive && value <= 0.0 {
        return Err(err(&format!("{name} must be greater than zero.")));
    }
    Ok(value)
}

/// The tag-rating utilisation as a percentage.
///
/// This is **not** `sling::tag_utilisation`. The two differ on purpose: the
/// uniform engine divides the *printed* figures, while this module publishes
/// the exact quotient. Using one for the other would move the last digit of
/// every utilisation here.
fn tag_utilisation_exact(required_wll: f64, capacity: Option<f64>) -> Option<f64> {
    let capacity = capacity?;
    if capacity == 0.0 {
        return None;
    }
    Some(required_wll / capacity * 100.0)
}

/// What every scenario reports, so the shared rows have one home.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NonuniformResult {
    Asym(Asym2LegResult),
    Tandem(TandemResult),
}

impl NonuniformResult {
    pub fn scenario(&self) -> &str {
        match self {
            NonuniformResult::Asym(result) => &result.scenario,
            NonuniformResult::Tandem(result) => &result.scenario,
        }
    }

    pub fn scenario_label(&self) -> &str {
        match self {
            NonuniformResult::Asym(result) => &result.scenario_label,
            NonuniformResult::Tandem(result) => &result.scenario_label,
        }
    }

    pub fn load_weight(&self) -> f64 {
        match self {
            NonuniformResult::Asym(result) => result.load_weight,
            NonuniformResult::Tandem(result) => result.load_weight,
        }
    }

    pub fn equipment_ok(&self) -> Option<bool> {
        match self {
            NonuniformResult::Asym(result) => result.equipment_ok,
            NonuniformResult::Tandem(result) => result.equipment_ok,
        }
    }

    pub fn setup_swl(&self) -> f64 {
        match self {
            NonuniformResult::Asym(result) => result.setup_swl,
            NonuniformResult::Tandem(result) => result.setup_swl,
        }
    }
}

/// The 2-leg asymmetric bridle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asym2LegResult {
    pub scenario: String,
    pub scenario_label: String,
    pub load_weight: f64,
    pub base_reeve_factor: f64,
    pub sling_length: f64,
    pub pick_distance: f64,
    pub cog_from_pick1: f64,
    pub pick_point2_height: f64,
    pub span1: f64,
    pub span2: f64,
    pub long_leg: i64,
    pub short_leg: i64,
    pub headroom: f64,
    pub drop1: f64,
    pub drop2: f64,
    pub short_leg_drop: f64,
    pub short_leg_length: f64,
    pub shortening: f64,
    pub shortening_mm: f64,
    pub needs_lengthening: bool,
    pub lengthening: f64,
    pub length1: f64,
    pub length2: f64,
    pub angle1: f64,
    pub angle2: f64,
    pub angle_factor1: f64,
    pub angle_factor2: f64,
    pub share1: f64,
    pub share2: f64,
    pub tension1: f64,
    pub tension2: f64,
    pub required_wll1: f64,
    pub required_wll2: f64,
    pub max_tension: f64,
    pub required_wll: f64,
    pub governing_leg: i64,
    pub tag_wll1: f64,
    pub tag_wll1_angle: f64,
    pub tag_angle_factor1: f64,
    pub sling_legs1: i64,
    pub rated_legs1: i64,
    pub leg_capacity1: Option<f64>,
    pub utilisation1: Option<f64>,
    pub tag_wll2: f64,
    pub tag_wll2_angle: f64,
    pub tag_angle_factor2: f64,
    pub sling_legs2: i64,
    pub rated_legs2: i64,
    pub leg_capacity2: Option<f64>,
    pub utilisation2: Option<f64>,
    pub equipment_ok: Option<bool>,
    pub setup_swl: f64,
}

/// The two-crane tandem lift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TandemResult {
    pub scenario: String,
    pub scenario_label: String,
    pub load_weight: f64,
    pub base_reeve_factor: f64,
    pub method_key: String,
    pub method_label: String,
    pub method_full_label: String,
    pub method_rule: String,
    pub method_lug2: String,
    pub method_span_behaviour: String,
    pub method_max_rotation: String,
    pub method_use_for: String,
    pub full_rotation: bool,
    pub dist_a: f64,
    pub dist_b: f64,
    pub span: f64,
    pub cog_height: f64,
    pub tilt_angle: f64,
    pub angle: f64,
    pub angle_factor: f64,
    pub dist_a_new: f64,
    pub dist_b_new: f64,
    pub span_new: f64,
    pub cog_shift: f64,
    pub lug2_travel: f64,
    pub height_difference: f64,
    pub max_tilt_angle: f64,
    pub share_a_level: f64,
    pub share_b_level: f64,
    pub tension_a_level: f64,
    pub tension_b_level: f64,
    pub required_a_level: f64,
    pub required_b_level: f64,
    pub share_a_tilt: f64,
    pub share_b_tilt: f64,
    pub tension_a_tilt: f64,
    pub tension_b_tilt: f64,
    pub required_a_tilt: f64,
    pub required_b_tilt: f64,
    pub share_a_vertical: Option<f64>,
    pub share_b_vertical: Option<f64>,
    pub tension_a_vertical: Option<f64>,
    pub tension_b_vertical: Option<f64>,
    pub required_a_vertical: Option<f64>,
    pub required_b_vertical: Option<f64>,
    pub required_wll_a: f64,
    pub required_wll_b: f64,
    pub governing_case_a: String,
    pub governing_case_b: String,
    pub max_tension: f64,
    pub required_wll: f64,
    pub tag_wll_a: f64,
    pub tag_wll_a_angle: f64,
    pub tag_angle_factor_a: f64,
    pub sling_legs_a: i64,
    pub rated_legs_a: i64,
    pub leg_capacity_a: Option<f64>,
    pub utilisation_a: Option<f64>,
    pub tag_wll_b: f64,
    pub tag_wll_b_angle: f64,
    pub tag_angle_factor_b: f64,
    pub sling_legs_b: i64,
    pub rated_legs_b: i64,
    pub leg_capacity_b: Option<f64>,
    pub utilisation_b: Option<f64>,
    pub crane_rated_a: f64,
    pub crane_rated_b: f64,
    pub crane_usage_a: Option<f64>,
    pub crane_usage_b: Option<f64>,
    pub crane_usage_state_a: Option<UsageState>,
    pub crane_usage_state_b: Option<UsageState>,
    pub equipment_ok: Option<bool>,
    /// True only when every entered crane sits outside the 75% window.
    pub crane_risk_flag: Option<bool>,
    pub setup_swl: f64,
}

/// The 2-leg asymmetric bridle with a clevis shortening grab hook.
pub fn solve_asym_2leg(
    inputs: &NonuniformInputs,
    load_weight: f64,
) -> Result<Asym2LegResult, NonuniformInputError> {
    let sling_length = number(inputs.sling_length, "Sling length", true)?;
    let span = number(inputs.pick_distance, "Distance between pick points", true)?;
    let cog = number(inputs.cog_from_pick1, "COG position from pick point 1", false)?;
    let reeve = number(inputs.base_reeve_factor, "Base reeve factor", true)?;
    let load = number(load_weight, "Load weight", false)?;

    if load < 0.0 {
        return Err(err("Load weight cannot be negative."));
    }
    if !(cog > 0.0 && cog < span) {
        return Err(err(&format!(
            "The COG must sit between the two pick points: enter a position \
             greater than 0 m and less than the {} m span.",
            thousands(span, 3)
        )));
    }

    // Statics: the load share follows the moment about the opposite pick point,
    // so the leg on the short side of the COG carries the larger share.
    let x1 = cog;
    let x2 = span - cog;
    let share1 = load * x2 / span;
    let share2 = load * x1 / span;

    // Geometry: the longer horizontal run keeps the full standard length, and
    // that leg alone sets the hook's height.
    let long_is1 = x1 >= x2;
    let x_long = x1.max(x2);
    let x_short = x1.min(x2);
    if sling_length <= x_long {
        return Err(err(&format!(
            "The {} m sling is too short to reach {} m across. Use a longer \
             sling or a shorter span.",
            thousands(sling_length, 3),
            thousands(x_long, 3)
        )));
    }

    let step = number(
        inputs.pick_point2_height,
        "Height difference between pick points",
        false,
    )?;
    let drop_long = published_length((sling_length * sling_length - x_long * x_long).sqrt());
    let (drop1, drop2);
    if long_is1 {
        // Pick point 1 sets the drop; pick point 2 is `step` above it.
        drop1 = drop_long;
        drop2 = published_length(drop_long - step);
    } else {
        // Pick point 2 sets the drop; pick point 1 is `step` below it.
        drop2 = drop_long;
        drop1 = published_length(drop_long + step);
    }
    if drop1 <= 0.0 || drop2 <= 0.0 {
        let raised = if step > 0.0 { "2" } else { "1" };
        return Err(err(&format!(
            "A {} m step leaves pick point {raised} at or above hook level, so \
             the sling would have to run uphill. Reduce the step or use a \
             longer sling.",
            thousands(step.abs(), 3)
        )));
    }
    let drop_short = if long_is1 { drop2 } else { drop1 };
    let short_length = published_length((drop_short * drop_short + x_short * x_short).sqrt());
    let shortening = published_length(sling_length - short_length);

    let length1 = if long_is1 { sling_length } else { short_length };
    let length2 = if long_is1 { short_length } else { sling_length };

    let angle1 = (x1 / length1).asin().to_degrees();
    let angle2 = (x2 / length2).asin().to_degrees();
    let af1 = published_af(angle1.to_radians().cos());
    let af2 = published_af(angle2.to_radians().cos());

    // The document publishes the line tension from the angle factor alone and
    // the tag rating from the reeve factor as well. They coincide at the
    // published RF of 1.00 and are kept separate so a reeved bridle stays
    // honest about which figure is the leg pull and which is the rating.
    let tension1 = share1 / af1;
    let tension2 = share2 / af2;
    let required1 = share1 / (reeve * af1);
    let required2 = share2 / (reeve * af2);

    let tag_wll1 = number(inputs.tag_wll1, "Sling 1 tag WLL", false)?;
    let tag_wll2 = number(inputs.tag_wll2, "Sling 2 tag WLL", false)?;
    let chosen_legs1 = number(inputs.sling_legs1 as f64, "Sling 1 sling legs", false)? as i64;
    let chosen_legs2 = number(inputs.sling_legs2 as f64, "Sling 2 sling legs", false)? as i64;
    let rated_legs1 = rated_legs(chosen_legs1);
    let rated_legs2 = rated_legs(chosen_legs2);
    // Resolved from the chosen count, not from the field the UI has just hidden.
    let tag_angle1 = effective_tag_angle(
        chosen_legs1,
        number(inputs.tag_wll1_angle, "Sling 1 tag WLL angle", false)?,
    );
    let tag_angle2 = effective_tag_angle(
        chosen_legs2,
        number(inputs.tag_wll2_angle, "Sling 2 tag WLL angle", false)?,
    );
    let tag_af1 = published_af(tag_angle1.to_radians().cos());
    let tag_af2 = published_af(tag_angle2.to_radians().cos());
    let capacity1 = leg_capacity(tag_wll1, rated_legs1, tag_af1);
    let capacity2 = leg_capacity(tag_wll2, rated_legs2, tag_af2);
    let utilisation1 = tag_utilisation_exact(required1, capacity1);
    let utilisation2 = tag_utilisation_exact(required2, capacity2);

    let governing_leg = if required1 >= required2 { 1 } else { 2 };
    // A clevis shortening grab hook can only take length *out* of a leg.
    let needs_lengthening = shortening < 0.0;

    let equipment_ok = match (utilisation1, utilisation2) {
        (Some(first), Some(second)) => Some(first <= 100.0 && second <= 100.0),
        _ => None,
    };

    Ok(Asym2LegResult {
        scenario: inputs.scenario.clone(),
        scenario_label: scenario_spec(&inputs.scenario)?.label.to_string(),
        load_weight: load,
        sling_length,
        pick_distance: span,
        cog_from_pick1: cog,
        pick_point2_height: step,
        span1: x1,
        span2: x2,
        long_leg: if long_is1 { 1 } else { 2 },
        short_leg: if long_is1 { 2 } else { 1 },
        headroom: drop_long,
        drop1,
        drop2,
        short_leg_drop: drop_short,
        short_leg_length: short_length,
        shortening,
        shortening_mm: shortening * 1000.0,
        needs_lengthening,
        lengthening: if needs_lengthening { shortening.abs() } else { 0.0 },
        length1,
        length2,
        angle1,
        angle2,
        angle_factor1: af1,
        angle_factor2: af2,
        base_reeve_factor: reeve,
        share1,
        share2,
        tension1,
        tension2,
        required_wll1: required1,
        required_wll2: required2,
        max_tension: tension1.max(tension2),
        required_wll: required1.max(required2),
        governing_leg,
        tag_wll1,
        // The angle that applied, not the one that was typed.
        tag_wll1_angle: tag_angle1,
        tag_angle_factor1: tag_af1,
        sling_legs1: chosen_legs1,
        rated_legs1,
        leg_capacity1: capacity1,
        utilisation1,
        tag_wll2,
        tag_wll2_angle: tag_angle2,
        tag_angle_factor2: tag_af2,
        sling_legs2: chosen_legs2,
        rated_legs2,
        leg_capacity2: capacity2,
        utilisation2,
        equipment_ok,
        // The document's setup SWL is the whole load.
        setup_swl: load,
    })
}

/// Project each hook's distance to the COG after the tilt has been applied.
///
/// Both documents publish these three decimals, and both take their shares
/// from the published figures rather than from the exact ones, so they are
/// rounded here for the same reason.
pub fn tandem_projected_distances(
    method: &TandemMethod,
    dist_a: f64,
    dist_b: f64,
    cog_height: f64,
    tilt: f64,
) -> (f64, f64) {
    let radians = tilt.to_radians();
    let cos_t = radians.cos();
    let sin_t = radians.sin();
    let dist_a_new = published_length(dist_a * cos_t + cog_height * sin_t);
    let dist_b_new = if method.full_rotation {
        published_length(dist_b * cos_t)
    } else {
        published_length((dist_b - cog_height * radians.tan()) * cos_t)
    };
    (dist_a_new, dist_b_new)
}

/// Solve the two-crane tandem lift at the level, tilted and vertical cases.
#[allow(clippy::too_many_lines)]
pub fn solve_tandem_2_crane(
    inputs: &NonuniformInputs,
    load_weight: f64,
) -> Result<TandemResult, NonuniformInputError> {
    let method = tandem_method(&inputs.scenario)?;
    let dist_a = number(inputs.dist_a, "Distance from crane 1 hook to COG", true)?;
    let dist_b = number(inputs.dist_b, "Distance from crane 2 hook to COG", true)?;
    let cog_height = number(inputs.cog_height, "COG depth below the baseline", false)?;
    let tilt = number(inputs.tilt_angle, "Cargo tilt angle", false)?;
    let reeve = number(inputs.base_reeve_factor, "Base reeve factor", true)?;
    let load = number(load_weight, "Load weight", false)?;

    if load < 0.0 {
        return Err(err("Load weight cannot be negative."));
    }
    if cog_height < 0.0 {
        return Err(err(
            "The COG depth cannot be negative: measure it downward from the line \
             joining the two lifting points.",
        ));
    }
    if !(0.0..=90.0).contains(&tilt) {
        return Err(err(&format!(
            "The cargo tilt angle must be between 0 and 90 degrees; {} was entered.",
            thousands(tilt, 2)
        )));
    }

    // The span between the two hooks is the sum of the two COG distances.
    let span = published_length(dist_a + dist_b);

    // The angle beyond which the lift stops being a lift.
    let max_tilt = if method.full_rotation || cog_height == 0.0 {
        90.0
    } else {
        (dist_b / cog_height).atan().to_degrees()
    };

    // Case A: the level baseline.
    let angle: f64 = 0.0;
    let af = published_af(angle.to_radians().cos());
    let share_a_level = load * dist_b / span;
    let share_b_level = load * dist_a / span;

    // Case B: the declared tilt.
    let (dist_a_new, dist_b_new) =
        tandem_projected_distances(method, dist_a, dist_b, cog_height, tilt);
    if dist_b_new < 0.0 || (dist_b_new == 0.0 && !method.full_rotation) {
        return Err(err(&format!(
            "At a {} degree tilt with the COG {} m below the baseline, crane 2's \
             projected distance closes to nothing (the span runs out at \
             {} degrees). The load would tip about crane 2 instead of hanging \
             from both hooks; reduce the tilt angle or the COG depth.",
            thousands(tilt, 2),
            thousands(cog_height, 3),
            fixed(max_tilt, 1)
        )));
    }
    let span_new = published_length(dist_a_new + dist_b_new);
    let (share_a_tilt, share_b_tilt);
    if span_new > 0.0 {
        share_a_tilt = load * dist_b_new / span_new;
        share_b_tilt = load * dist_a_new / span_new;
    } else {
        share_a_tilt = 0.0;
        share_b_tilt = load;
    }

    // Case C: the full 90 degrees of vertical rotation, which only the aligned
    // pivot method can be taken through.
    let (share_a_vertical, share_b_vertical): (Option<f64>, Option<f64>);
    let cases: Vec<(&str, f64, f64)>;
    if method.full_rotation {
        share_a_vertical = Some(0.0);
        share_b_vertical = Some(load);
        cases = vec![
            ("level", share_a_level, share_b_level),
            ("tilt", share_a_tilt, share_b_tilt),
            ("vertical", 0.0, load),
        ];
    } else {
        share_a_vertical = None;
        share_b_vertical = None;
        cases = vec![
            ("level", share_a_level, share_b_level),
            ("tilt", share_a_tilt, share_b_tilt),
        ];
    }

    // Each crane is governed by whichever of its own cases demands most. The
    // *first* maximum is taken, as the source's `maxByOrNull` does.
    let governing = |index: usize| -> (&'static str, f64) {
        let key = |case: &(&'static str, f64, f64)| if index == 0 { case.1 } else { case.2 };
        let best = max_by_first(&cases, key).expect("cases is never empty");
        (best.0, key(best))
    };
    let (governing_a, share_a) = governing(0);
    let (governing_b, share_b) = governing(1);

    let case_figures = |share: f64| -> (f64, f64) { (share / af, share / (reeve * af)) };

    let (tension_a_level, required_a_level) = case_figures(share_a_level);
    let (tension_b_level, required_b_level) = case_figures(share_b_level);
    let (tension_a_tilt, required_a_tilt) = case_figures(share_a_tilt);
    let (tension_b_tilt, required_b_tilt) = case_figures(share_b_tilt);
    let required_a = case_figures(share_a).1;
    let required_b = case_figures(share_b).1;

    let mut tension_a_vertical: Option<f64> = None;
    let mut required_a_vertical: Option<f64> = None;
    let mut tension_b_vertical: Option<f64> = None;
    let mut required_b_vertical: Option<f64> = None;
    if let Some(vertical_b) = share_b_vertical {
        let a = case_figures(share_a_vertical.expect("vertical shares come in pairs"));
        let b = case_figures(vertical_b);
        tension_a_vertical = Some(a.0);
        required_a_vertical = Some(a.1);
        tension_b_vertical = Some(b.0);
        required_b_vertical = Some(b.1);
    }

    let tag_wll_a = number(inputs.tag_wll_a, "Crane 1 sling tag WLL", false)?;
    let tag_wll_b = number(inputs.tag_wll_b, "Crane 2 sling tag WLL", false)?;
    let chosen_legs_a = number(inputs.sling_legs_a as f64, "Crane 1 sling legs", false)? as i64;
    let chosen_legs_b = number(inputs.sling_legs_b as f64, "Crane 2 sling legs", false)? as i64;
    let rated_legs_a = rated_legs(chosen_legs_a);
    let rated_legs_b = rated_legs(chosen_legs_b);
    let tag_angle_a = effective_tag_angle(
        chosen_legs_a,
        number(inputs.tag_wll_a_angle, "Crane 1 sling tag WLL angle", false)?,
    );
    let tag_angle_b = effective_tag_angle(
        chosen_legs_b,
        number(inputs.tag_wll_b_angle, "Crane 2 sling tag WLL angle", false)?,
    );
    let tag_af_a = published_af(tag_angle_a.to_radians().cos());
    let tag_af_b = published_af(tag_angle_b.to_radians().cos());
    let capacity_a = leg_capacity(tag_wll_a, rated_legs_a, tag_af_a);
    let capacity_b = leg_capacity(tag_wll_b, rated_legs_b, tag_af_b);
    let utilisation_a = tag_utilisation_exact(required_a, capacity_a);
    let utilisation_b = tag_utilisation_exact(required_b, capacity_b);

    // Each crane is booked against its own rating.
    let crane_rated_a = number(inputs.crane_rated_a, "Crane 1 rated capacity", false)?;
    let crane_rated_b = number(inputs.crane_rated_b, "Crane 2 rated capacity", false)?;
    let crane_usage_a = if crane_rated_a > 0.0 {
        Some(calc::chart_usage_percent(required_a, crane_rated_a).unwrap_or(0.0))
    } else {
        None
    };
    let crane_usage_b = if crane_rated_b > 0.0 {
        Some(calc::chart_usage_percent(required_b, crane_rated_b).unwrap_or(0.0))
    } else {
        None
    };

    let radians = tilt.to_radians();
    let cog_shift = published_length(cog_height * radians.sin());
    let lug2_travel = if !method.full_rotation {
        published_length(cog_height * radians.tan())
    } else {
        0.0
    };

    let mut all_tensions = vec![
        tension_a_level,
        tension_b_level,
        tension_a_tilt,
        tension_b_tilt,
    ];
    if let Some(vertical) = tension_a_vertical {
        all_tensions.push(vertical);
        all_tensions.push(tension_b_vertical.unwrap_or(0.0));
    }
    let max_tension = all_tensions.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let equipment_ok = match (utilisation_a, utilisation_b) {
        (Some(first), Some(second)) => Some(first <= 100.0 && second <= 100.0),
        _ => None,
    };

    let crane_risk_flag = match (crane_usage_a, crane_usage_b) {
        (Some(first), Some(second)) => {
            Some(!matches!(calc::crane_usage_band(first), UsageState::Within)
                || !matches!(calc::crane_usage_band(second), UsageState::Within))
        }
        _ => None,
    };

    Ok(TandemResult {
        scenario: inputs.scenario.clone(),
        scenario_label: scenario_spec(&inputs.scenario)?.label.to_string(),
        load_weight: load,
        method_key: method.key.to_string(),
        method_label: method.label.to_string(),
        method_full_label: method.full_label.to_string(),
        method_rule: method.rule.to_string(),
        method_lug2: method.lug2.to_string(),
        method_span_behaviour: method.span_behaviour.to_string(),
        method_max_rotation: method.max_rotation.to_string(),
        method_use_for: method.use_for.to_string(),
        full_rotation: method.full_rotation,
        dist_a,
        dist_b,
        span,
        cog_height,
        tilt_angle: tilt,
        angle,
        angle_factor: af,
        base_reeve_factor: reeve,
        dist_a_new,
        dist_b_new,
        span_new,
        cog_shift,
        lug2_travel,
        height_difference: published_length(span * radians.sin()),
        max_tilt_angle: max_tilt,
        share_a_level,
        share_b_level,
        tension_a_level,
        tension_b_level,
        required_a_level,
        required_b_level,
        share_a_tilt,
        share_b_tilt,
        tension_a_tilt,
        tension_b_tilt,
        required_a_tilt,
        required_b_tilt,
        share_a_vertical,
        share_b_vertical,
        tension_a_vertical,
        tension_b_vertical,
        required_a_vertical,
        required_b_vertical,
        required_wll_a: required_a,
        required_wll_b: required_b,
        governing_case_a: governing_a.to_string(),
        governing_case_b: governing_b.to_string(),
        max_tension,
        required_wll: required_a.max(required_b),
        tag_wll_a,
        tag_wll_a_angle: tag_angle_a,
        tag_angle_factor_a: tag_af_a,
        sling_legs_a: chosen_legs_a,
        rated_legs_a,
        leg_capacity_a: capacity_a,
        utilisation_a,
        tag_wll_b,
        tag_wll_b_angle: tag_angle_b,
        tag_angle_factor_b: tag_af_b,
        sling_legs_b: chosen_legs_b,
        rated_legs_b,
        leg_capacity_b: capacity_b,
        utilisation_b,
        crane_rated_a,
        crane_rated_b,
        crane_usage_a,
        crane_usage_b,
        // A state rather than a string, because "within 75%", "between 75% and
        // the chart" and "over the chart" are three different things.
        crane_usage_state_a: crane_usage_a.map(calc::crane_usage_band),
        crane_usage_state_b: crane_usage_b.map(calc::crane_usage_band),
        equipment_ok,
        crane_risk_flag,
        setup_swl: load,
    })
}

/// The case that governs one crane, in words rather than as a key.
pub fn governing_case_words(case_key: Option<&str>) -> String {
    match case_key {
        Some("level") => "level baseline".to_string(),
        Some("tilt") => "declared tilt".to_string(),
        Some("vertical") => "full 90\u{00b0} vertical rotation".to_string(),
        _ => "\u{2014}".to_string(),
    }
}

/// A figure with a thousands separator and a fixed number of decimals.
fn thousands(value: f64, decimals: usize) -> String {
    crate::figures::grouped(value, decimals)
}

/// A figure at a fixed number of decimals, no grouping.
fn fixed(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// Solve one nonuniform scenario.
pub fn solve_nonuniform(
    inputs: &NonuniformInputs,
    load_weight: f64,
) -> Result<NonuniformResult, NonuniformInputError> {
    match inputs.scenario.as_str() {
        "asym_2leg_shortening" => {
            Ok(NonuniformResult::Asym(solve_asym_2leg(inputs, load_weight)?))
        }
        "tandem_aligned_pivot" | "tandem_top_lug" => Ok(NonuniformResult::Tandem(
            solve_tandem_2_crane(inputs, load_weight)?,
        )),
        other => Err(err(&format!("Unknown nonuniform scenario: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-6;

    fn asym(load: f64) -> Asym2LegResult {
        match solve_nonuniform(&example_nonuniform_inputs("asym_2leg_shortening").unwrap(), load)
            .unwrap()
        {
            NonuniformResult::Asym(result) => result,
            _ => panic!("expected the asymmetric bridle"),
        }
    }

    fn tandem(scenario: &str, load: f64) -> TandemResult {
        match solve_nonuniform(&example_nonuniform_inputs(scenario).unwrap(), load).unwrap() {
            NonuniformResult::Tandem(result) => result,
            _ => panic!("expected a tandem lift"),
        }
    }

    #[test]
    fn the_asymmetric_bridle_matches_the_reference() {
        let result = asym(2000.0);
        assert!((result.span1 - 3.5).abs() < TOL);
        assert!((result.span2 - 1.5).abs() < TOL);
        assert_eq!(result.long_leg, 1);
        assert_eq!(result.short_leg, 2);
        assert!((result.share1 - 600.0).abs() < TOL);
        assert!((result.share2 - 1400.0).abs() < TOL);
        assert!((result.headroom - 6.062).abs() < TOL);
        assert!((result.drop1 - 6.062).abs() < TOL);
        assert!((result.drop2 - 5.562).abs() < TOL);
        assert!((result.short_leg_length - 5.761).abs() < TOL);
        assert!((result.shortening - 1.239).abs() < TOL);
        assert!((result.shortening_mm - 1239.0).abs() < TOL);
        assert!(!result.needs_lengthening);
        assert!((result.lengthening - 0.0).abs() < TOL);
        assert!((result.length1 - 7.0).abs() < TOL);
        assert!((result.length2 - 5.761).abs() < TOL);
        assert!((result.angle1 - 30.000000000000004).abs() < TOL);
        assert!((result.angle2 - 15.09210459686422).abs() < TOL);
        assert!((result.angle_factor1 - 0.866).abs() < TOL);
        assert!((result.angle_factor2 - 0.966).abs() < TOL);
        assert!((result.tension1 - 692.8406466512702).abs() < TOL);
        assert!((result.tension2 - 1449.2753623188407).abs() < TOL);
        assert!((result.required_wll2 - 1449.2753623188407).abs() < TOL);
        assert!((result.max_tension - 1449.2753623188407).abs() < TOL);
        assert!((result.required_wll - 1449.2753623188407).abs() < TOL);
        assert_eq!(result.governing_leg, 2);
        assert!((result.setup_swl - 2000.0).abs() < TOL);
    }

    #[test]
    fn a_four_leg_tag_is_rated_on_three_but_a_one_leg_tag_on_one() {
        let inputs = NonuniformInputs {
            scenario: "asym_2leg_shortening".to_string(),
            sling_length: 7.0,
            pick_distance: 5.0,
            cog_from_pick1: 3.5,
            pick_point2_height: 0.5,
            tag_wll1: 5000.0,
            sling_legs1: 2,
            tag_wll1_angle: 45.0,
            tag_wll2: 6000.0,
            sling_legs2: 4,
            tag_wll2_angle: 30.0,
            ..NonuniformInputs::default()
        };
        let result = solve_asym_2leg(&inputs, 2000.0).unwrap();
        assert_eq!(result.sling_legs1, 2);
        assert_eq!(result.rated_legs1, 2);
        assert!((result.tag_angle_factor1 - 0.707).abs() < TOL);
        assert!((result.leg_capacity1.unwrap() - 3536.0678925035363).abs() < TOL);
        assert!((result.utilisation1.unwrap() - 19.59353348729792).abs() < TOL);
        assert_eq!(result.sling_legs2, 4);
        assert_eq!(result.rated_legs2, 3);
        assert!((result.tag_angle_factor2 - 0.866).abs() < TOL);
        assert!((result.leg_capacity2.unwrap() - 2309.4688221709007).abs() < TOL);
        assert!((result.utilisation2.unwrap() - 62.753623188405804).abs() < TOL);
        assert_eq!(result.equipment_ok, Some(true));
    }

    #[test]
    fn a_single_leg_tag_ignores_its_leftover_angle() {
        let inputs = NonuniformInputs {
            scenario: "asym_2leg_shortening".to_string(),
            sling_length: 7.0,
            pick_distance: 5.0,
            cog_from_pick1: 3.5,
            pick_point2_height: 0.5,
            tag_wll1: 5000.0,
            sling_legs1: 1,
            tag_wll1_angle: 45.0,
            ..NonuniformInputs::default()
        };
        let result = solve_asym_2leg(&inputs, 2000.0).unwrap();
        assert!((result.tag_wll1_angle - 0.0).abs() < TOL);
        assert!((result.tag_angle_factor1 - 1.0).abs() < TOL);
        assert_eq!(result.rated_legs1, 1);
        assert!((result.leg_capacity1.unwrap() - 5000.0).abs() < TOL);
        assert!((result.utilisation1.unwrap() - 13.856812933025402).abs() < TOL);
        assert!((effective_tag_angle(1, 45.0) - 0.0).abs() < TOL);
        assert!((effective_tag_angle(2, 45.0) - 45.0).abs() < TOL);
        assert!((effective_tag_angle(4, 45.0) - 45.0).abs() < TOL);
    }

    #[test]
    fn the_aligned_pivot_tandem_matches_the_reference() {
        let result = tandem("tandem_aligned_pivot", 60000.0);
        assert!((result.span - 8.0).abs() < TOL);
        assert!((result.dist_a_new - 3.605).abs() < TOL);
        assert!((result.dist_b_new - 4.096).abs() < TOL);
        assert!((result.span_new - 7.701).abs() < TOL);
        assert!((result.cog_shift - 1.147).abs() < TOL);
        assert!((result.height_difference - 4.589).abs() < TOL);
        assert!((result.lug2_travel - 0.0).abs() < TOL);
        assert!((result.max_tilt_angle - 90.0).abs() < TOL);
        assert!((result.angle_factor - 1.0).abs() < TOL);
        assert!((result.share_a_level - 37500.0).abs() < TOL);
        assert!((result.share_b_level - 22500.0).abs() < TOL);
        assert!((result.share_a_tilt - 31912.738605375926).abs() < TOL);
        assert!((result.share_b_tilt - 28087.261394624074).abs() < TOL);
        assert!((result.share_a_vertical.unwrap() - 0.0).abs() < TOL);
        assert!((result.share_b_vertical.unwrap() - 60000.0).abs() < TOL);
        assert!((result.required_wll_a - 37500.0).abs() < TOL);
        assert_eq!(result.governing_case_a, "level");
        assert!((result.required_wll_b - 60000.0).abs() < TOL);
        assert_eq!(result.governing_case_b, "vertical");
        assert!((result.max_tension - 60000.0).abs() < TOL);
        assert!((result.required_wll - 60000.0).abs() < TOL);
    }

    #[test]
    fn the_top_lug_tandem_matches_the_reference() {
        let result = tandem("tandem_top_lug", 60000.0);
        assert!((result.span - 8.0).abs() < TOL);
        assert!((result.dist_a_new - 3.605).abs() < TOL);
        assert!((result.lug2_travel - 1.4).abs() < TOL);
        assert!((result.dist_b_new - 2.949).abs() < TOL);
        assert!((result.span_new - 6.554).abs() < TOL);
        assert!((result.cog_shift - 1.147).abs() < TOL);
        assert!((result.max_tilt_angle - 68.19859051364818).abs() < TOL);
        assert!((result.share_a_tilt - 26997.25358559658).abs() < TOL);
        assert!((result.share_b_tilt - 33002.74641440342).abs() < TOL);
        assert!((result.required_wll_a - 37500.0).abs() < TOL);
        assert_eq!(result.governing_case_a, "level");
        assert!((result.required_wll_b - 33002.74641440342).abs() < TOL);
        assert_eq!(result.governing_case_b, "tilt");
        assert!((result.required_wll - 37500.0).abs() < TOL);
    }

    #[test]
    fn the_two_tandem_methods_differ_only_in_crane_two() {
        let aligned = tandem("tandem_aligned_pivot", 60000.0);
        let top = tandem("tandem_top_lug", 60000.0);
        assert!((aligned.dist_a_new - top.dist_a_new).abs() < TOL);
        assert!((aligned.cog_shift - top.cog_shift).abs() < TOL);
        assert!((aligned.share_a_level - top.share_a_level).abs() < TOL);
        assert!((aligned.required_wll_a - top.required_wll_a).abs() < TOL);
        assert!((aligned.lug2_travel - 0.0).abs() < TOL);
        assert!((top.lug2_travel - 1.4).abs() < TOL);
        assert!((aligned.required_wll_b - 60000.0).abs() < TOL);
        assert!((top.required_wll_b - 33002.74641440342).abs() < TOL);
    }

    #[test]
    fn the_vertical_case_only_exists_for_the_method_that_can_reach_it() {
        let aligned = tandem("tandem_aligned_pivot", 60000.0);
        let top = tandem("tandem_top_lug", 60000.0);
        assert_eq!(aligned.share_a_vertical, Some(0.0));
        assert_eq!(aligned.share_b_vertical, Some(60000.0));
        assert_eq!(top.share_a_vertical, None);
        assert_eq!(top.share_b_vertical, None);
        assert_eq!(top.required_a_vertical, None);
        assert_eq!(top.required_b_vertical, None);
        assert!(aligned.full_rotation);
        assert!(!top.full_rotation);
    }

    #[test]
    fn the_aligned_pivot_can_be_stood_fully_upright() {
        let inputs = NonuniformInputs {
            scenario: "tandem_aligned_pivot".to_string(),
            dist_a: 3.0,
            dist_b: 5.0,
            cog_height: 2.0,
            tilt_angle: 90.0,
            ..NonuniformInputs::default()
        };
        let result = solve_tandem_2_crane(&inputs, 60000.0).unwrap();
        assert!((result.dist_b_new - 0.0).abs() < TOL);
        assert!((result.span_new - 2.0).abs() < TOL);
        assert!((result.share_a_tilt - 0.0).abs() < TOL);
        assert!((result.share_b_tilt - 60000.0).abs() < TOL);
        assert!((result.required_wll_b - 60000.0).abs() < TOL);
    }

    #[test]
    fn with_no_tilt_the_two_cranes_split_by_their_lever_arms() {
        let inputs = NonuniformInputs {
            scenario: "tandem_top_lug".to_string(),
            dist_a: 3.0,
            dist_b: 5.0,
            cog_height: 2.0,
            tilt_angle: 0.0,
            ..NonuniformInputs::default()
        };
        let result = solve_tandem_2_crane(&inputs, 60000.0).unwrap();
        assert!((result.dist_b_new - 5.0).abs() < TOL);
        assert!((result.lug2_travel - 0.0).abs() < TOL);
        assert!((result.span_new - 8.0).abs() < TOL);
        assert!((result.required_wll_a - 37500.0).abs() < TOL);
        assert!((result.required_wll_b - 22500.0).abs() < TOL);
        assert_eq!(result.governing_case_b, "level");
    }

    #[test]
    fn each_crane_is_booked_against_its_own_rating() {
        let inputs = NonuniformInputs {
            scenario: "tandem_aligned_pivot".to_string(),
            dist_a: 3.0,
            dist_b: 5.0,
            cog_height: 2.0,
            tilt_angle: 35.0,
            crane_rated_a: 50000.0,
            crane_rated_b: 68000.0,
            ..NonuniformInputs::default()
        };
        let result = solve_tandem_2_crane(&inputs, 60000.0).unwrap();
        assert!((result.crane_usage_a.unwrap() - 75.0).abs() < TOL);
        assert!((result.crane_usage_b.unwrap() - 88.23529411764706).abs() < TOL);
        assert_eq!(result.crane_usage_state_a, Some(UsageState::Within));
        assert_eq!(result.crane_usage_state_b, Some(UsageState::Caution));
        assert_eq!(result.crane_risk_flag, Some(true));
        assert_eq!(result.equipment_ok, None);
    }

    #[test]
    fn the_governing_case_is_named_in_words() {
        assert_eq!(governing_case_words(Some("level")), "level baseline");
        assert_eq!(governing_case_words(Some("tilt")), "declared tilt");
        assert_eq!(
            governing_case_words(Some("vertical")),
            "full 90\u{00b0} vertical rotation"
        );
        assert_eq!(governing_case_words(None), "\u{2014}");
    }

    #[test]
    fn the_cog_must_sit_between_the_pick_points() {
        for cog in [0.0, 6.0] {
            let inputs = NonuniformInputs {
                scenario: "asym_2leg_shortening".to_string(),
                sling_length: 7.0,
                pick_distance: 5.0,
                cog_from_pick1: cog,
                pick_point2_height: 0.5,
                ..NonuniformInputs::default()
            };
            assert_eq!(
                solve_nonuniform(&inputs, 2000.0).unwrap_err().0,
                "The COG must sit between the two pick points: enter a position \
                 greater than 0 m and less than the 5.000 m span."
            );
        }
    }

    #[test]
    fn the_sling_must_span_the_longer_lever_arm() {
        let inputs = NonuniformInputs {
            scenario: "asym_2leg_shortening".to_string(),
            sling_length: 7.0,
            pick_distance: 15.0,
            cog_from_pick1: 8.0,
            ..NonuniformInputs::default()
        };
        assert_eq!(
            solve_nonuniform(&inputs, 2000.0).unwrap_err().0,
            "The 7.000 m sling is too short to reach 8.000 m across. Use a \
             longer sling or a shorter span."
        );
    }

    #[test]
    fn a_step_can_lift_a_pick_point_above_hook_level() {
        let inputs = NonuniformInputs {
            scenario: "asym_2leg_shortening".to_string(),
            sling_length: 7.0,
            pick_distance: 5.0,
            cog_from_pick1: 3.5,
            pick_point2_height: 7.0,
            ..NonuniformInputs::default()
        };
        assert_eq!(
            solve_nonuniform(&inputs, 2000.0).unwrap_err().0,
            "A 7.000 m step leaves pick point 2 at or above hook level, so the \
             sling would have to run uphill. Reduce the step or use a longer sling."
        );
    }

    #[test]
    fn the_top_lug_span_runs_out_before_vertical() {
        let inputs = NonuniformInputs {
            scenario: "tandem_top_lug".to_string(),
            dist_a: 3.0,
            dist_b: 5.0,
            cog_height: 2.0,
            tilt_angle: 80.0,
            ..NonuniformInputs::default()
        };
        assert_eq!(
            solve_nonuniform(&inputs, 60000.0).unwrap_err().0,
            "At a 80.00 degree tilt with the COG 2.000 m below the baseline, \
             crane 2's projected distance closes to nothing (the span runs out \
             at 68.2 degrees). The load would tip about crane 2 instead of \
             hanging from both hooks; reduce the tilt angle or the COG depth."
        );
    }

    #[test]
    fn a_blank_form_is_refused_by_name_rather_than_by_division() {
        assert_eq!(
            solve_nonuniform(&NonuniformInputs::default(), 2000.0)
                .unwrap_err()
                .0,
            "Sling length must be greater than zero."
        );
        let tandem_inputs = NonuniformInputs {
            scenario: "tandem_aligned_pivot".to_string(),
            ..NonuniformInputs::default()
        };
        assert_eq!(
            solve_nonuniform(&tandem_inputs, 60000.0).unwrap_err().0,
            "Distance from crane 1 hook to COG must be greater than zero."
        );
    }

    #[test]
    fn the_tilt_and_the_cog_depth_have_their_own_bounds() {
        let over_tilt = NonuniformInputs {
            scenario: "tandem_aligned_pivot".to_string(),
            dist_a: 3.0,
            dist_b: 5.0,
            cog_height: 2.0,
            tilt_angle: 95.0,
            ..NonuniformInputs::default()
        };
        assert_eq!(
            solve_nonuniform(&over_tilt, 60000.0).unwrap_err().0,
            "The cargo tilt angle must be between 0 and 90 degrees; 95.00 was entered."
        );
        let negative_cog = NonuniformInputs {
            scenario: "tandem_aligned_pivot".to_string(),
            dist_a: 3.0,
            dist_b: 5.0,
            cog_height: -1.0,
            tilt_angle: 35.0,
            ..NonuniformInputs::default()
        };
        assert_eq!(
            solve_nonuniform(&negative_cog, 60000.0).unwrap_err().0,
            "The COG depth cannot be negative: measure it downward from the \
             line joining the two lifting points."
        );
    }

    #[test]
    fn every_scenario_in_the_registry_can_be_solved() {
        for scenario in scenario_order() {
            let result = solve_nonuniform(&example_nonuniform_inputs(scenario).unwrap(), 2000.0)
                .unwrap();
            assert_eq!(result.scenario(), scenario);
            assert!(!result.scenario_label().is_empty());
        }
    }
}
