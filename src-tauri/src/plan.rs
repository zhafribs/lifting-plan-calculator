//! A saved plan: the wire state plus the report settings, parsed and validated
//! at the trust boundary. The frontend never parses a plan file.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dto::{AppState, PlanSignoff};
use crate::nonuniform::{SCENARIO_SPECS, TANDEM_METHODS};
use crate::outrigger::GROUND_TYPES;
use crate::report::PageMargins;
use crate::sling::HITCH_SPECS;

pub const FORMAT: &str = "lifting-plan-calculator/plan";
pub const VERSION: u32 = 1;
pub const MAX_BYTES: u64 = 4 * 1024 * 1024;

const MAX_STRING: usize = 500;
const MAX_ITEMS: usize = 500;
const MAX_ROWS: usize = 10_000;
const MAX_POINTS: usize = 50_000;
const MAX_MONEY: f64 = 1e7;
const MAX_LENGTH: f64 = 1e6;
const MAX_POSITION: f64 = 1e6;
const MAX_ANGLE: f64 = 360.0;
const MAX_QTY: i64 = 1_000_000;

const SECTION_KEYS: [&str; 7] = [
    "plan",
    "crane",
    "lines",
    "outrigger",
    "uniform",
    "nonuniform",
    "tandem",
];
const EXCLUSIVE: [&str; 3] = ["uniform", "nonuniform", "tandem"];
const STATE_KEYS: [&str; 7] = [
    "load_items",
    "tackle_items",
    "crane",
    "outrigger",
    "sling",
    "nonuniform",
    "tandem",
];
const REPORT_KEYS: [&str; 9] = [
    "sections",
    "page_breaks",
    "diagram",
    "diagram_state",
    "signoff",
    "footer_text",
    "margins",
    "diagram_scale",
    "issued_at",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFile {
    pub format: String,
    pub version: u32,
    pub state: AppState,
    pub report: PlanReport,
}

/// The Graph tab's drawing state: the geometry the printed diagram was drawn
/// from. Without it, opening a plan would redraw the default sweep and labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiagramState {
    pub config: String,
    pub boom_length: f64,
    pub jib_length: f64,
    pub ext_jib_length: f64,
    pub jib_offset: f64,
    pub angle: f64,
    pub env_min: f64,
    pub env_max: f64,
    pub crane_x: f64,
    pub crane_y: f64,
}

impl Default for DiagramState {
    fn default() -> Self {
        Self {
            config: "boom".to_string(),
            boom_length: 30.0,
            jib_length: 12.0,
            ext_jib_length: 24.0,
            jib_offset: 0.0,
            angle: 45.0,
            env_min: 0.0,
            env_max: 80.0,
            crane_x: 0.0,
            crane_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanReport {
    pub sections: Vec<String>,
    pub page_breaks: Vec<String>,
    pub diagram: bool,
    pub diagram_state: DiagramState,
    pub signoff: PlanSignoff,
    pub footer_text: String,
    pub margins: PageMargins,
    pub diagram_scale: f64,
    pub issued_at: String,
}

impl PlanFile {
    /// Read a plan from bytes, refusing anything that is not a valid version 1
    /// plan. Every refusal names what is wrong; none returns a partial plan.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() as u64 > MAX_BYTES {
            return Err("That plan file is larger than 4 MB.".to_string());
        }
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|_| "That file is not a lifting plan: it is not valid JSON.".to_string())?;
        if value.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err("That file is not a Lifting Plan Calculator plan.".to_string());
        }
        match value.get("version").and_then(Value::as_u64) {
            Some(version) if version == VERSION as u64 => {}
            Some(version) => {
                return Err(format!(
                    "That plan needs a newer version of the app (format {version}); \
                     update the app to open it."
                ))
            }
            None => return Err("The plan file has no format version.".to_string()),
        }
        let state = value
            .get("state")
            .and_then(Value::as_object)
            .ok_or_else(|| "The plan file has no plan data.".to_string())?;
        for key in STATE_KEYS {
            if !state.contains_key(key) {
                return Err(format!("The plan file is missing '{key}'."));
            }
        }
        let report = value
            .get("report")
            .and_then(Value::as_object)
            .ok_or_else(|| "The plan file has no report settings.".to_string())?;
        for key in REPORT_KEYS {
            if !report.contains_key(key) {
                return Err(format!("The plan file is missing 'report.{key}'."));
            }
        }
        let plan: Self = serde_json::from_value(value)
            .map_err(|error| format!("The plan file is damaged: {error}"))?;
        plan.validate()?;
        Ok(plan)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| format!("The plan could not be written: {error}"))?;
        // The same cap the reader applies: a plan the app writes must be one it
        // can open again.
        if bytes.len() as u64 > MAX_BYTES {
            return Err(
                "The plan is larger than 4 MB; reduce the chart or the number of lines."
                    .to_string(),
            );
        }
        Ok(bytes)
    }

    fn validate(&self) -> Result<(), String> {
        if self.format != FORMAT {
            return Err("That is not a Lifting Plan Calculator plan.".to_string());
        }
        if self.version != VERSION {
            return Err(format!("Unsupported plan format {}.", self.version));
        }

        // Every string is bounded and free of control characters; every number
        // is finite and within a sane magnitude. The tighter bounds below then
        // check the fields whose range matters.
        let value = serde_json::to_value(self)
            .map_err(|error| format!("The plan could not be checked: {error}"))?;
        walk(&value, "plan")?;

        validate_state(&self.state)?;
        validate_report(&self.report)?;
        Ok(())
    }
}

