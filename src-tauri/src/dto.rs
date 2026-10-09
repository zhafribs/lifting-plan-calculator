//! The wire shapes the UI talks in.
//!
//! The whole app state comes up with every call, and every solved section comes
//! back together — one round trip per keystroke, and no partial state on the
//! JavaScript side.

use serde::{Deserialize, Serialize};

use crate::calc::LoadType;
use crate::crane::{CraneInputs, CraneResult};
use crate::excel::{chart_table, LoadChartData};
use crate::model::{LiftTotals, LoadItem, TackleItem};
use crate::nonuniform::{
    example_nonuniform_inputs, NonuniformInputs, NonuniformResult, ScenarioSpec, TandemMethod,
    SLING_LEG_OPTIONS, TANDEM_METHODS,
};
use crate::sling::{example_inputs, HitchSpec, OneLegInputs, SlingResult, HITCH_SPECS};

/// The whole document, as the UI holds it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub load_items: Vec<LoadItem>,
    pub tackle_items: Vec<TackleItem>,
    pub crane: CraneInputs,
    pub sling: OneLegInputs,
    pub nonuniform: NonuniformInputs,
    pub tandem: NonuniformInputs,
}

impl AppState {
    pub fn totals(&self) -> LiftTotals {
        LiftTotals::of(&self.load_items, &self.tackle_items)
    }
}

/// One solved section, or why it could not be solved.
#[derive(Debug, Clone, Serialize)]
pub struct Section<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub error: Option<String>,
}

