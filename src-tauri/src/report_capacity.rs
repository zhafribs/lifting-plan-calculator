//! The rest of the uniform-lift report: the rigging factors, the final
//! capacity and equipment selection, and the notes. Ported from
//! `ReportCapacity.kt`.

use crate::report::{equation, fmt, heading, p, subheading, tex_deg, tex_num};
use crate::sling::{HitchSpec, SlingResult, SINGLE_LEG};

/// The four arrangements a choke reduction applies to.
pub const CHOKE_KEYS: [&str; 4] = [
    "round_choke",
    "rect_choke",
    "two_leg_round_choke",
    "two_leg_rect_choke",
];

/// The arrangements whose factor is the product of a longitudinal and a
/// transverse one.
const TWO_LEG_BASKET_KEYS: [&str; 4] = [
    "two_leg_round_basket",
    "two_leg_rect_basket",
    "two_leg_vertical_round_basket",
    "two_leg_vertical_rect_basket",
];

/// The leg count a sling tag is rated on, as the reference documents state it
/// under their note heading.
const RATING_LEG_NOTE: [&str; 3] = [
    "If 4-Leg Sling, use Sling Legs = 3",
    "If 3-Leg Sling, use Sling Legs = 2",
    "If 2-Leg Sling, use Sling Legs = 2",
];

