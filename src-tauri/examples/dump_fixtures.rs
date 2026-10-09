//! Dev helper: dump real engine output as JSON fixtures for the UI harness.
//!
//! Run with: `cargo run --example dump_fixtures`
//! Writes `catalog.json`, `solve_all.json` and `report.html` to /tmp/opencode/fixtures.

use std::fs;

use lifting_plan_calculator_lib::crane::{example_crane_inputs, solve_crane};
use lifting_plan_calculator_lib::dto::{build_catalog, AppState, Section, SolveAll};
use lifting_plan_calculator_lib::model::{LoadItem, TackleItem};
use lifting_plan_calculator_lib::nonuniform::{example_nonuniform_inputs, solve_nonuniform};
use lifting_plan_calculator_lib::report::{build_report_html, wrap_katex, ReportInputs};
use lifting_plan_calculator_lib::sling::{example_inputs, solve_one_leg};

fn solve_all(state: &AppState) -> SolveAll {
    let totals = state.totals();
    let line_weights = state.load_items.iter().map(|item| item.weight()).collect();
    let tackle_subtotals = state
        .tackle_items
        .iter()
        .map(|item| item.subtotal())
        .collect();
    SolveAll {
        totals,
        line_weights,
        tackle_subtotals,
        crane: match solve_crane(&state.crane, state.totals().gross) {
            Ok(result) => Section::solved(result),
            Err(error) => Section::failed(error.0),
        },
        sling: match solve_one_leg(&state.sling, state.totals().gross) {
            Ok(result) => Section::solved(result),
            Err(error) => Section::failed(error.0),
        },
        nonuniform: match solve_nonuniform(&state.nonuniform, state.totals().gross) {
            Ok(result) => Section::solved(result),
            Err(error) => Section::failed(error.0),
        },
        tandem: match solve_nonuniform(&state.tandem, state.totals().gross) {
            Ok(result) => Section::solved(result),
            Err(error) => Section::failed(error.0),
        },
    }
}

fn main() {
    let out = "/tmp/opencode/fixtures";
    fs::create_dir_all(out).unwrap();

    let catalog = build_catalog();
    fs::write(
        format!("{out}/catalog.json"),
        serde_json::to_string(&catalog).unwrap(),
    )
    .unwrap();

    // A worked state: crane example, nested sling with tags, the asymmetric
    // bridle with tags, and the tandem lift with tags and crane ratings.
    let mut sling = example_inputs("two_leg_nested_3leg").unwrap();
    sling.tag_wll = 5000.0;
    sling.tag_wll_angle = 45.0;
    sling.secondary_tag_wll = 3000.0;
    sling.secondary_tag_wll_angle = 45.0;

    let mut nonuniform = example_nonuniform_inputs("asym_2leg_shortening").unwrap();
    nonuniform.tag_wll1 = 5000.0;
    nonuniform.sling_legs1 = 2;
    nonuniform.tag_wll1_angle = 45.0;
    nonuniform.tag_wll2 = 6000.0;
    nonuniform.sling_legs2 = 4;
    nonuniform.tag_wll2_angle = 30.0;

    let mut tandem = example_nonuniform_inputs("tandem_aligned_pivot").unwrap();
    tandem.tag_wll_a = 25000.0;
    tandem.sling_legs_a = 2;
    tandem.tag_wll_a_angle = 45.0;
    tandem.tag_wll_b = 30000.0;
    tandem.sling_legs_b = 3;
    tandem.tag_wll_b_angle = 45.0;
    tandem.crane_rated_a = 50000.0;
    tandem.crane_rated_b = 68000.0;

    let state = AppState {
        load_items: vec![
            LoadItem {
                name: "Steel plate".to_string(),
                weight_kg: 1500.0,
                ..LoadItem::default()
            },
            LoadItem {
                name: "Edge beam".to_string(),
                load_type: lifting_plan_calculator_lib::calc::LoadType::PerMetre,
                kg_per_metre: 120.0,
                length_m: 9.0,
                ..LoadItem::default()
            },
        ],
        tackle_items: vec![TackleItem {
            description: "4-leg chain sling".to_string(),
            qty: 1,
            weight_kg: 120.0,
            ..TackleItem::default()
        }],
        crane: example_crane_inputs(),
        sling,
        nonuniform,
        tandem,
        ..AppState::default()
    };

    let solved = solve_all(&state);
    fs::write(
        format!("{out}/solve_all.json"),
        serde_json::to_string(&solved).unwrap(),
    )
    .unwrap();
    fs::write(
        format!("{out}/state.json"),
        serde_json::to_string(&state).unwrap(),
    )
    .unwrap();

    let tandem_result = match &solved.tandem.result {
        Some(lifting_plan_calculator_lib::nonuniform::NonuniformResult::Tandem(result)) => {
            Some(result.clone())
        }
        _ => None,
    };
    let report = wrap_katex(&build_report_html(
        &ReportInputs {
            totals: solved.totals.clone(),
            crane: solved.crane.result.clone(),
            sling: solved.sling.result.clone(),
            nonuniform: solved.nonuniform.result.clone(),
            nonuniform_error: solved.nonuniform.error.clone(),
            tandem: tandem_result,
            tandem_error: solved.tandem.error.clone(),
        },
        &["crane", "uniform", "nonuniform", "tandem"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
    ));
    fs::write(format!("{out}/report.html"), report).unwrap();

    println!("fixtures written to {out}");
}