impl<T> Section<T> {
    pub fn solved(result: T) -> Self {
        Section {
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn failed(message: String) -> Self {
        Section {
            ok: false,
            result: None,
            error: Some(message),
        }
    }
}

/// Every section's result, solved from one state.
#[derive(Debug, Clone, Serialize)]
pub struct SolveAll {
    pub totals: LiftTotals,
    /// Per-line weights, in item order, from the engine's own arithmetic.
    pub line_weights: Vec<f64>,
    /// Per-line tackle subtotals, in item order.
    pub tackle_subtotals: Vec<f64>,
    pub crane: Section<CraneResult>,
    pub sling: Section<SlingResult>,
    pub nonuniform: Section<NonuniformResult>,
    pub tandem: Section<NonuniformResult>,
}

/// One hitch arrangement, as the form needs it.
#[derive(Debug, Clone, Serialize)]
pub struct HitchSpecDto {
    pub key: String,
    pub label: String,
    pub shape: String,
    pub effective_legs: i64,
    pub base_reeve_factor: f64,
    pub needs_angle: bool,
    pub angle_description: String,
    pub round_load: bool,
    pub rectangular_load: bool,
    pub bridle: bool,
    pub leg_count: i64,
    pub needs_pick_distance: bool,
    pub nested: i64,
    pub needs_tag_angle: bool,
}

impl From<&HitchSpec> for HitchSpecDto {
    fn from(spec: &HitchSpec) -> Self {
        HitchSpecDto {
            key: spec.key.to_string(),
            label: spec.label.to_string(),
            shape: spec.shape.to_string(),
            effective_legs: spec.effective_legs,
            base_reeve_factor: spec.base_reeve_factor,
            needs_angle: spec.needs_angle,
            angle_description: spec.angle_description.to_string(),
            round_load: spec.round_load,
            rectangular_load: spec.rectangular_load,
            bridle: spec.bridle,
            leg_count: spec.leg_count,
            needs_pick_distance: spec.needs_pick_distance,
            nested: spec.nested,
            needs_tag_angle: spec.needs_tag_angle(),
        }
    }
}

/// The static catalogue: every list and rule the forms render from.
#[derive(Debug, Clone, Serialize)]
pub struct Catalog {
    pub app_name: String,
    pub version: String,
    pub contact_name: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub usage_ratio: f64,
    pub load_types: Vec<LoadTypeDto>,
    pub chart_columns: Vec<ChartColumnDto>,
    pub hitches: Vec<HitchSpecDto>,
    pub scenarios: Vec<ScenarioSpec>,
    pub tandem_methods: Vec<TandemMethodDto>,
    pub sling_leg_options: Vec<i64>,
    pub example_crane: CraneInputs,
    pub example_sling: Vec<(String, OneLegInputs)>,
    pub example_nonuniform: Vec<(String, NonuniformInputs)>,
    pub new_load_item: LoadItem,
    pub new_tackle_item: TackleItem,
    pub katex_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadTypeDto {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChartColumnDto {
    pub key: String,
    pub caption: String,
    pub unit: String,
    pub decimals: usize,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TandemMethodDto {
    pub key: String,
    pub label: String,
    pub full_label: String,
    pub rule: String,
    pub lug2: String,
    pub span_behaviour: String,
    pub max_rotation: String,
    pub use_for: String,
    pub full_rotation: bool,
}

impl From<&TandemMethod> for TandemMethodDto {
    fn from(method: &TandemMethod) -> Self {
        TandemMethodDto {
            key: method.key.to_string(),
            label: method.label.to_string(),
            full_label: method.full_label.to_string(),
            rule: method.rule.to_string(),
            lug2: method.lug2.to_string(),
            span_behaviour: method.span_behaviour.to_string(),
            max_rotation: method.max_rotation.to_string(),
            use_for: method.use_for.to_string(),
            full_rotation: method.full_rotation,
        }
    }
}

/// The KaTeX files bundled with the app, saved alongside a report so the file
/// prints with its equations typeset wherever it is opened.
pub const KATEX_FILES: [&str; 23] = [
    "katex.min.css",
    "katex.min.js",
    "auto-render.min.js",
    "fonts/KaTeX_AMS-Regular.woff2",
    "fonts/KaTeX_Caligraphic-Bold.woff2",
    "fonts/KaTeX_Caligraphic-Regular.woff2",
    "fonts/KaTeX_Fraktur-Bold.woff2",
    "fonts/KaTeX_Fraktur-Regular.woff2",
    "fonts/KaTeX_Main-BoldItalic.woff2",
    "fonts/KaTeX_Main-Bold.woff2",
    "fonts/KaTeX_Main-Italic.woff2",
    "fonts/KaTeX_Main-Regular.woff2",
    "fonts/KaTeX_Math-BoldItalic.woff2",
    "fonts/KaTeX_Math-Italic.woff2",
    "fonts/KaTeX_SansSerif-Bold.woff2",
    "fonts/KaTeX_SansSerif-Italic.woff2",
    "fonts/KaTeX_SansSerif-Regular.woff2",
    "fonts/KaTeX_Script-Regular.woff2",
    "fonts/KaTeX_Size1-Regular.woff2",
    "fonts/KaTeX_Size2-Regular.woff2",
    "fonts/KaTeX_Size3-Regular.woff2",
    "fonts/KaTeX_Size4-Regular.woff2",
    "fonts/KaTeX_Typewriter-Regular.woff2",
];

/// The catalogue, assembled from the engine's own tables.
pub fn build_catalog() -> Catalog {
    Catalog {
        app_name: "Lifting Plan Calculator".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        contact_name: "Zhafri Syazwi".to_string(),
        contact_email: "zhafribs@gmail.com".to_string(),
        contact_phone: "+60 11-2142 5200".to_string(),
        usage_ratio: crate::calc::CRANE_USAGE_RATIO,
        load_types: LoadType::ORDER
            .iter()
            .map(|load_type| LoadTypeDto {
                key: load_type.key().to_string(),
                label: load_type.label().to_string(),
            })
            .collect(),
        chart_columns: crate::crane::CHART_COLUMNS
            .iter()
            .map(|column| ChartColumnDto {
                key: column.key.to_string(),
                caption: column.caption.to_string(),
                unit: column.unit.to_string(),
                decimals: column.decimals,
                min: column.min,
                max: column.max,
            })
            .collect(),
        hitches: HITCH_SPECS.iter().map(HitchSpecDto::from).collect(),
        scenarios: crate::nonuniform::SCENARIO_SPECS.to_vec(),
        tandem_methods: TANDEM_METHODS.iter().map(TandemMethodDto::from).collect(),
        sling_leg_options: SLING_LEG_OPTIONS.to_vec(),
        example_crane: crate::crane::example_crane_inputs(),
        example_sling: HITCH_SPECS
            .iter()
            .filter_map(|spec| example_inputs(spec.key).ok().map(|inputs| (spec.key.to_string(), inputs)))
            .collect(),
        example_nonuniform: crate::nonuniform::SCENARIO_SPECS
            .iter()
            .filter_map(|spec| {
                example_nonuniform_inputs(spec.key)
                    .ok()
                    .map(|inputs| (spec.key.to_string(), inputs))
            })
            .collect(),
        new_load_item: LoadItem::default(),
        new_tackle_item: TackleItem::default(),
        katex_files: KATEX_FILES.iter().map(|name| name.to_string()).collect(),
    }
}

/// A workbook imported into the form, plus the table the tab displays.
#[derive(Debug, Clone, Serialize)]
pub struct ChartImport {
    pub crane: CraneInputs,
    pub table: ChartTable,
}

/// The workbook table under the chart controls.
#[derive(Debug, Clone, Serialize)]
pub struct ChartTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Build the display table for a chart at the given boom.
pub fn table_for(chart: &LoadChartData, boom: Option<f64>) -> ChartTable {
    let (headers, rows) = chart_table(chart, boom);
    ChartTable { headers, rows }
}