/// Section 3: the angle factor, the reeve factor and the leg tension.
pub fn factors_section(result: &SlingResult, spec: &HitchSpec) -> String {
    let key = spec.key;
    let weight = result.load_weight;
    let angle = result.angle;
    let af = result.angle_factor;
    let rf = result.base_reeve_factor;
    let reduction = result.choke_reduction_factor;
    let combined = result.combined_reeve_factor;
    let eff = result.effective_legs;
    let tension = result.tension_each;
    let mut out = vec![heading(3, "Rigging Factors & Tension Calculations")];

    if result.nested != 0 {
        let af_long = result.af_long.unwrap_or(1.0);
        let secondary_af = result.secondary_af.unwrap_or(1.0);
        let secondary_combined = result.secondary_combined_af.unwrap_or(1.0);
        let nested_labels = crate::report_uniform::secondary_labels(result, result.nested);
        out.push(subheading("Longitudinal Sling Angle Factor (AF Long):"));
        out.push(equation(&format!(
            "\\text{{AF}}_{{Long}} = \\cos({}) = {}",
            tex_deg(result.angle_long, 2),
            tex_num(Some(af_long), 3)
        )));
        if !nested_labels.is_empty() {
            // The 3- and 4-leg nested documents publish one factor per group.
            for (group, label) in &nested_labels {
                out.push(subheading(&format!(
                    "Transverse Sling Angle Factor for {} Legs (AF Trans {}):",
                    label.token, label.token
                )));
                out.push(equation(&format!(
                    "\\text{{AF}}_{{\\text{{Trans {}}}}} = \\cos({}) = {}",
                    label.token,
                    tex_deg(Some(group.angle), 2),
                    tex_num(Some(group.af), 3)
                )));
                out.push(subheading(&format!(
                    "Combined Sling Angle Factor (AF) for {} Secondary Legs:",
                    label.name
                )));
                out.push(equation(&format!(
                    "\\text{{AF}}_{{\\text{{Combined {}}}}} = \
                     \\text{{AF}}_{{Long}} \\times \\text{{AF}}_{{\\text{{Trans {}}}}} = \
                     {} \\times {} = {}",
                    label.token,
                    label.token,
                    tex_num(Some(af_long), 3),
                    tex_num(Some(group.af), 3),
                    tex_num(Some(group.combined_af), 3)
                )));
            }
        } else {
            out.push(subheading("Transverse Sling Angle Factor (AF Trans):"));
            out.push(equation(&format!(
                "\\text{{AF}}_{{Trans}} = \\cos({}) = {}",
                tex_deg(result.secondary_angle, 2),
                tex_num(Some(secondary_af), 3)
            )));
            out.push(subheading("Combined Sling Angle Factor (AF Combined):"));
            out.push(equation(&format!(
                "\\text{{AF}}_{{Combined}} = \\text{{AF}}_{{Long}} \\times \
                 \\text{{AF}}_{{Trans}} = {} \\times {} = {}",
                tex_num(Some(af_long), 3),
                tex_num(Some(secondary_af), 3),
                tex_num(Some(secondary_combined), 3)
            )));
        }
    } else if TWO_LEG_BASKET_KEYS.contains(&key) {
        let af_long = result.af_long.unwrap_or(1.0);
        let af_trans = result.af_trans.unwrap_or(1.0);
        out.push(subheading("Longitudinal Sling Angle Factor (AF Long):"));
        out.push(equation(&format!(
            "\\text{{AF}}_{{Long}} = \\cos({}) = {}",
            tex_deg(result.angle_long, 2),
            tex_num(Some(af_long), 3)
        )));
        out.push(subheading("Transverse Basket Angle Factor (AF Trans):"));
        out.push(equation(&format!(
            "\\text{{AF}}_{{Trans}} = \\cos({}) = {}",
            tex_deg(result.angle_trans, 2),
            tex_num(Some(af_trans), 3)
        )));
        out.push(subheading("Combined Sling Angle Factor (AF):"));
        out.push(equation(&format!(
            "\\text{{AF}} = \\text{{AF}}_{{Long}} \\times \\text{{AF}}_{{Trans}} = \
             {} \\times {} = {}",
            tex_num(Some(af_long), 3),
            tex_num(Some(af_trans), 3),
            tex_num(Some(af), 3)
        )));
    } else {
        out.push(subheading("Sling Angle Factor (AF):"));
        if CHOKE_KEYS.contains(&key) {
            out.push(p("The main lifting leg stands perfectly vertical (0\u{00b0}).", ""));
        }
        if key == "round_choke" || key == "rect_choke" {
            out.push(equation("\\text{AF} = \\cos(0^\\circ) = 1.0"));
        } else {
            out.push(equation(&format!(
                "\\text{{AF}} = \\cos({}) = {}",
                tex_deg(Some(angle), 2),
                tex_num(Some(af), 3)
            )));
        }
    }

    out.push(subheading("Base Reeve Factor (RF):"));
    out.push(equation(&format!(
        "\\text{{RF}}_{{Base}} = {}",
        tex_num(Some(rf), 2)
    )));
    if CHOKE_KEYS.contains(&key) {
        out.push(subheading("Choke Angle Reduction Factor:"));
        out.push(equation(&format!(
            "\\text{{Reduction Factor}} = {}",
            tex_num(Some(reduction), 2)
        )));
        out.push(subheading("Combined Reeve Factor:"));
        out.push(equation(&format!(
            "\\text{{Combined RF}} = \\text{{RF}}_{{Base}} \\times \
             \\text{{Reduction Factor}} = {} \\times {} = {}",
            tex_num(Some(rf), 2),
            tex_num(Some(reduction), 2),
            tex_num(Some(combined), 4)
        )));
    }

    if result.nested != 0 {
        let secondary_tension = result.secondary_tension.unwrap_or(0.0);
        let secondary_combined = result.secondary_combined_af.unwrap_or(1.0);
        let nested_labels = crate::report_uniform::secondary_labels(result, result.nested);
        let secondary_legs = result.secondary_legs.unwrap_or(1);
        out.push(subheading("Tension in Each First Sling Leg (2 Master Legs Total):"));
        out.push(equation(&format!(
            "\\text{{Tension}}_1 = \\frac{{\\text{{Load Weight}}}}\
             {{\\text{{Sling Legs}}_1 \\times \\text{{AF}}_{{Long}}}} = \
             \\frac{{{}}}{{2 \\times {}}} = {}\\ \\text{{kg}}",
            tex_num(Some(weight), 0),
            tex_num(Some(af), 3),
            tex_num(Some(tension), 1)
        )));
        if !nested_labels.is_empty() {
            // The 3- and 4-leg nested documents state one tension per group.
            for (group, label) in &nested_labels {
                out.push(subheading(&format!("Tension in {}:", label.phrase)));
                out.push(equation(&format!(
                    "\\text{{Tension}}_{{\\text{{Secondary {}}}}} = \
                     \\frac{{\\text{{Load Weight}}}}\
                     {{\\text{{Sling Legs}}_1 \\times \\text{{Sling Legs}}_2 \\times \
                     \\text{{AF}}_{{\\text{{Combined {}}}}}}} = \
                     \\frac{{{}}}{{2 \\times {secondary_legs} \\times {}}} = \
                     {}\\ \\text{{kg}}",
                    label.token,
                    label.token,
                    tex_num(Some(weight), 0),
                    tex_num(Some(group.combined_af), 3),
                    tex_num(Some(group.tension), 1)
                )));
            }
        } else {
            out.push(subheading(&format!(
                "Tension in Each Second Sling Leg ({eff} Secondary Legs Total):"
            )));
            out.push(equation(&format!(
                "\\text{{Tension}}_2 = \\frac{{\\text{{Load Weight}}}}\
                 {{\\text{{Sling Legs}}_1 \\times \\text{{Sling Legs}}_2 \\times \
                 \\text{{AF}}_{{Combined}}}} = \
                 \\frac{{{}}}{{2 \\times {secondary_legs} \\times {}}} = {}\\ \\text{{kg}}",
                tex_num(Some(weight), 0),
                tex_num(Some(secondary_combined), 3),
                tex_num(Some(secondary_tension), 1)
            )));
        }
    } else {
        // The 4-leg reference document states the ideal-distribution
        // assumption instead of counting the effective legs.
        let tension_heading = if key == "four_leg_direct" {
            format!("Tension in Each Sling Leg (Assuming ideal distribution across {eff} Legs):")
        } else if spec.bridle {
            format!("Tension in Each Sling Leg ({eff} Effective Legs Total):")
        } else {
            "Tension in Each Effective Legs:".to_string()
        };
        out.push(subheading(&tension_heading));
        out.push(equation(&format!(
            "\\text{{Tension}} = \\frac{{\\text{{Load Weight}}}}\
             {{\\text{{Effective Legs}} \\times \\text{{AF}}}} = \
             \\frac{{{}}}{{{eff} \\times {}}} = {}\\ \\text{{kg}}",
            tex_num(Some(weight), 0),
            tex_num(Some(af), 3),
            tex_num(Some(tension), 1)
        )));
    }
    out.join("")
}