fn walk(value: &Value, path: &str) -> Result<(), String> {
    match value {
        Value::String(text) => {
            if text.chars().count() > MAX_STRING {
                return Err(format!("{path} is longer than {MAX_STRING} characters."));
            }
            if text
                .chars()
                .any(|character| character.is_control() && character != '\n' && character != '\t')
            {
                return Err(format!("{path} contains an unsupported control character."));
            }
        }
        Value::Number(number) => {
            let value = number
                .as_f64()
                .ok_or_else(|| format!("{path} is not a number."))?;
            if !value.is_finite() || value.abs() > 1e9 {
                return Err(format!("{path} is out of range."));
            }
        }
        Value::Array(items) => {
            if items.len() > MAX_POINTS {
                return Err(format!("{path} has too many entries."));
            }
            for (index, item) in items.iter().enumerate() {
                walk(item, &format!("{path}[{index}]"))?;
            }
        }
        Value::Object(fields) => {
            for (key, field) in fields {
                walk(field, &format!("{path}.{key}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn number(value: f64, min: f64, max: f64, what: &str) -> Result<(), String> {
    if !value.is_finite() || value < min || value > max {
        Err(format!("{what} is out of range."))
    } else {
        Ok(())
    }
}

fn quantity(value: i64, what: &str) -> Result<(), String> {
    if !(0..=MAX_QTY).contains(&value) {
        Err(format!("{what} is out of range."))
    } else {
        Ok(())
    }
}

fn one_of(value: &str, allowed: &[&str], what: &str) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!("{what} is not a value this app knows."))
    }
}

fn validate_state(state: &AppState) -> Result<(), String> {
    if state.load_items.len() > MAX_ITEMS {
        return Err(format!("The plan has more than {MAX_ITEMS} load lines."));
    }
    if state.tackle_items.len() > MAX_ITEMS {
        return Err(format!("The plan has more than {MAX_ITEMS} tackle lines."));
    }
    let mut ids = HashSet::new();
    for (index, item) in state.load_items.iter().enumerate() {
        let at = format!("load_items[{index}]");
        if item.id == 0 || !ids.insert(item.id) {
            return Err(format!("{at}.id is missing or repeated."));
        }
        quantity(item.qty, &format!("{at}.qty"))?;
        for (field, value) in [
            ("weight_kg", item.weight_kg),
            ("kg_per_metre", item.kg_per_metre),
            ("length_m", item.length_m),
            ("volume_m3", item.volume_m3),
            ("density_kg_m3", item.density_kg_m3),
        ] {
            number(value, 0.0, MAX_MONEY, &format!("{at}.{field}"))?;
        }
    }
    for (index, item) in state.tackle_items.iter().enumerate() {
        let at = format!("tackle_items[{index}]");
        if item.id == 0 || !ids.insert(item.id) {
            return Err(format!("{at}.id is missing or repeated."));
        }
        quantity(item.qty, &format!("{at}.qty"))?;
        number(item.weight_kg, 0.0, MAX_MONEY, &format!("{at}.weight_kg"))?;
    }

    let crane = &state.crane;
    number(crane.capacity_t, 0.0, MAX_MONEY, "crane.capacity_t")?;
    number(
        crane.working_radius,
        0.0,
        MAX_LENGTH,
        "crane.working_radius",
    )?;
    if crane.rows.len() > MAX_ROWS {
        return Err(format!("crane.rows has more than {MAX_ROWS} rows."));
    }
    for (index, row) in crane.rows.iter().enumerate() {
        let at = format!("crane.rows[{index}]");
        number(row.radius, 0.0, MAX_LENGTH, &format!("{at}.radius"))?;
        number(row.capacity, 0.0, MAX_MONEY, &format!("{at}.capacity"))?;
        number(row.angle, 0.0, MAX_ANGLE, &format!("{at}.angle"))?;
        number(row.boom, 0.0, MAX_LENGTH, &format!("{at}.boom"))?;
    }
    if crane.chart_points.len() > MAX_POINTS {
        return Err(format!(
            "crane.chart_points has more than {MAX_POINTS} points."
        ));
    }
    for (index, (radius, capacity)) in crane.chart_points.iter().enumerate() {
        number(
            *radius,
            0.0,
            MAX_LENGTH,
            &format!("crane.chart_points[{index}].radius"),
        )?;
        number(
            *capacity,
            0.0,
            MAX_MONEY,
            &format!("crane.chart_points[{index}].capacity"),
        )?;
    }
    if crane.chart_jibs.len() > MAX_ROWS {
        return Err(format!("crane.chart_jibs has more than {MAX_ROWS} entries."));
    }
    for (index, jib) in crane.chart_jibs.iter().enumerate() {
        number(
            jib.length,
            0.0,
            MAX_LENGTH,
            &format!("crane.chart_jibs[{index}].length"),
        )?;
        number(
            jib.offset,
            0.0,
            MAX_ANGLE,
            &format!("crane.chart_jibs[{index}].offset"),
        )?;
    }
    for (index, (angle, capacity)) in crane.chart_jib_points.iter().enumerate() {
        number(
            *angle,
            0.0,
            MAX_ANGLE,
            &format!("crane.chart_jib_points[{index}].angle"),
        )?;
        number(
            *capacity,
            0.0,
            MAX_MONEY,
            &format!("crane.chart_jib_points[{index}].capacity"),
        )?;
    }
    for (index, value) in crane.chart_booms.iter().enumerate() {
        number(*value, 0.0, MAX_LENGTH, &format!("crane.chart_booms[{index}]"))?;
    }
    for (index, value) in crane.chart_radii.iter().enumerate() {
        number(*value, 0.0, MAX_LENGTH, &format!("crane.chart_radii[{index}]"))?;
    }
    for (name, value) in [
        ("chart_boom", crane.chart_boom),
        ("jib_length", crane.jib_length),
        ("jib_offset", crane.jib_offset),
    ] {
        if let Some(value) = value {
            number(value, 0.0, MAX_LENGTH, &format!("crane.{name}"))?;
        }
    }
    if let Some((x, y)) = crane.chart_position {
        number(x, -MAX_POSITION, MAX_POSITION, "crane.chart_position.x")?;
        number(y, -MAX_POSITION, MAX_POSITION, "crane.chart_position.y")?;
    }
    if let Some(angle) = crane.boom_angle_deg {
        number(angle, 0.0, MAX_ANGLE, "crane.boom_angle_deg")?;
    }
    one_of(
        &crane.chart_source,
        &["", "excel"],
        "crane.chart_source",
    )?;
    one_of(
        &crane.jib_config,
        &["", "boom", "boom_jib", "boom_ext_jib"],
        "crane.jib_config",
    )?;

    let out = &state.outrigger;
    for (field, value) in [
        ("bearing_kpa", out.bearing_kpa),
        ("crane_mass_kn", out.crane_mass_kn),
        ("lift_load_kn", out.lift_load_kn),
        ("pad_width_m", out.pad_width_m),
        ("crane_capacity_t", out.crane_capacity_t),
        ("foot_length_m", out.foot_length_m),
        ("foot_width_m", out.foot_width_m),
    ] {
        number(value, 0.0, MAX_MONEY, &format!("outrigger.{field}"))?;
    }
    if !out.ground_key.is_empty() {
        one_of(
            &out.ground_key,
            &GROUND_TYPES.map(|ground| ground.key),
            "outrigger.ground_key",
        )?;
    }
    one_of(
        &out.pad_shape,
        &["square", "rectangular"],
        "outrigger.pad_shape",
    )?;
    one_of(
        &out.thickness_material,
        &["steel", "timber"],
        "outrigger.thickness_material",
    )?;

    let sling = &state.sling;
    one_of(&sling.hitch, &HITCH_SPECS.map(|spec| spec.key), "sling.hitch")?;
    one_of(
        &sling.angle_basis,
        &["internal", "external"],
        "sling.angle_basis",
    )?;
    for (field, value) in [
        ("sling_length", sling.sling_length),
        ("diameter", sling.diameter),
        ("side", sling.side),
        ("bottom", sling.bottom),
        ("tag_wll", sling.tag_wll),
        ("pick_distance", sling.pick_distance),
        ("pick_length", sling.pick_length),
        ("pick_width", sling.pick_width),
        ("secondary_length", sling.secondary_length),
        ("secondary_spread", sling.secondary_spread),
        ("secondary_tag_wll", sling.secondary_tag_wll),
    ] {
        number(value, 0.0, MAX_MONEY, &format!("sling.{field}"))?;
    }
    for (field, value) in [
        ("choke_angle", sling.choke_angle),
        ("tag_wll_angle", sling.tag_wll_angle),
        ("secondary_tag_wll_angle", sling.secondary_tag_wll_angle),
    ] {
        number(value, 0.0, MAX_ANGLE, &format!("sling.{field}"))?;
    }

    let scenarios = SCENARIO_SPECS.map(|spec| spec.key);
    let tandem_methods = TANDEM_METHODS.map(|method| method.key);
    validate_nonuniform(&state.nonuniform, "nonuniform", &scenarios)?;
    validate_nonuniform(&state.tandem, "tandem", &tandem_methods)?;
    Ok(())
}

fn validate_nonuniform(
    inputs: &crate::nonuniform::NonuniformInputs,
    at: &str,
    scenarios: &[&str],
) -> Result<(), String> {
    one_of(&inputs.scenario, scenarios, &format!("{at}.scenario"))?;
    for (field, value) in [
        ("sling_length", inputs.sling_length),
        ("pick_distance", inputs.pick_distance),
        ("pick_point2_height", inputs.pick_point2_height),
        ("tag_wll1", inputs.tag_wll1),
        ("tag_wll2", inputs.tag_wll2),
        ("dist_a", inputs.dist_a),
        ("dist_b", inputs.dist_b),
        ("tag_wll_a", inputs.tag_wll_a),
        ("tag_wll_b", inputs.tag_wll_b),
        ("crane_rated_a", inputs.crane_rated_a),
        ("crane_rated_b", inputs.crane_rated_b),
    ] {
        number(value, 0.0, MAX_MONEY, &format!("{at}.{field}"))?;
    }
    number(
        inputs.cog_from_pick1,
        -MAX_POSITION,
        MAX_POSITION,
        &format!("{at}.cog_from_pick1"),
    )?;
    number(
        inputs.cog_height,
        -MAX_POSITION,
        MAX_POSITION,
        &format!("{at}.cog_height"),
    )?;
    number(
        inputs.base_reeve_factor,
        0.001,
        100.0,
        &format!("{at}.base_reeve_factor"),
    )?;
    for (field, value) in [
        ("tag_wll1_angle", inputs.tag_wll1_angle),
        ("tag_wll2_angle", inputs.tag_wll2_angle),
        ("tilt_angle", inputs.tilt_angle),
        ("tag_wll_a_angle", inputs.tag_wll_a_angle),
        ("tag_wll_b_angle", inputs.tag_wll_b_angle),
    ] {
        number(value, 0.0, MAX_ANGLE, &format!("{at}.{field}"))?;
    }
    for (field, value) in [
        ("sling_legs1", inputs.sling_legs1),
        ("sling_legs2", inputs.sling_legs2),
        ("sling_legs_a", inputs.sling_legs_a),
        ("sling_legs_b", inputs.sling_legs_b),
    ] {
        quantity(value, &format!("{at}.{field}"))?;
    }
    Ok(())
}

fn validate_report(report: &PlanReport) -> Result<(), String> {
    let mut enabled = HashSet::new();
    for key in &report.sections {
        one_of(key, &SECTION_KEYS, "report.sections")?;
        if !enabled.insert(key.as_str()) {
            return Err(format!("report.sections repeats '{key}'."));
        }
    }
    let exclusive = report
        .sections
        .iter()
        .filter(|key| EXCLUSIVE.contains(&key.as_str()))
        .count();
    if exclusive > 1 {
        return Err("The plan enables more than one sling arrangement.".to_string());
    }
    let mut breaks = HashSet::new();
    for key in &report.page_breaks {
        one_of(key, &SECTION_KEYS, "report.page_breaks")?;
        if !breaks.insert(key.as_str()) {
            return Err(format!("report.page_breaks repeats '{key}'."));
        }
        if !enabled.contains(key.as_str()) {
            return Err(format!(
                "report.page_breaks starts '{key}' on a new page, but that section is off."
            ));
        }
    }

    let margins = report.margins;
    for (field, value) in [
        ("top", margins.top),
        ("right", margins.right),
        ("bottom", margins.bottom),
        ("left", margins.left),
    ] {
        number(value, 5.0, 50.0, &format!("report.margins.{field}"))?;
    }
    if 210.0 - margins.left - margins.right < 60.0
        || 297.0 - margins.top - margins.bottom < 60.0
    {
        return Err("The plan's margins leave less than 60 mm of text area.".to_string());
    }
    number(
        report.diagram_scale,
        25.0,
        100.0,
        "report.diagram_scale",
    )?;
    validate_diagram_state(&report.diagram_state)?;
    validate_issued_at(&report.issued_at)?;
    Ok(())
}

fn validate_diagram_state(state: &DiagramState) -> Result<(), String> {
    let at = "report.diagram_state";
    one_of(
        &state.config,
        &["boom", "boom_jib", "boom_ext_jib"],
        &format!("{at}.config"),
    )?;
    for (field, value) in [
        ("boom_length", state.boom_length),
        ("jib_length", state.jib_length),
        ("ext_jib_length", state.ext_jib_length),
    ] {
        number(value, 0.0, MAX_LENGTH, &format!("{at}.{field}"))?;
    }
    // The Graph tab allows a jib offset from -30 to 60 degrees.
    number(state.jib_offset, -30.0, 60.0, &format!("{at}.jib_offset"))?;
    for (field, value) in [
        ("angle", state.angle),
        ("env_min", state.env_min),
        ("env_max", state.env_max),
    ] {
        number(value, 0.0, MAX_ANGLE, &format!("{at}.{field}"))?;
    }
    if state.env_min > state.env_max {
        return Err(format!("{at} ends its sweep before it starts."));
    }
    number(
        state.crane_x,
        -MAX_POSITION,
        MAX_POSITION,
        &format!("{at}.crane_x"),
    )?;
    number(
        state.crane_y,
        -MAX_POSITION,
        MAX_POSITION,
        &format!("{at}.crane_y"),
    )?;
    Ok(())
}

fn validate_issued_at(value: &str) -> Result<(), String> {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 3
        || parts[0].len() != 2
        || parts[1].len() != 2
        || parts[2].len() != 4
        || parts
            .iter()
            .any(|part| !part.chars().all(|character| character.is_ascii_digit()))
    {
        return Err("report.issued_at is not DD-MM-YYYY.".to_string());
    }
    let day: u32 = parts[0]
        .parse()
        .map_err(|_| "report.issued_at is not DD-MM-YYYY.".to_string())?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| "report.issued_at is not DD-MM-YYYY.".to_string())?;
    let year: u32 = parts[2]
        .parse()
        .map_err(|_| "report.issued_at is not DD-MM-YYYY.".to_string())?;
    if !(1..=12).contains(&month) || !(2000..=2100).contains(&year) {
        return Err("report.issued_at is not a real date.".to_string());
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if day < 1 || day > days {
        return Err("report.issued_at is not a real date.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::LoadType;
    use crate::crane::CraneInputs;
    use crate::model::{LoadItem, TackleItem};
    use crate::nonuniform::{example_nonuniform_inputs, NonuniformInputs};
    use crate::outrigger::example_outrigger_inputs;
    use crate::sling::example_inputs;

    fn sample() -> PlanFile {
        let mut sling = example_inputs("two_leg_nested_3leg").unwrap();
        sling.tag_wll = 5000.0;
        let nonuniform = example_nonuniform_inputs("asym_2leg_shortening").unwrap();
        let tandem = example_nonuniform_inputs("tandem_aligned_pivot").unwrap();
        PlanFile {
            format: FORMAT.to_string(),
            version: VERSION,
            state: AppState {
                load_items: vec![LoadItem {
                    id: 1,
                    name: "Steel plate".to_string(),
                    load_type: LoadType::Fixed,
                    qty: 1,
                    weight_kg: 1500.0,
                    ..LoadItem::default()
                }],
                tackle_items: vec![TackleItem {
                    id: 2,
                    description: "4-leg chain".to_string(),
                    qty: 1,
                    weight_kg: 120.0,
                    ..TackleItem::default()
                }],
                crane: CraneInputs {
                    name: "Test crane".to_string(),
                    capacity_t: 50.0,
                    working_radius: 12.0,
                    ..CraneInputs::default()
                },
                outrigger: example_outrigger_inputs(),
                sling,
                nonuniform: NonuniformInputs {
                    scenario: "asym_2leg_shortening".to_string(),
                    ..nonuniform
                },
                tandem,
            },
            report: PlanReport {
                sections: vec!["plan".to_string(), "crane".to_string()],
                page_breaks: vec!["crane".to_string()],
                diagram: false,
                diagram_state: DiagramState::default(),
                signoff: PlanSignoff {
                    reference: "LP-2026-001".to_string(),
                    ..PlanSignoff::default()
                },
                footer_text: "Prepared with Lifting Plan Calculator".to_string(),
                margins: PageMargins::default(),
                diagram_scale: 50.0,
                issued_at: "28-11-2026".to_string(),
            },
        }
    }

    fn parsed(plan: &PlanFile) -> PlanFile {
        PlanFile::parse(&plan.to_bytes().unwrap()).unwrap()
    }

    #[test]
    fn a_plan_round_trips() {
        let plan = sample();
        let again = parsed(&plan);
        assert!(!again.report.diagram);
        assert_eq!(again.report.signoff.reference, "LP-2026-001");
        assert_eq!(again.report.issued_at, "28-11-2026");
        assert_eq!(again.state.load_items.len(), 1);
    }

    #[test]
    fn the_outrigger_linked_flags_round_trip() {
        let mut plan = sample();
        plan.state.outrigger.crane_mass_linked = false;
        plan.state.outrigger.lift_load_linked = false;
        let again = parsed(&plan);
        assert!(!again.state.outrigger.crane_mass_linked);
        assert!(!again.state.outrigger.lift_load_linked);
    }

    #[test]
    fn a_wrong_format_is_refused() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["format"] = Value::String("something/else".to_string());
        let error = PlanFile::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.contains("not a Lifting Plan Calculator plan"));
    }

    #[test]
    fn a_newer_version_is_refused_by_name() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["version"] = Value::from(2);
        let error = PlanFile::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.contains("format 2"));
    }

    #[test]
    fn a_missing_state_key_is_refused() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["state"].as_object_mut().unwrap().remove("crane");
        let error = PlanFile::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.contains("missing 'crane'"));
    }

    #[test]
    fn a_missing_report_key_is_refused() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["report"].as_object_mut().unwrap().remove("diagram");
        let error = PlanFile::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.contains("report.diagram"));
    }

    #[test]
    fn a_negative_weight_is_refused() {
        let mut plan = sample();
        plan.state.load_items[0].weight_kg = -1.0;
        assert!(plan.validate().unwrap_err().contains("weight_kg"));
    }

    #[test]
    fn an_absurd_capacity_is_refused() {
        let mut plan = sample();
        plan.state.crane.capacity_t = 1e9;
        assert!(plan.validate().unwrap_err().contains("capacity_t"));
    }

    #[test]
    fn duplicate_line_ids_are_refused() {
        let mut plan = sample();
        plan.state.load_items.push(LoadItem {
            id: 1,
            ..LoadItem::default()
        });
        assert!(plan.validate().unwrap_err().contains("id"));
    }

    #[test]
    fn an_unknown_catalog_key_is_refused() {
        let mut plan = sample();
        plan.state.outrigger.ground_key = "not_a_ground".to_string();
        assert!(plan.validate().unwrap_err().contains("ground_key"));
        let mut plan = sample();
        plan.state.sling.hitch = "not_a_hitch".to_string();
        assert!(plan.validate().unwrap_err().contains("hitch"));
        let mut plan = sample();
        plan.state.nonuniform.scenario = "not_a_scenario".to_string();
        assert!(plan.validate().unwrap_err().contains("scenario"));
        let mut plan = sample();
        plan.state.sling.angle_basis = "sideways".to_string();
        assert!(plan.validate().unwrap_err().contains("angle_basis"));
    }

    #[test]
    fn an_unknown_section_or_two_arrangements_are_refused() {
        let mut plan = sample();
        plan.report.sections = vec!["plan".to_string(), "not_a_section".to_string()];
        assert!(plan.validate().unwrap_err().contains("sections"));
        let mut plan = sample();
        plan.report.sections = vec!["uniform".to_string(), "nonuniform".to_string()];
        assert!(plan.validate().unwrap_err().contains("sling arrangement"));
    }

    #[test]
    fn a_page_break_on_a_disabled_section_is_refused() {
        let mut plan = sample();
        plan.report.page_breaks = vec!["outrigger".to_string()];
        assert!(plan.validate().unwrap_err().contains("that section is off"));
    }

    #[test]
    fn margins_and_scale_are_bounded() {
        let mut plan = sample();
        plan.report.margins.left = 4.0;
        assert!(plan.validate().unwrap_err().contains("margins.left"));
        let mut plan = sample();
        plan.report.margins.left = 50.0;
        plan.report.margins.right = 50.0;
        assert!(plan.validate().is_ok());
        let mut plan = sample();
        plan.report.diagram_scale = 10.0;
        assert!(plan.validate().unwrap_err().contains("diagram_scale"));
    }

    #[test]
    fn a_bad_issue_date_is_refused() {
        let mut plan = sample();
        plan.report.issued_at = "2026-11-28".to_string();
        assert!(plan.validate().unwrap_err().contains("DD-MM-YYYY"));
        let mut plan = sample();
        plan.report.issued_at = "31-02-2026".to_string();
        assert!(plan.validate().unwrap_err().contains("real date"));
    }

    #[test]
    fn a_date_must_be_digits() {
        let mut plan = sample();
        plan.report.issued_at = "+1-01-2026".to_string();
        assert!(plan.validate().unwrap_err().contains("DD-MM-YYYY"));
    }

    #[test]
    fn a_long_string_is_refused() {
        let mut plan = sample();
        plan.state.load_items[0].name = "x".repeat(MAX_STRING + 1);
        assert!(plan.validate().unwrap_err().contains("longer than"));
    }

    #[test]
    fn the_diagram_state_round_trips() {
        let mut plan = sample();
        plan.report.diagram_state = DiagramState {
            config: "boom_jib".to_string(),
            angle: 30.0,
            env_min: 10.0,
            env_max: 70.0,
            crane_x: 3.5,
            ..DiagramState::default()
        };
        let again = parsed(&plan);
        assert_eq!(again.report.diagram_state.config, "boom_jib");
        assert!((again.report.diagram_state.angle - 30.0).abs() < 1e-9);
        assert!((again.report.diagram_state.env_max - 70.0).abs() < 1e-9);
        assert!((again.report.diagram_state.crane_x - 3.5).abs() < 1e-9);
    }

    #[test]
    fn a_negative_jib_offset_is_allowed() {
        let mut plan = sample();
        plan.report.diagram_state.config = "boom_jib".to_string();
        plan.report.diagram_state.jib_offset = -10.0;
        assert!(plan.validate().is_ok());
        plan.report.diagram_state.jib_offset = -31.0;
        assert!(plan.validate().unwrap_err().contains("jib_offset"));
        plan.report.diagram_state.jib_offset = 61.0;
        assert!(plan.validate().unwrap_err().contains("jib_offset"));
    }

    #[test]
    fn a_bad_diagram_state_is_refused() {
        let mut plan = sample();
        plan.report.diagram_state.config = "jib".to_string();
        assert!(plan.validate().unwrap_err().contains("diagram_state.config"));
        let mut plan = sample();
        plan.report.diagram_state.env_min = 90.0;
        plan.report.diagram_state.env_max = 45.0;
        assert!(plan.validate().unwrap_err().contains("sweep"));
        let mut plan = sample();
        plan.report.diagram_state.angle = 400.0;
        assert!(plan.validate().unwrap_err().contains("diagram_state.angle"));
    }

    #[test]
    fn a_plan_too_large_to_open_cannot_be_written() {
        let mut plan = sample();
        plan.state.crane.chart_points = (0..MAX_POINTS)
            .map(|index| (index as f64, 1000.0))
            .collect();
        plan.state.crane.chart_jib_points =
            (0..MAX_POINTS).map(|_| (100.0, 1000.0)).collect();
        plan.state.crane.chart_booms = (0..MAX_POINTS)
            .map(|index| 1000.0 + index as f64)
            .collect();
        plan.state.crane.chart_radii = (0..MAX_POINTS)
            .map(|index| 1000.0 + index as f64)
            .collect();
        let error = plan.to_bytes().unwrap_err();
        assert!(error.contains("larger than 4 MB"), "{error}");
    }

    #[test]
    fn a_bad_plan_cannot_be_written() {
        let mut plan = sample();
        plan.report.diagram_scale = 0.0;
        assert!(plan.to_bytes().is_err());
    }

    #[test]
    fn an_oversized_file_is_refused_before_parsing() {
        let bytes = vec![b' '; MAX_BYTES as usize + 1];
        let error = PlanFile::parse(&bytes).unwrap_err();
        assert!(error.contains("larger than 4 MB"));
    }

    #[test]
    fn malformed_json_is_refused() {
        let error = PlanFile::parse(b"{ not json").unwrap_err();
        assert!(error.contains("not valid JSON"));
    }
}