// ---------------------------------------------------------------------------
// Section 4 -- final capacity and equipment selection
// ---------------------------------------------------------------------------

/// The nested documents number the master and secondary slings 1 and 2.
fn label_index(label: &str) -> &'static str {
    match label {
        "First" => "_1",
        "Second" => "_2",
        _ => "",
    }
}

/// The "Sling Leg Capacity Each" block for one installed tag.
fn tag_capacity_block(
    tag_wll: f64,
    rating_legs: i64,
    tag_angle_factor: f64,
    leg_capacity: Option<f64>,
    label: &str,
) -> String {
    let suffix = if !label.is_empty() {
        format!(" for {label} Sling")
    } else {
        String::new()
    };
    let index = label_index(label);
    let name = if !label.is_empty() {
        format!("{label} Sling Leg Capacity Per")
    } else {
        "Sling Leg Capacity Per".to_string()
    };
    let mut out = vec![subheading(&format!(
        "Sling Leg Capacity Each{suffix} (Tag Rating):"
    ))];
    let Some(leg_capacity) = leg_capacity else {
        out.push(p(
            &format!(
                "<b>Installed tag WLL:</b> Not entered \u{2014} enter the sling tag rating \
                 and its rated angle to check the {rating_legs}-leg sling leg capacity."
            ),
            "",
        ));
        return out.join("");
    };
    out.push(equation(&format!(
        "\\text{{{name}}} = \
         \\frac{{\\text{{Installed Tag WLL}}{index}}}\
         {{\\text{{Sling Legs}}{index} \\times \\cos(\\text{{Tag WLL Angle}}{index})}} = \
         \\frac{{{}}}{{{rating_legs} \\times {}}} = {}\\ \\text{{kg}}",
        tex_num(Some(tag_wll), 0),
        tex_num(Some(tag_angle_factor), 3),
        tex_num(Some(leg_capacity), 1)
    )));
    out.join("")
}

/// The "WLL Utilisation" block.
fn utilisation_block(
    required: f64,
    capacity_label: &str,
    capacity: Option<f64>,
    utilisation: Option<f64>,
    index: &str,
    check: &str,
    for_sling: &str,
    needs_angle: bool,
) -> String {
    let prefix = if !for_sling.is_empty() {
        format!(" for {for_sling} Sling")
    } else {
        String::new()
    };
    let mut out = vec![subheading(&format!(
        "WLL Utilisation{prefix} (Tag Rating):"
    ))];
    let (Some(capacity), Some(utilisation)) = (capacity, utilisation) else {
        out.push(p(
            &format!(
                "Not calculated \u{2014} enter the sling tag rating{} \
                 to complete the equipment check.",
                if needs_angle { " and its rated angle" } else { "" }
            ),
            "",
        ));
        return out.join("");
    };
    out.push(equation(&format!(
        "\\text{{WLL Utilisation}}{index} = \
         \\frac{{\\text{{Minimum WLL}}{index}}}{{{capacity_label}}} \\times 100\\% = \
         \\frac{{{}}}{{{}}} \\times 100\\% = {:.2}\\%",
        tex_num(Some(required), 1),
        tex_num(Some(capacity), 1),
        utilisation
    )));
    let pass = utilisation <= 100.0;
    let class = if pass { "pass" } else { "fail" };
    let word = if pass { "PASS" } else { "FAIL" };
    out.push(p(
        &format!(
            "<b>Tag rating check{check}:</b> <span class='{class}'>{word}</span> \u{2014} the \
             sling leg is {utilisation:.2}% utilised."
        ),
        "",
    ));
    out.join("")
}

/// Section 4: what the sling has to be rated at, and whether the tag meets it.
pub fn capacity_section(result: &SlingResult, spec: &HitchSpec) -> String {
    let key = spec.key;
    let weight = result.load_weight;
    let af = result.angle_factor;
    let rf = result.base_reeve_factor;
    let combined = result.combined_reeve_factor;
    let required = result.required_wll;
    let utilisation = result.utilisation;
    let capacity = result.leg_capacity;
    let rating_legs = result.rating_legs;
    let nested = result.nested != 0;
    let mut out = vec![heading(4, "Final Capacity & Equipment Selection")];

    // The multi-leg reference documents open section 4 with the installed tag
    // rating.
    if spec.leg_count > 1 {
        if nested {
            out.push(tag_capacity_block(
                result.tag_wll,
                result.rating_legs,
                result.tag_angle_factor,
                result.leg_capacity,
                "First",
            ));
            out.push(tag_capacity_block(
                result.secondary_tag_wll,
                result.secondary_rating_legs.unwrap_or(0),
                result.secondary_tag_angle_factor,
                result.secondary_leg_capacity,
                "Second",
            ));
        } else {
            out.push(tag_capacity_block(
                result.tag_wll,
                result.rating_legs,
                result.tag_angle_factor,
                result.leg_capacity,
                "",
            ));
        }
    }

    if nested {
        let secondary_required = result.secondary_required_wll.unwrap_or(0.0);
        let secondary_combined = result.secondary_combined_af.unwrap_or(1.0);
        let secondary_capacity = result.secondary_leg_capacity;
        let secondary_legs = result.secondary_legs.unwrap_or(1);
        out.push(subheading("Minimum Required Sling WLL for First Sling (Tag Rating):"));
        out.push(equation(&format!(
            "\\text{{Minimum WLL}}_1 = \\frac{{\\text{{Load Weight}}}}\
             {{\\text{{Sling Legs}}_1 \\times \\text{{RF}} \\times \\text{{AF}}_{{Long}}}} = \
             \\frac{{{}}}{{{rating_legs} \\times {} \\times {}}} = {}\\ \\text{{kg}}",
            tex_num(Some(weight), 0),
            tex_num(Some(rf), 2),
            tex_num(Some(af), 3),
            tex_num(Some(required), 1)
        )));
        out.push(utilisation_block(
            required,
            "\\text{First Sling Leg Capacity Per}",
            capacity,
            utilisation,
            "_1",
            " (first sling)",
            "First",
            true,
        ));
        out.push(subheading("Minimum Required Sling WLL for Second Sling (Tag Rating):"));
        out.push(equation(&format!(
            "\\text{{Minimum WLL}}_2 = \\frac{{\\text{{Load Weight}}}}\
             {{\\text{{Sling Legs}}_1 \\times \\text{{Sling Legs}}_2 \\times \
             \\text{{RF}} \\times \\text{{AF}}_{{Combined}}}} = \
             \\frac{{{}}}{{{rating_legs} \\times {secondary_legs} \\times {} \\times {}}} = \
             {}\\ \\text{{kg}}",
            tex_num(Some(weight), 0),
            tex_num(Some(rf), 2),
            tex_num(Some(secondary_combined), 3),
            tex_num(Some(secondary_required), 1)
        )));
        out.push(utilisation_block(
            secondary_required,
            "\\text{Second Sling Leg Capacity Per}",
            secondary_capacity,
            result.secondary_utilisation,
            "_2",
            " (second sling)",
            "Second",
            true,
        ));
    } else {
        // The single-sling documents keep the leg count in this heading.
        let required_heading = if spec.leg_count > 1 {
            format!("Minimum Required Sling WLL (Tag Rating for Sling Legs = {rating_legs}):")
        } else {
            "Minimum Required Sling WLL (Tag Rating):".to_string()
        };
        out.push(subheading(&required_heading));
        if CHOKE_KEYS.contains(&key) {
            out.push(equation(&format!(
                "\\text{{Minimum WLL}} = \\frac{{\\text{{Load Weight}}}}\
                 {{\\text{{Sling Legs}} \\times \\text{{Combined RF}} \\times \\text{{AF}}}} = \
                 \\frac{{{}}}{{{rating_legs} \\times {} \\times {}}} = {}\\ \\text{{kg}}",
                tex_num(Some(weight), 0),
                tex_num(Some(combined), 4),
                tex_num(Some(af), 3),
                tex_num(Some(required), 1)
            )));
        } else {
            out.push(equation(&format!(
                "\\text{{Minimum WLL}} = \\frac{{\\text{{Load Weight}}}}\
                 {{\\text{{Sling Legs}} \\times \\text{{RF}} \\times \\text{{AF}}}} = \
                 \\frac{{{}}}{{{rating_legs} \\times {} \\times {}}} = {}\\ \\text{{kg}}",
                tex_num(Some(weight), 0),
                tex_num(Some(rf), 2),
                tex_num(Some(af), 3),
                tex_num(Some(required), 1)
            )));
        }
        if spec.leg_count > 1 {
            out.push(utilisation_block(
                required,
                "\\text{Sling Leg Capacity Per}",
                capacity,
                utilisation,
                "",
                "",
                "",
                true,
            ));
        } else {
            // A single leg hangs vertical, so there is no derate to show.
            out.push(utilisation_block(
                required,
                "\\text{Installed tag WLL}",
                capacity,
                utilisation,
                "",
                "",
                "",
                false,
            ));
        }
    }

    out.push(subheading("Configured Setup SWL:"));
    out.push(equation(&format!(
        "\\mathbf{{\\text{{Configured Setup SWL}}}} = \\text{{Load Weight}} = \
         {}\\ \\text{{kg}}",
        tex_num(Some(weight), 0)
    )));
    out.join("")
}

// ---------------------------------------------------------------------------
// Section 5 -- notes
// ---------------------------------------------------------------------------

/// Whether section 5 applies.
pub fn shows_notes(spec: &HitchSpec) -> bool {
    CHOKE_KEYS.contains(&spec.key) || spec.leg_count > 1
}

/// Section 5: the choke reduction table and the tag-rating leg rule.
pub fn notes_section(result: &SlingResult, spec: &HitchSpec) -> String {
    let mut out = vec![
        heading(5, "Note"),
        subheading("Choke Notes"),
    ];
    if !CHOKE_KEYS.contains(&spec.key) {
        out.push(p(
            "No choke reduction applies to the selected arrangement.",
            "",
        ));
    } else {
        let reduction = result.choke_reduction_factor;
        let external = result.external_angle;
        out.push(p(
            "(1) Percent of sling rated capacity in a choker hitch. The rating \
             reduction is applied to the tag WLL, not to the resolved \
             main-leg tension.",
            "",
        ));
        out.push(
            "<table><thead><tr><th>External choke angle</th>\
             <th>Capacity multiplier</th></tr></thead><tbody>"
                .to_string(),
        );
        out.push("<tr><td>120\u{00b0} and over</td><td>1.00</td></tr>".to_string());
        out.push("<tr><td>90\u{00b0} to under 120\u{00b0}</td><td>0.87</td></tr>".to_string());
        out.push("<tr><td>60\u{00b0} to under 90\u{00b0}</td><td>0.74</td></tr>".to_string());
        out.push("<tr><td>30\u{00b0} to under 60\u{00b0}</td><td>0.62</td></tr>".to_string());
        out.push("<tr><td>Below 30\u{00b0}</td><td>0.49</td></tr>".to_string());
        out.push("</tbody></table>".to_string());
        out.push(p(
            &format!(
                "Selected external angle: {} \u{2014} reduction factor {:.2}.",
                fmt(external, 0, "\u{00b0}"),
                reduction
            ),
            "",
        ));
    }
    if spec.leg_count > 1 {
        out.push(subheading(
            "To Calculate Sling Leg Capacity Each (Tag Rating):",
        ));
        out.extend(RATING_LEG_NOTE.iter().map(|note| p(note, "")));
        if spec.nested != 0 {
            out.push(nested_rating_note(result, spec));
        }
    }
    out.join("")
}

/// The note line naming the rated leg count of a nested system.
fn nested_rating_note(result: &SlingResult, spec: &HitchSpec) -> String {
    let first = result.rating_legs;
    let Some(second) = result.secondary_rating_legs else {
        return String::new();
    };
    p(
        &format!(
            "Applied in this report: the first sling is a {}-leg \
             sling, so the rule above gives Sling Legs = {first} for the first \
             sling's leg capacity, and the second sling is a {}-leg \
             sling, so the same rule gives Sling Legs = {second} for the second \
             sling's leg capacity.",
            spec.leg_count, spec.nested
        ),
        "",
    )
}

// ---------------------------------------------------------------------------
// Shared with the nonuniform and tandem blocks
// ---------------------------------------------------------------------------

/// The installed tag's leg capacity and the utilisation that divides by it.
///
/// One block for the two asymmetric slings *and* the two tandem cranes.
/// Returns an empty list when no capacity was entered.
pub fn installed_tag_block(
    tag_wll: f64,
    rated_legs: i64,
    tag_angle_factor: f64,
    leg_capacity: Option<f64>,
    required_wll: f64,
    utilisation: Option<f64>,
    label: &str,
) -> Vec<String> {
    let Some(leg_capacity) = leg_capacity else {
        return Vec::new();
    };
    let capacity_line = if rated_legs == SINGLE_LEG {
        // A single leg is a definition rather than a derivation: it hangs
        // vertical, so there is no leg count to share with and no angle to
        // derate it by.
        equation(&format!(
            "\\text{{Capacity}}_{{{label}}} = \\text{{Tag WLL}}_{{{label}}} = \
             {}\\ \\text{{kg}}",
            tex_num(Some(tag_wll), 1)
        ))
    } else {
        equation(&format!(
            // Without this the utilisation's denominator appeared from nowhere.
            "\\text{{Capacity}}_{{{label}}} = \
             \\frac{{\\text{{Tag WLL}}_{{{label}}}}}\
             {{\\text{{Sling Legs}}_{{{label}}} \\times \\cos(\\text{{Tag Angle}}_{{{label}}})}} = \
             \\frac{{{}}}{{{rated_legs} \\times {}}} = {}\\ \\text{{kg}}",
            tex_num(Some(tag_wll), 1),
            tex_num(Some(tag_angle_factor), 3),
            tex_num(Some(leg_capacity), 1)
        ))
    };
    vec![
        capacity_line,
        equation(&format!(
            // "Sling Leg Capacity", not "Tag WLL": the number divided by is the
            // *derated* capacity, not the tag's rating.
            "\\text{{Utilisation}}_{{{label}}} = \
             \\frac{{\\text{{Min WLL}}_{{{label}}}}}\
             {{\\text{{Sling Leg Capacity}}_{{{label}}}}} \\times 100\\% = \
             \\frac{{{}}}{{{}}} \\times 100\\% = {}",
            tex_num(Some(required_wll), 1),
            tex_num(Some(leg_capacity), 1),
            tex_num(utilisation, 2)
        )),
    ]
}

/// State the leg count each tag rating is declared on.
pub fn nonuniform_rating_legs_note(pairs: &[(String, Option<i64>, Option<i64>)]) -> String {
    let mut applied: Vec<String> = Vec::new();
    let mut rule_used = false;
    for (caption, chosen_raw, rated_raw) in pairs {
        let (Some(chosen), Some(rated)) = (chosen_raw, rated_raw) else {
            continue;
        };
        if *chosen == SINGLE_LEG {
            continue;
        }
        rule_used = rule_used || *chosen != *rated;
        applied.push(format!(
            "{caption}'s tag is a {chosen}-leg sling, so the rule gives \
             Sling Legs = {rated} for its leg capacity."
        ));
    }
    if applied.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    if rule_used {
        out.push_str(&subheading(
            "To Calculate Sling Leg Capacity Each (Tag Rating):",
        ));
        for note in RATING_LEG_NOTE {
            out.push_str(&p(note, ""));
        }
    }
    out.push_str(&p(
        &format!("Applied in this report: {}", applied.join(" ")),
        "",
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{LiftTotals, LoadItem};
    use crate::report::{build_report_html, ReportInputs};
    use crate::sling::{hitch_spec, solve_one_leg, OneLegInputs};

    fn solve(inputs: &OneLegInputs) -> SlingResult {
        solve_one_leg(inputs, 2000.0).unwrap()
    }

    fn sections(inputs: &OneLegInputs) -> (String, String, String) {
        let result = solve(inputs);
        let spec = hitch_spec(&result.hitch).unwrap();
        (
            factors_section(&result, spec),
            capacity_section(&result, spec),
            notes_section(&result, spec),
        )
    }

    fn report_with(inputs: &OneLegInputs) -> String {
        let result = solve(inputs);
        let report_inputs = ReportInputs {
            totals: LiftTotals::of(
                &[LoadItem {
                    weight_kg: 2000.0,
                    ..LoadItem::default()
                }],
                &[],
            ),
            sling: Some(result),
            ..ReportInputs::default()
        };
        build_report_html(&report_inputs, &["uniform".to_string()])
    }

    fn direct_hook() -> OneLegInputs {
        OneLegInputs {
            hitch: "direct_hook".to_string(),
            sling_length: 7.0,
            tag_wll: 5000.0,
            ..OneLegInputs::default()
        }
    }

    fn two_leg_direct() -> OneLegInputs {
        OneLegInputs {
            hitch: "two_leg_direct".to_string(),
            sling_length: 7.0,
            pick_distance: 5.0,
            tag_wll: 5300.0,
            tag_wll_angle: 45.0,
            ..OneLegInputs::default()
        }
    }

    fn round_choke() -> OneLegInputs {
        OneLegInputs {
            hitch: "round_choke".to_string(),
            sling_length: 7.0,
            diameter: 1.0,
            choke_angle: 60.0,
            tag_wll: 5000.0,
            ..OneLegInputs::default()
        }
    }

    fn nested3() -> OneLegInputs {
        OneLegInputs {
            hitch: "two_leg_nested_3leg".to_string(),
            sling_length: 7.0,
            pick_distance: 5.4,
            secondary_length: 0.6,
            secondary_spread: 0.2,
            tag_wll: 5000.0,
            secondary_tag_wll: 3000.0,
            ..OneLegInputs::default()
        }
    }

    #[test]
    fn the_single_leg_factors_match_the_reference() {
        let factors = sections(&direct_hook()).0;
        for expected in [
            "<h2>3. RIGGING FACTORS &amp; TENSION CALCULATIONS</h2>",
            "\\[\\text{AF} = \\cos(0.00^\\circ) = 1.000\\]",
            "\\[\\text{RF}_{Base} = 1.00\\]",
            "Tension in Each Effective Legs:",
            "\\[\\text{Tension} = \\frac{\\text{Load Weight}}{\\text{Effective Legs} \\times \\text{AF}} = \\frac{2{,}000}{1 \\times 1.000} = 2{,}000.0\\ \\text{kg}\\]",
        ] {
            assert!(factors.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_choke_factor_table_and_its_notes_match_the_reference() {
        let (factors, capacity, notes) = sections(&round_choke());
        for expected in [
            "\\[\\text{AF} = \\cos(0^\\circ) = 1.0\\]",
            "The main lifting leg stands perfectly vertical (0\u{00b0}).",
            "\\[\\text{RF}_{Base} = 0.75\\]",
            "\\[\\text{Reduction Factor} = 1.00\\]",
            "\\[\\text{Combined RF} = \\text{RF}_{Base} \\times \\text{Reduction Factor} = 0.75 \\times 1.00 = 0.7500\\]",
        ] {
            assert!(factors.contains(expected), "missing {expected}");
        }
        assert!(capacity.contains(
            "\\[\\text{Minimum WLL} = \\frac{\\text{Load Weight}}{\\text{Sling Legs} \\times \\text{Combined RF} \\times \\text{AF}} = \\frac{2{,}000}{1 \\times 0.7500 \\times 1.000} = 2{,}666.7\\ \\text{kg}\\]"
        ));
        for expected in [
            "<h2>5. NOTE</h2>",
            "Choke Notes",
            "(1) Percent of sling rated capacity in a choker hitch. The rating reduction is applied to the tag WLL, not to the resolved main-leg tension.",
            "Selected external angle: 120 \u{00b0} \u{2014} reduction factor 1.00.",
            "<tr><td>120\u{00b0} and over</td><td>1.00</td></tr>",
            "<tr><td>90\u{00b0} to under 120\u{00b0}</td><td>0.87</td></tr>",
            "<tr><td>60\u{00b0} to under 90\u{00b0}</td><td>0.74</td></tr>",
            "<tr><td>30\u{00b0} to under 60\u{00b0}</td><td>0.62</td></tr>",
            "<tr><td>Below 30\u{00b0}</td><td>0.49</td></tr>",
        ] {
            assert!(notes.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_nested_factors_publish_one_row_per_leg_group() {
        let factors = sections(&nested3()).0;
        for expected in [
            "\\[\\text{AF}_{Long} = \\cos(20.91^\\circ) = 0.934\\]",
            "Transverse Sling Angle Factor for Outer Legs (AF Trans Outer):",
            "\\[\\text{AF}_{\\text{Trans Outer}} = \\cos(19.47^\\circ) = 0.943\\]",
            "Combined Sling Angle Factor (AF) for Outer Secondary Legs:",
            "\\[\\text{AF}_{\\text{Combined Outer}} = \\text{AF}_{Long} \\times \\text{AF}_{\\text{Trans Outer}} = 0.934 \\times 0.943 = 0.881\\]",
            "Transverse Sling Angle Factor for Mid Legs (AF Trans Mid):",
            "Combined Sling Angle Factor (AF) for Middle Secondary Legs:",
            "Tension in the Sloped Outer Secondary Legs:",
            "\\[\\text{Tension}_{\\text{Secondary Outer}} = \\frac{\\text{Load Weight}}{\\text{Sling Legs}_1 \\times \\text{Sling Legs}_2 \\times \\text{AF}_{\\text{Combined Outer}}} = \\frac{2{,}000}{2 \\times 3 \\times 0.881} = 378.4\\ \\text{kg}\\]",
        ] {
            assert!(factors.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn a_single_leg_prints_no_tag_capacity_block() {
        let capacity = sections(&direct_hook()).1;
        assert!(capacity.contains("<h2>4. FINAL CAPACITY &amp; EQUIPMENT SELECTION</h2>"));
        assert!(!capacity.contains("Sling Leg Capacity Each"));
        assert!(capacity.contains(
            "\\[\\text{WLL Utilisation} = \\frac{\\text{Minimum WLL}}{\\text{Installed tag WLL}} \\times 100\\% = \\frac{2{,}000.0}{5{,}000.0} \\times 100\\% = 40.00\\%\\]"
        ));
        assert!(capacity.contains(
            "<b>Tag rating check:</b> <span class='pass'>PASS</span> \u{2014} the sling leg is 40.00% utilised."
        ));
    }

    #[test]
    fn the_multi_leg_capacity_block_derives_its_denominator() {
        let capacity = sections(&two_leg_direct()).1;
        for expected in [
            "Sling Leg Capacity Each (Tag Rating):",
            "\\[\\text{Sling Leg Capacity Per} = \\frac{\\text{Installed Tag WLL}}{\\text{Sling Legs} \\times \\cos(\\text{Tag WLL Angle})} = \\frac{5{,}300}{2 \\times 0.707} = 3{,}748.2\\ \\text{kg}\\]",
            "Minimum Required Sling WLL (Tag Rating for Sling Legs = 2):",
            "\\[\\text{WLL Utilisation} = \\frac{\\text{Minimum WLL}}{\\text{Sling Leg Capacity Per}} \\times 100\\% = \\frac{1{,}070.7}{3{,}748.2} \\times 100\\% = 28.57\\%\\]",
        ] {
            assert!(capacity.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn a_nested_lift_names_both_slings_throughout() {
        let (_, capacity, notes) = sections(&nested3());
        for expected in [
            "Sling Leg Capacity Each for First Sling (Tag Rating):",
            "Sling Leg Capacity Each for Second Sling (Tag Rating):",
            "\\[\\text{First Sling Leg Capacity Per} = \\frac{\\text{Installed Tag WLL}_1}{\\text{Sling Legs}_1 \\times \\cos(\\text{Tag WLL Angle}_1)} = \\frac{5{,}000}{2 \\times 0.707} = 3{,}536.1\\ \\text{kg}\\]",
            "\\[\\text{Second Sling Leg Capacity Per} = \\frac{\\text{Installed Tag WLL}_2}{\\text{Sling Legs}_2 \\times \\cos(\\text{Tag WLL Angle}_2)} = \\frac{3{,}000}{2 \\times 0.707} = 2{,}121.6\\ \\text{kg}\\]",
            "<b>Tag rating check (first sling):</b> <span class='pass'>PASS</span>",
            "<b>Tag rating check (second sling):</b> <span class='pass'>PASS</span>",
        ] {
            assert!(capacity.contains(expected), "missing {expected}");
        }
        assert!(notes.contains(
            "Applied in this report: the first sling is a 2-leg sling, so the rule above gives Sling Legs = 2 for the first sling's leg capacity, and the second sling is a 3-leg sling, so the same rule gives Sling Legs = 2 for the second sling's leg capacity."
        ));
    }

    #[test]
    fn every_arrangement_carries_the_tag_rating_rule_when_it_has_more_than_one_leg() {
        let notes = sections(&two_leg_direct()).2;
        for expected in [
            "No choke reduction applies to the selected arrangement.",
            "To Calculate Sling Leg Capacity Each (Tag Rating):",
            "If 4-Leg Sling, use Sling Legs = 3",
            "If 3-Leg Sling, use Sling Legs = 2",
            "If 2-Leg Sling, use Sling Legs = 2",
        ] {
            assert!(notes.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn section_five_is_shown_exactly_for_the_arrangements_that_have_one() {
        let without: Vec<&str> = crate::sling::HITCH_SPECS
            .iter()
            .filter(|spec| !shows_notes(spec))
            .map(|spec| spec.key)
            .collect();
        assert_eq!(
            without,
            vec![
                "direct_hook",
                "round_basket",
                "rect_basket",
                "vertical_round_basket",
                "vertical_rect_basket"
            ]
        );
    }

    #[test]
    fn the_whole_sling_block_numbers_its_sections_in_order() {
        let html = report_with(&two_leg_direct());
        for number in 1..=5 {
            assert!(html.contains(&format!("<h2>{number}. ")), "missing section {number}");
        }
        assert_eq!(count_h2(&html), 5);
        assert!(html.contains("<h2>2. GEOMETRIC &amp; HEADROOM CALCULATIONS</h2>"));
        assert!(html.contains("<h2>5. NOTE</h2>"));
    }

    #[test]
    fn a_half_filled_form_contributes_no_sling_section_at_all() {
        let html = build_report_html(
            &ReportInputs {
                totals: LiftTotals::of(
                    &[LoadItem {
                        weight_kg: 2000.0,
                        ..LoadItem::default()
                    }],
                    &[],
                ),
                sling: None,
                ..ReportInputs::default()
            },
            &["uniform".to_string()],
        );
        assert!(!html.contains("LIFT INPUT PARAMETERS"));
        assert!(html.contains("<h1>Lifting Plan Calculation</h1>"));
    }

    fn count_h2(html: &str) -> usize {
        let bytes = html.as_bytes();
        let mut count = 0;
        let mut i = 0;
        while i + 4 <= bytes.len() {
            if &bytes[i..i + 4] == b"<h2>" {
                let start = i + 4;
                let mut j = start;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j > start && j < bytes.len() && bytes[j] == b'.' {
                    count += 1;
                }
                i = j.max(i + 4);
            } else {
                i += 1;
            }
        }
        count
    }
}
