//! The nonuniform-lift report blocks, ported from `ReportNonuniform.kt`.
//!
//! Kept apart from the uniform sections because a nonuniform lift reports
//! **every figure per leg**: the uniform tab's single `tension_each` and
//! `required_wll` have no counterpart here.

use crate::calc::{crane_usage_band, UsageState};
use crate::nonuniform::{governing_case_words, NonuniformResult, TandemResult};
use crate::report::{
    equation, escape, fmt, heading, label_or_not_entered, label_value, p, subheading, table,
    tex_num,
};

/// How a governing case reads in the report's prose.
///
/// Deliberately **not** the same words the Tandem tab uses: the tab names the
/// case that sized a crane ("level baseline"); the report names the *angle* the
/// case was solved at ("0° level baseline").
fn report_case_words(case: Option<&str>) -> String {
    match case {
        Some("level") => "0\u{00b0} level baseline".to_string(),
        Some("tilt") => "declared tilt".to_string(),
        Some("vertical") => "90\u{00b0} vertical rotation".to_string(),
        other => other.unwrap_or("").to_string(),
    }
}

/// The crane carrying the larger of the two governing WLL figures.
fn governing_crane(result: &TandemResult) -> String {
    if result.required_wll_a >= result.required_wll_b {
        "1".to_string()
    } else {
        "2".to_string()
    }
}

/// One nonuniform block, or the reason there is none.
pub fn nonuniform_sections(
    result: Option<&NonuniformResult>,
    first_number: usize,
    title: &str,
    error: Option<&str>,
) -> Vec<String> {
    let heading_line = heading(first_number, title);
    if let Some(error) = error {
        return vec![
            heading_line,
            p(
                &format!(
                    "{error} Complete the {title} tab to include it in this plan."
                ),
                "",
            ),
        ];
    }
    let Some(result) = result else {
        return vec![
            heading_line,
            p(
                &format!(
                    "No {} has been set up. Complete the {title} tab \
                     to include it in this plan.",
                    title.to_lowercase()
                ),
                "",
            ),
        ];
    };
    // A solved scenario is not by itself evidence that a lift was set up: the
    // forms seed themselves with the documents' worked examples. The hook load
    // arriving from the Overall Weight tab is what marks it as real.
    if result.load_weight() <= 0.0 {
        return vec![
            heading_line,
            p(
                &format!(
                    "No {} has been set up. Enter a load weight on \
                     the Overall Weight tab and complete the {title} tab to include \
                     it in this plan.",
                    title.to_lowercase()
                ),
                "",
            ),
        ];
    }
    match result {
        NonuniformResult::Asym(asym) => asym_sections(asym, first_number, title),
        NonuniformResult::Tandem(tandem) => tandem_sections(tandem, first_number, title),
    }
}

/// The two-leg asymmetric bridle with a shortened leg.
///
/// Every step the reference document publishes is reproduced with its own
/// heading, because the report is the evidence a reviewer reads.
pub fn asym_sections(
    result: &crate::nonuniform::Asym2LegResult,
    first_number: usize,
    title: &str,
) -> Vec<String> {
    let mut out = vec![heading(first_number, title)];
    let step = result.pick_point2_height;
    out.push(subheading("Lift Input Parameters"));
    out.push(table(&[
        (
            "Load weight".to_string(),
            fmt(Some(result.load_weight), 2, "kg"),
        ),
        (
            "Lift configuration".to_string(),
            label_or_not_entered(Some(&result.scenario_label)),
        ),
        (
            "Standard sling length available (both legs)".to_string(),
            fmt(Some(result.sling_length), 3, "m"),
        ),
        (
            "Distance between pick points (total span)".to_string(),
            fmt(Some(result.pick_distance), 3, "m"),
        ),
        (
            "COG horizontal position from pick point 1".to_string(),
            fmt(Some(result.cog_from_pick1), 3, "m"),
        ),
        (
            "Vertical height difference between pick points".to_string(),
            fmt(Some(step), 3, "m")
                + if step > 0.0 {
                    " (pick point 2 higher)"
                } else if step < 0.0 {
                    " (pick point 2 lower)"
                } else {
                    " (level)"
                },
        ),
        (
            "Base reeve factor (RF)".to_string(),
            fmt(Some(result.base_reeve_factor), 2, ""),
        ),
    ]));

    let long_leg = result.long_leg;
    let short_leg = result.short_leg;
    let span1 = result.span1;
    let span2 = result.span2;
    let total_span = result.pick_distance;
    let standard = result.sling_length;
    let headroom = result.headroom;
    let short_length = result.short_leg_length;
    let drop1 = result.drop1;
    let drop2 = result.drop2;
    let long_leg_name = format!("Sling {long_leg}");
    let short_leg_name = format!("Sling {short_leg}");
    let long_span = span1.max(span2);

    out.push(heading(
        first_number + 1,
        "Geometric & Headroom Calculations (Shortening Method with Elevation Delta)",
    ));
    let step_prose = if step > 0.0 {
        format!(
            "The pick points are not level: pick point 2 is {} \
             higher than pick point 1. That step is not cosmetic. The long leg \
             sets the hook's height above its own pick point, and the drop to \
             the far pick point follows from the step, so raising it shortens \
             that drop and lengthens the leg the operator has to shorten. The \
             document states the rule as Y2 = Y1 - Height Difference; a \
             positive entry means pick point 2 is higher and a negative one \
             means it is lower.",
            fmt(Some(step), 3, "m")
        )
    } else if step < 0.0 {
        format!(
            "Pick point 2 sits {} below pick point 1, so the \
             drop from the hook down to it is the longer of the two and the \
             leg spanning to it needs less shortening. The document's rule is \
             the same one, Y2 = Y1 - Height Difference, entered with a \
             negative value to say 'lower' rather than 'level'.",
            fmt(Some(-step), 3, "m")
        )
    } else {
        "Both pick points are level, so the two legs share one headroom and \
         the shorter span is set purely by the off-centre COG. Enter a \
         non-zero height difference to handle a step between the pick \
         points; the rule is Y2 = Y1 - Height Difference."
            .to_string()
    };
    out.push(p(
        &format!(
            "Operational rule: to make an asymmetrical lift without tilting the load, \
             the crane hook must sit vertically above the centre of gravity. \
             Because the COG is off-centre, one side of the horizontal span is \
             shorter than the other. Both legs start at the same standard length, \
             and the longer span keeps that full length and so fixes the hook's \
             height. {step_prose}"
        ),
        "",
    ));
    out.push(subheading("Declared Horizontal Spans:"));
    out.push(label_value("Left span (X1)", &fmt(Some(span1), 3, "m")));
    out.push(equation(&format!(
        "\\text{{X}}_2 = \\text{{Total Span}} - \\text{{X}}_1 = \
         {} - {} = {}\\ \\text{{m}}",
        tex_num(Some(total_span), 3),
        tex_num(Some(span1), 3),
        tex_num(Some(span2), 3)
    )));
    out.push(subheading(&format!(
        "Required Length for the Long Leg ({long_leg_name}, Stays at Full Length):"
    )));
    out.push(equation(&format!(
        "\\text{{Length}}_{{{long_leg}}} = \\text{{Standard Length}} = {}\\ \\text{{m}}",
        tex_num(Some(standard), 3)
    )));
    // Whichever pick point the long leg reaches, that leg's drop comes from the
    // Pythagoras; the other pick point's drop follows from the step.
    let long_is1 = long_leg == 1;
    let span_long = if long_is1 { span1 } else { span2 };
    let span_far = if long_is1 { span2 } else { span1 };
    let drop_long = headroom;
    let drop_far = if long_is1 { drop2 } else { drop1 };
    // The step is printed as a magnitude, with the direction carried by the
    // operator, so no line ever reads "6.062 - -0.500".
    let step_sign = if drop_far < drop_long { "-" } else { "+" };
    let step_mag = tex_num(Some(step.abs()), 3);
    out.push(subheading(&format!(
        "Required Vertical Height from the Hook to Pick Point {long_leg} Level (Y{long_leg}):"
    )));
    out.push(equation(&format!(
        "\\text{{Y}}_{{{long_leg}}} = \\sqrt{{\\text{{Length}}^2 - \\text{{X}}_{{{long_leg}}}^2}} = \
         \\sqrt{{{}^2 - {}^2}} = {}\\ \\text{{m}}",
        tex_num(Some(standard), 3),
        tex_num(Some(span_long), 3),
        tex_num(Some(drop_long), 3)
    )));
    out.push(subheading(&format!(
        "Required Vertical Height from the Hook to Pick Point {short_leg} Level (Y{short_leg}):"
    )));
    out.push(equation(&format!(
        "\\text{{Y}}_{{{short_leg}}} = \\text{{Y}}_{{{long_leg}}} {step_sign} \
         \\text{{Height Difference}} = {} {step_sign} \
         {step_mag} = {}\\ \\text{{m}}",
        tex_num(Some(drop_long), 3),
        tex_num(Some(drop_far), 3)
    )));
    out.push(subheading(&format!(
        "Required Length for the Short Leg ({short_leg_name}, Adjusted Leg):"
    )));
    out.push(equation(&format!(
        "\\text{{Length}}_{{{short_leg}}} = \\sqrt{{\\text{{Y}}_{{{short_leg}}}^2 + \
         \\text{{X}}_{{{short_leg}}}^2}} = \\sqrt{{{}^2 + {}^2}} = {}\\ \\text{{m}}",
        tex_num(Some(drop_far), 3),
        tex_num(Some(span_far), 3),
        tex_num(Some(short_length), 3)
    )));
    if result.needs_lengthening {
        out.push(subheading("Sling Lengthening Required:"));
        out.push(p(
            "The far pick point is low enough that the leg reaching it has to be \
             longer than the standard sling. A clevis shortening grab hook can \
             only take length out of a leg, so this setup cannot be made with \
             the equipment on hand: use a longer sling, or reduce the step \
             between the pick points.",
            "",
        ));
        out.push(equation(&format!(
            "\\text{{Lengthening}} = \\text{{Required Length}} - \\text{{Standard Length}} = \
             {} - {} = {}\\ \\text{{m}}",
            tex_num(Some(short_length), 3),
            tex_num(Some(standard), 3),
            tex_num(Some(result.lengthening), 3)
        )));
    } else {
        out.push(subheading("Sling Shortening Adjustment Needed:"));
        out.push(equation(&format!(
            "\\text{{Shortening}} = \\text{{Standard Length}} - \\text{{Required Length}} = \
             {} - {} = {}\\ \\text{{m}}",
            tex_num(Some(standard), 3),
            tex_num(Some(short_length), 3),
            tex_num(Some(result.shortening), 3)
        )));
    }
    out.push(subheading("Sling Leg Angle from Vertical:"));
    out.push(equation(&format!(
        "\\text{{Angle}}_{{{long_leg}}} = \\arcsin\\frac{{\\text{{X}}_{{{long_leg}}}}}\
         {{\\text{{Length}}_{{{long_leg}}}}} = \
         \\arcsin\\frac{{{}}}{{{}}} = {}",
        tex_num(Some(span1.max(span2)), 3),
        tex_num(Some(standard), 3),
        fmt(Some(if long_leg == 1 { result.angle1 } else { result.angle2 }), 2, "\u{00b0}")
    )));
    out.push(equation(&format!(
        "\\text{{Angle}}_{{{short_leg}}} = \\arcsin\\frac{{\\text{{X}}_{{{short_leg}}}}}\
         {{\\text{{Length}}_{{{short_leg}}}}} = \
         \\arcsin\\frac{{{}}}{{{}}} = {}",
        tex_num(Some(span1.min(span2)), 3),
        tex_num(Some(short_length), 3),
        fmt(Some(if short_leg == 1 { result.angle1 } else { result.angle2 }), 2, "\u{00b0}")
    )));
    out.push(table(&[
        (
            format!(
                "{long_leg_name} angle from vertical ({} m span)",
                crate::figures::grouped(long_span, 3)
            ),
            fmt(Some(if long_leg == 1 { result.angle1 } else { result.angle2 }), 2, "\u{00b0}"),
        ),
        (
            format!("{short_leg_name} angle from vertical"),
            fmt(Some(if short_leg == 1 { result.angle1 } else { result.angle2 }), 2, "\u{00b0}"),
        ),
    ]));

    out.push(heading(
        first_number + 2,
        "Rigging Factors & Tension Calculations",
    ));
    out.push(p(
        "The load share is a statics result rather than a leg count: each leg \
         carries the share set by the moment about the opposite pick point, so \
         the leg on the short side of the COG carries the larger share. The \
         angle factor then divides that share, because a leg standing further \
         from vertical carries more tension for the same share.",
        "",
    ));
    for leg in 1..=2i64 {
        out.push(subheading(&format!("Sling {leg} Angle Factor (AF{leg}):")));
        out.push(equation(&format!(
            "\\text{{AF}}_{{{leg}}} = \\cos({}) = {}",
            fmt(Some(if leg == 1 { result.angle1 } else { result.angle2 }), 2, "\u{00b0}"),
            fmt(
                Some(if leg == 1 {
                    result.angle_factor1
                } else {
                    result.angle_factor2
                }),
                3,
                ""
            )
        )));
    }
    out.push(subheading("Base Reeve Factor (RF):"));
    out.push(equation(&format!(
        "\\text{{RF}} = {}",
        tex_num(Some(result.base_reeve_factor), 2)
    )));
    out.push(subheading("Load Weight Distribution to Pick Point 1:"));
    out.push(equation(&format!(
        "\\text{{Weight}}_1 = \\text{{Load}} \\times \\frac{{\\text{{X}}_2}}\
         {{\\text{{Total Span}}}} = {} \\times \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.load_weight), 1),
        tex_num(Some(span2), 3),
        tex_num(Some(total_span), 3),
        tex_num(Some(result.share1), 1)
    )));
    out.push(subheading("Load Weight Distribution to Pick Point 2:"));
    out.push(equation(&format!(
        "\\text{{Weight}}_2 = \\text{{Load}} \\times \\frac{{\\text{{X}}_1}}\
         {{\\text{{Total Span}}}} = {} \\times \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.load_weight), 1),
        tex_num(Some(span1), 3),
        tex_num(Some(total_span), 3),
        tex_num(Some(result.share2), 1)
    )));
    out.push(subheading(&format!(
        "Tension in Sling 1 ({} Leg):",
        if long_leg == 1 { "Longer" } else { "Shorter" }
    )));
    out.push(equation(&format!(
        "\\text{{Tension}}_1 = \\frac{{\\text{{Weight}}_1}}{{\\text{{AF}}_1}} = \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.share1), 1),
        tex_num(Some(result.angle_factor1), 3),
        tex_num(Some(result.tension1), 1)
    )));
    out.push(subheading(&format!(
        "Tension in Sling 2 ({} Leg):",
        if long_leg == 2 { "Longer" } else { "Shorter" }
    )));
    out.push(equation(&format!(
        "\\text{{Tension}}_2 = \\frac{{\\text{{Weight}}_2}}{{\\text{{AF}}_2}} = \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.share2), 1),
        tex_num(Some(result.angle_factor2), 3),
        tex_num(Some(result.tension2), 1)
    )));
    out.push(table(&[
        (
            "Weight share, sling 1".to_string(),
            fmt(Some(result.share1), 1, "kg"),
        ),
        (
            "Weight share, sling 2".to_string(),
            fmt(Some(result.share2), 1, "kg"),
        ),
        (
            "Sling line tension, sling 1".to_string(),
            fmt(Some(result.tension1), 1, "kg"),
        ),
        (
            "Sling line tension, sling 2".to_string(),
            fmt(Some(result.tension2), 1, "kg"),
        ),
    ]));

    out.push(heading(
        first_number + 3,
        "Final Capacity & Equipment Selection",
    ));
    let reeve = result.base_reeve_factor;
    for leg in 1..=2i64 {
        out.push(subheading(&format!(
            "Minimum Required Sling WLL for Sling {leg} (Tag Rating):"
        )));
        out.push(equation(&format!(
            "\\text{{Min WLL}}_{{{leg}}} = \\frac{{\\text{{Weight}}_{{{leg}}}}}\
             {{\\text{{RF}} \\times \\text{{AF}}_{{{leg}}}}} = \
             \\frac{{{}}}{{{} \\times {}}} = {}\\ \\text{{kg}}",
            tex_num(
                Some(if leg == 1 { result.share1 } else { result.share2 }),
                1
            ),
            tex_num(Some(reeve), 2),
            tex_num(
                Some(if leg == 1 {
                    result.angle_factor1
                } else {
                    result.angle_factor2
                }),
                3
            ),
            tex_num(
                Some(if leg == 1 {
                    result.required_wll1
                } else {
                    result.required_wll2
                }),
                1
            )
        )));
    }
    out.push(subheading("Governing Leg:"));
    out.push(p(
        "The governing leg is the one demanding the higher rated sling. It is the \
         leg on the short side of the COG, because it carries the larger share \
         and stands closer to vertical.",
        "",
    ));
    out.push(table(&[
        (
            "Minimum required sling WLL, sling 1".to_string(),
            fmt(Some(result.required_wll1), 1, "kg"),
        ),
        (
            "Minimum required sling WLL, sling 2".to_string(),
            fmt(Some(result.required_wll2), 1, "kg"),
        ),
        (
            "Governing leg".to_string(),
            format!(
                "Sling {} \u{2014} {}",
                result.governing_leg,
                fmt(Some(result.required_wll), 1, "kg")
            ),
        ),
    ]));
    out.push(subheading("Installed Tag Rating Check:"));
    let mut checked = false;
    for leg in 1..=2i64 {
        let block = if leg == 1 {
            crate::report_capacity::installed_tag_block(
                result.tag_wll1,
                result.rated_legs1,
                result.tag_angle_factor1,
                result.leg_capacity1,
                result.required_wll1,
                result.utilisation1,
                &leg.to_string(),
            )
        } else {
            crate::report_capacity::installed_tag_block(
                result.tag_wll2,
                result.rated_legs2,
                result.tag_angle_factor2,
                result.leg_capacity2,
                result.required_wll2,
                result.utilisation2,
                &leg.to_string(),
            )
        };
        if block.is_empty() {
            continue;
        }
        checked = true;
        out.extend(block);
    }
    if checked {
        let note = crate::report_capacity::nonuniform_rating_legs_note(&[
            (
                "Sling 1".to_string(),
                Some(result.sling_legs1),
                Some(result.rated_legs1),
            ),
            (
                "Sling 2".to_string(),
                Some(result.sling_legs2),
                Some(result.rated_legs2),
            ),
        ]);
        if !note.is_empty() {
            out.push(note);
        }
        out.push(table(&[
            (
                "Sling 1 leg capacity".to_string(),
                match result.leg_capacity1 {
                    Some(capacity) => fmt(Some(capacity), 1, "kg"),
                    None => "Not entered".to_string(),
                },
            ),
            (
                "Sling 1 utilisation".to_string(),
                match result.utilisation1 {
                    Some(utilisation) => fmt(Some(utilisation), 2, "%"),
                    None => "\u{2014}".to_string(),
                },
            ),
            (
                "Sling 2 leg capacity".to_string(),
                match result.leg_capacity2 {
                    Some(capacity) => fmt(Some(capacity), 1, "kg"),
                    None => "Not entered".to_string(),
                },
            ),
            (
                "Sling 2 utilisation".to_string(),
                match result.utilisation2 {
                    Some(utilisation) => fmt(Some(utilisation), 2, "%"),
                    None => "\u{2014}".to_string(),
                },
            ),
        ]));
    } else {
        // Not the same statement as a pass.
        out.push(p(
            "Not calculated \u{2014} enter the installed tag WLL for each leg on the \
             Nonuniform Load tab to complete the equipment check.",
            "",
        ));
    }
    out.push(subheading("Configured Setup SWL:"));
    out.push(equation(&format!(
        "\\text{{Setup SWL}} = \\text{{Load Weight}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.setup_swl), 0)
    )));
    out
}

/// The two-crane tandem lift at the level and tilted cases.
pub fn tandem_sections(
    result: &TandemResult,
    first_number: usize,
    title: &str,
) -> Vec<String> {
    let tilt = result.tilt_angle;
    let tilt0 = crate::figures::grouped(tilt, 0);
    let dist_a = result.dist_a;
    let dist_b = result.dist_b;
    let span = result.span;
    let load = result.load_weight;
    let af = result.angle_factor;
    let reeve = result.base_reeve_factor;
    let angle = result.angle;
    let has_vertical = result.tension_b_vertical.is_some();

    let mut out = vec![heading(first_number, title)];
    out.push(subheading("Lift Input Parameters"));
    out.push(table(&[
        (
            "Total load weight".to_string(),
            fmt(Some(result.load_weight), 2, "kg"),
        ),
        (
            "Lift configuration".to_string(),
            label_or_not_entered(Some(&result.method_full_label)),
        ),
        (
            "Lug 2 position".to_string(),
            label_or_not_entered(Some(&result.method_lug2)),
        ),
        (
            "Crane 2 projected distance, solved as".to_string(),
            label_or_not_entered(Some(&result.method_rule)),
        ),
        (
            "Sling arrangement".to_string(),
            "Straight vertical direct hitch \u{2014} both sling legs vertical, so \
             the angle factor is 1.000"
                .to_string(),
        ),
        (
            "Initial horizontal distance, crane 1 hook to COG (D-1)".to_string(),
            fmt(Some(result.dist_a), 3, "m"),
        ),
        (
            "Initial horizontal distance, crane 2 hook to COG (D-2)".to_string(),
            fmt(Some(result.dist_b), 3, "m"),
        ),
        (
            "Total span between crane hooks (L)".to_string(),
            fmt(Some(result.span), 3, "m"),
        ),
        (
            "COG vertical depth below the lifting point baseline".to_string(),
            fmt(Some(result.cog_height), 3, "m"),
        ),
        (
            "Cargo tilt angle from horizontal".to_string(),
            crate::figures::grouped(tilt, 2) + "\u{00b0}",
        ),
        (
            "Base reeve factor (RF)".to_string(),
            fmt(Some(result.base_reeve_factor), 2, ""),
        ),
    ]));
    out.push(p(
        "The total span between the hooks is the sum of the two COG distances, \
         because the centre of gravity lies between the hooks on the level \
         baseline. The COG depth is the vertical distance from that baseline \
         down to the physical centre of gravity inside the cargo, and it is \
         what lets the tilt shift the COG sideways instead of only rotating \
         the load about it.",
        "",
    ));
    out.push(p(
        &format!(
            "The configuration decides how crane 2's distance is resolved once the \
             load leans: {} {} Maximum rotation: {} Use for: {}",
            label_or_not_entered(Some(&result.method_rule)),
            label_or_not_entered(Some(&result.method_span_behaviour)),
            label_or_not_entered(Some(&result.method_max_rotation)),
            label_or_not_entered(Some(&result.method_use_for))
        ),
        "",
    ));

    out.push(heading(
        first_number + 1,
        "Geometric & Headroom Calculations (Dynamic Tilt Method)",
    ));
    out.push(p(
        "Operational rule: the geometry is solved in order. The baseline 0° \
         level lift fixes where the load divides between the two hooks, and \
         the tilt is then applied, which shifts that division because the \
         centre of gravity sits below the baseline joining the lifting points \
         rather than in it.",
        "",
    ));
    out.push(subheading("Declared Sling Leg Angles:"));
    out.push(equation(&format!(
        "\\text{{Angle}}_1 = \\text{{Angle}}_2 = {}^\\circ\\ \\text{{from vertical}}",
        tex_num(Some(angle), 2)
    )));
    out.push(subheading(&format!(
        "New Horizontal Distance from Crane 1 to COG at {tilt0}° Tilt:"
    )));
    out.push(equation(&format!(
        "\\text{{ND}}_1 = \\text{{h}} \\times \\sin\\theta + \
         \\text{{D}}_1 \\times \\cos\\theta = \
         {} \\times \\sin({}^\\circ) + \
         {} \\times \\cos({}^\\circ) = {}\\ \\text{{m}}",
        tex_num(Some(result.cog_height), 3),
        tex_num(Some(tilt), 2),
        tex_num(Some(dist_a), 3),
        tex_num(Some(tilt), 2),
        tex_num(Some(result.dist_a_new), 3)
    )));
    out.push(subheading(&format!(
        "New Horizontal Distance from Crane 2 to COG at {tilt0}° Tilt:"
    )));
    if result.full_rotation {
        out.push(p(
            "Lug 2 is inline with the centre of gravity, so it does not travel \
             inward as the load leans; only the projection shortens it.",
            "",
        ));
        out.push(equation(&format!(
            "\\text{{ND}}_2 = \\text{{D}}_2 \\times \\cos\\theta = \
             {} \\times \\cos({}^\\circ) = {}\\ \\text{{m}}",
            tex_num(Some(dist_b), 3),
            tex_num(Some(tilt), 2),
            tex_num(Some(result.dist_b_new), 3)
        )));
    } else {
        out.push(p(
            "Lug 2 sits on the top surface, above the centre of gravity, so it \
             travels inward by its own height above the COG before it is \
             projected. The document's heading writes this as \
             D-2 − h×tan(θ)×cos(θ); its own working, and every figure \
             printed from it, take the projection over the whole of \
             D-2 − h×tan(θ), which is what is solved here.",
            "",
        ));
        out.push(equation(&format!(
            "\\text{{ND}}_2 = \\left(\\text{{D}}_2 - \\text{{h}} \\times \
             \\tan\\theta\\right) \\times \\cos\\theta = \
             \\left({} - {} \\times \\tan({}^\\circ)\\right) \\times \
             \\cos({}^\\circ) = {}\\ \\text{{m}}",
            tex_num(Some(dist_b), 3),
            tex_num(Some(result.cog_height), 3),
            tex_num(Some(tilt), 2),
            tex_num(Some(tilt), 2),
            tex_num(Some(result.dist_b_new), 3)
        )));
        out.push(p(
            &format!(
                "Lug 2's inward travel is {}, taken off \
                 D-2 before the projection. The projected distance reaches nothing \
                 at {}, which is where tan(θ) = \
                 D-2 / h: beyond that angle the load has tipped about crane 2 \
                 rather than hanging from both hooks, so it is a tip-over and not \
                 a rotation to plan against.",
                fmt(Some(result.lug2_travel), 3, "m"),
                fmt(Some(result.max_tilt_angle), 2, "\u{00b0}")
            ),
            "",
        ));
    }
    out.push(subheading("New Total Effective Span Between Hooks (L-new):"));
    out.push(equation(&format!(
        "\\text{{L}}_{{new}} = \\text{{ND}}_1 + \\text{{ND}}_2 = \
         {} + {} = {}\\ \\text{{m}}",
        tex_num(Some(result.dist_a_new), 3),
        tex_num(Some(result.dist_b_new), 3),
        tex_num(Some(result.span_new), 3)
    )));
    if !result.full_rotation {
        out.push(p(
            &format!(
                "The span closes from {} at the level baseline to \
                 {} at {tilt0}°, because lug 2 has \
                 travelled inward while the COG shifted \
                 {} sideways. Shortening crane 2's \
                 lever arm is what carries the load onto it.",
                fmt(Some(span), 3, "m"),
                fmt(Some(result.span_new), 3, "m"),
                fmt(Some(result.cog_shift), 3, "m")
            ),
            "",
        ));
    }
    out.push(subheading("Required Hook Height Differential to Hold the Declared Tilt:"));
    out.push(equation(&format!(
        "\\text{{Height Difference}} = \\text{{Total Span}} \\times \\sin\\theta = \
         {} \\times \\sin({}^\\circ) = {}\\ \\text{{m}}",
        tex_num(Some(span), 3),
        tex_num(Some(tilt), 2),
        tex_num(Some(result.height_difference), 3)
    )));

    out.push(heading(
        first_number + 2,
        "Rigging Factors & Tension Calculations",
    ));
    out.push(p(
        "Each hook takes the share set by the moment about the other hook, so the \
         crane nearer the COG carries the larger share. The angle factor then \
         divides that share to give the sling line tension.",
        "",
    ));
    out.push(subheading("Sling Angle Factor (AF-1 and AF-2):"));
    out.push(equation(&format!(
        "\\text{{AF}}_1 = \\text{{AF}}_2 = \\cos({}^\\circ) = {}",
        tex_num(Some(angle), 2),
        fmt(Some(result.angle_factor), 3, "")
    )));
    out.push(subheading("Base Reeve Factor (RF):"));
    out.push(equation(&format!(
        "\\text{{RF}} = {}",
        tex_num(Some(reeve), 2)
    )));
    out.push(subheading("Case A — Weight Distribution on Crane 1 (0° Level):"));
    out.push(equation(&format!(
        "\\text{{Weight}}_1 = \\text{{Load}} \\times \\frac{{\\text{{D}}_2}}\
         {{\\text{{L}}}} = {} \\times \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(load), 1),
        tex_num(Some(dist_b), 3),
        tex_num(Some(span), 3),
        tex_num(Some(result.share_a_level), 1)
    )));
    out.push(subheading("Case A — Weight Distribution on Crane 2 (0° Level):"));
    out.push(equation(&format!(
        "\\text{{Weight}}_2 = \\text{{Load}} \\times \\frac{{\\text{{D}}_1}}\
         {{\\text{{L}}}} = {} \\times \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(load), 1),
        tex_num(Some(dist_a), 3),
        tex_num(Some(span), 3),
        tex_num(Some(result.share_b_level), 1)
    )));
    out.push(subheading("Case A — Sling Line Tension at 0° Level Lift:"));
    out.push(equation(&format!(
        "\\text{{Tension}}_1 = \\frac{{\\text{{Weight}}_1}}{{\\text{{AF}}_1}} = \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.share_a_level), 1),
        tex_num(Some(af), 3),
        tex_num(Some(result.tension_a_level), 1)
    )));
    out.push(equation(&format!(
        "\\text{{Tension}}_2 = \\frac{{\\text{{Weight}}_2}}{{\\text{{AF}}_2}} = \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.share_b_level), 1),
        tex_num(Some(af), 3),
        tex_num(Some(result.tension_b_level), 1)
    )));
    out.push(subheading(&format!(
        "Case B — Dynamic Weight Share on Crane 1 ({tilt0}° Tilt):"
    )));
    out.push(equation(&format!(
        "\\text{{Weight}}_1 = \\text{{Load}} \\times \
         \\frac{{\\text{{ND}}_2}}{{\\text{{L}}_{{new}}}} = {} \\times \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(load), 1),
        tex_num(Some(result.dist_b_new), 3),
        tex_num(Some(result.span_new), 3),
        tex_num(Some(result.share_a_tilt), 1)
    )));
    out.push(subheading(&format!(
        "Case B — Dynamic Weight Share on Crane 2 ({tilt0}° Tilt):"
    )));
    out.push(equation(&format!(
        "\\text{{Weight}}_2 = \\text{{Load}} \\times \
         \\frac{{\\text{{ND}}_1}}{{\\text{{L}}_{{new}}}} = {} \\times \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(load), 1),
        tex_num(Some(result.dist_a_new), 3),
        tex_num(Some(result.span_new), 3),
        tex_num(Some(result.share_b_tilt), 1)
    )));
    out.push(subheading(&format!(
        "Case B — Sling Line Tension at {tilt0}° Dynamic Slant Lift:"
    )));
    out.push(equation(&format!(
        "\\text{{Tension}}_1 = \\frac{{\\text{{Weight}}_1}}{{\\text{{AF}}_1}} = \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.share_a_tilt), 1),
        tex_num(Some(af), 3),
        tex_num(Some(result.tension_a_tilt), 1)
    )));
    out.push(equation(&format!(
        "\\text{{Tension}}_2 = \\frac{{\\text{{Weight}}_2}}{{\\text{{AF}}_2}} = \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.share_b_tilt), 1),
        tex_num(Some(af), 3),
        tex_num(Some(result.tension_b_tilt), 1)
    )));
    let mut summary_rows = vec![
        (
            "Tension, crane 1 (level)".to_string(),
            fmt(Some(result.tension_a_level), 1, "kg"),
        ),
        (
            "Tension, crane 2 (level)".to_string(),
            fmt(Some(result.tension_b_level), 1, "kg"),
        ),
        (
            "Tension, crane 1 (tilted)".to_string(),
            fmt(Some(result.tension_a_tilt), 1, "kg"),
        ),
        (
            "Tension, crane 2 (tilted)".to_string(),
            fmt(Some(result.tension_b_tilt), 1, "kg"),
        ),
    ];
    if has_vertical {
        // Only the aligned pivot method can be rotated to vertical, and when it
        // is, the whole load ends up on crane 2. That case has to be worked
        // here, not merely mentioned.
        out.push(subheading("Case C — Weight Distribution at 90° Vertical Rotation:"));
        out.push(p(
            "Lug 2 sits under the COG on the level baseline, so the load can be \
             rotated all the way to vertical without either hook passing under \
             the COG. At 90° the COG is above crane 2's hook and beside crane \
             1's, which puts the whole weight on crane 2 and none on crane 1.",
            "",
        ));
        out.push(equation(&format!(
            "\\text{{Weight}}_1 = 0\\ \\text{{kg}}, \\qquad \
             \\text{{Weight}}_2 = \\text{{Load}} = {}\\ \\text{{kg}}",
            tex_num(Some(load), 1)
        )));
        out.push(equation(&format!(
            "\\text{{Tension}}_1 = 0\\ \\text{{kg}}, \\qquad \
             \\text{{Tension}}_2 = \\frac{{\\text{{Weight}}_2}}{{\\text{{AF}}_2}} = \
             \\frac{{{}}}{{{}}} = {}\\ \\text{{kg}}",
            tex_num(result.share_b_vertical, 1),
            tex_num(Some(af), 3),
            tex_num(result.tension_b_vertical, 1)
        )));
        summary_rows.push((
            "Tension, crane 1 (90° vertical)".to_string(),
            fmt(result.tension_a_vertical, 1, "kg"),
        ));
        summary_rows.push((
            "Tension, crane 2 (90° vertical)".to_string(),
            fmt(result.tension_b_vertical, 1, "kg"),
        ));
    } else {
        out.push(p(
            &format!(
                "This configuration cannot be rotated to vertical: the span reaches \
                 nothing at {}, so there is no \
                 90° case to rate. The two cases above are the whole of it.",
                fmt(Some(result.max_tilt_angle), 2, "\u{00b0}")
            ),
            "",
        ));
    }
    out.push(table(&summary_rows));

    out.push(heading(
        first_number + 3,
        "Final Capacity & Equipment Selection",
    ));
    out.push(p(
        "Each crane must carry the highest of its own case tensions. The \
         governing case is not the same crane in either direction, so each \
         crane is quoted against the case that sizes it.",
        "",
    ));
    let rf_af = format!("{} \\times {}", tex_num(Some(reeve), 2), tex_num(Some(af), 3));
    for (tag, letter, side) in [("a", "1", "Left"), ("b", "2", "Right")] {
        let required = if tag == "a" {
            result.required_wll_a
        } else {
            result.required_wll_b
        };
        let governing = report_case_words(Some(if tag == "a" {
            &result.governing_case_a
        } else {
            &result.governing_case_b
        }));
        // The case subscripts are assembled outside the equation: a LaTeX
        // escape cannot sit inside a template without doubling.
        let case_level = format!("{letter},\\,0^\\circ");
        let case_tilt = format!("{letter},\\,{tilt0}^\\circ");
        let case_vert = format!("{letter},\\,90^\\circ");
        let mut terms = vec![
            format!("\\frac{{\\text{{Weight}}_{{{case_level}}}}}{{{rf_af}}}"),
            format!("\\frac{{\\text{{Weight}}_{{{case_tilt}}}}}{{{rf_af}}}"),
        ];
        let share_level = if tag == "a" {
            result.share_a_level
        } else {
            result.share_b_level
        };
        let share_tilt = if tag == "a" {
            result.share_a_tilt
        } else {
            result.share_b_tilt
        };
        let mut values = vec![
            format!("\\frac{{{}}}{{{rf_af}}}", tex_num(Some(share_level), 1)),
            format!("\\frac{{{}}}{{{rf_af}}}", tex_num(Some(share_tilt), 1)),
        ];
        if has_vertical {
            terms.push(format!(
                "\\frac{{\\text{{Weight}}_{{{case_vert}}}}}{{{rf_af}}}"
            ));
            let share_vertical = if tag == "a" {
                result.share_a_vertical.unwrap_or(0.0)
            } else {
                result.share_b_vertical.unwrap_or(0.0)
            };
            values.push(format!(
                "\\frac{{{}}}{{{rf_af}}}",
                tex_num(Some(share_vertical), 1)
            ));
        }
        out.push(subheading(&format!(
            "Minimum Required Sling WLL for Crane {letter} ({side} Hook Path):"
        )));
        out.push(equation(&format!(
            "\\text{{Min WLL}}_{{{letter}}} = \
             \\max\\left({}\\right) = \
             \\max\\left({}\\right) = {}\\ \\text{{kg}}",
            terms.join(", "),
            values.join(", "),
            tex_num(Some(required), 1)
        )));
        out.push(p(
            &format!(
                "Governed by the highest value: the <b>{governing}</b> case governs \
                 crane {letter}, requiring {}.",
                fmt(Some(required), 1, "kg")
            ),
            "",
        ));
    }
    out.push(subheading("Governing Case Summary:"));
    out.push(table(&[
        (
            "Minimum required sling WLL, crane 1".to_string(),
            format!(
                "{} \u{2014} governed by the \
                 {} case",
                fmt(Some(result.required_wll_a), 1, "kg"),
                report_case_words(Some(&result.governing_case_a))
            ),
        ),
        (
            "Minimum required sling WLL, crane 2".to_string(),
            format!(
                "{} \u{2014} governed by the \
                 {} case",
                fmt(Some(result.required_wll_b), 1, "kg"),
                report_case_words(Some(&result.governing_case_b))
            ),
        ),
        (
            "Governing crane".to_string(),
            format!(
                "Crane {} \u{2014} {}",
                governing_crane(result),
                fmt(Some(result.required_wll), 1, "kg")
            ),
        ),
    ]));
    out.push(subheading("Installed Sling Tag Rating Check:"));
    let mut checked = false;
    for (tag, letter) in [("a", "1"), ("b", "2")] {
        let block = if tag == "a" {
            crate::report_capacity::installed_tag_block(
                result.tag_wll_a,
                result.rated_legs_a,
                result.tag_angle_factor_a,
                result.leg_capacity_a,
                result.required_wll_a,
                result.utilisation_a,
                letter,
            )
        } else {
            crate::report_capacity::installed_tag_block(
                result.tag_wll_b,
                result.rated_legs_b,
                result.tag_angle_factor_b,
                result.leg_capacity_b,
                result.required_wll_b,
                result.utilisation_b,
                letter,
            )
        };
        if block.is_empty() {
            continue;
        }
        checked = true;
        out.extend(block);
    }
    if checked {
        let note = crate::report_capacity::nonuniform_rating_legs_note(&[
            (
                "Crane 1".to_string(),
                Some(result.sling_legs_a),
                Some(result.rated_legs_a),
            ),
            (
                "Crane 2".to_string(),
                Some(result.sling_legs_b),
                Some(result.rated_legs_b),
            ),
        ]);
        if !note.is_empty() {
            out.push(note);
        }
    } else {
        out.push(p(
            "Not calculated \u{2014} enter each crane's installed sling tag WLL on \
             the Tandem tab to complete the equipment check.",
            "",
        ));
    }
    out.push(subheading("Crane Rated Capacity Check (75% Allowable):"));
    out.push(p(
        &format!(
            "The rated capacity is the figure read from the crane's own load chart \
             at the working radius, and it is the baseline this check is made \
             against. Usage is the share of that charted capacity the hook path \
             takes, so it is the demand over the charted figure and nothing else. \
             At or below {}% the crane is \
             within its allowable window; above the charted capacity it is over it.",
            tex_num(Some(crate::calc::CRANE_USAGE_RATIO * 100.0), 0)
        ),
        "",
    ));
    let mut rated = false;
    for (tag, letter) in [("a", "1"), ("b", "2")] {
        let usage = if tag == "a" {
            result.crane_usage_a
        } else {
            result.crane_usage_b
        };
        let Some(usage) = usage else {
            continue;
        };
        rated = true;
        let crane_rated = if tag == "a" {
            result.crane_rated_a
        } else {
            result.crane_rated_b
        };
        let required = if tag == "a" {
            result.required_wll_a
        } else {
            result.required_wll_b
        };
        out.push(equation(&format!(
            "\\text{{Usage}}_{{{letter}}} = \\frac{{\\text{{Min WLL}}_{{{letter}}}}}\
             {{\\text{{Rated Capacity}}_{{{letter}}}}} \\times 100\\% = \
             \\frac{{{}}}{{{}}} \\times 100\\% = {}",
            tex_num(Some(required), 1),
            tex_num(Some(crane_rated), 1),
            tex_num(Some(usage), 2)
        )));
        // The band is stated in the Crane tab's own words, from the one
        // classifier both tabs read.
        let state = crane_usage_band(usage);
        let tail = match state {
            UsageState::Within => ".",
            UsageState::Caution => {
                " \u{2013} above the 75% allowable window, so the charted figure has \
                 to be re-checked at the working radius before the lift."
            }
            UsageState::Over => {
                " \u{2013} over its charted capacity, so the hook path has to be \
                 re-planned against a crane of greater charted capacity at \
                 this radius."
            }
        };
        out.push(p(
            &format!(
                "Crane {letter} is <b>{}</b> at {} of \
                 its charted capacity{tail}",
                state.wording(),
                fmt(Some(usage), 2, "%")
            ),
            "",
        ));
    }
    if !rated {
        out.push(p(
            "Not calculated \u{2014} enter each crane's load chart capacity on the \
             Tandem tab to complete the 75% allowable check. A tandem lift is \
             carried by two independent cranes, so each hook path is checked \
             against its own booked rating.",
            "",
        ));
    }
    out.push(subheading("Configured Setup SWL:"));
    out.push(equation(&format!(
        "\\text{{Setup SWL}} = \\text{{Total Load Weight}} = {}\\ \\text{{kg}}",
        tex_num(Some(result.setup_swl), 0)
    )));

    // The document closes with a comparison table. It is the one place the
    // shift the tilt causes is stated as a single before-and-after.
    out.push(heading(first_number + 4, "Note"));
    out.push(subheading("Effect of the Declared Tilt on Each Hook"));
    out.push(p(
        "The same lift at the two declared load angles, for comparison. The level \
         baseline is the more onerous case for crane 1 and the tilt is the more \
         onerous case for crane 2; sizing either crane from the other case \
         would understate it.",
        "",
    ));
    out.push(
        "<table><thead><tr><th>Load angle (tilt)</th>\
         <th>Crane 1 tension (left side)</th>\
         <th>Crane 2 tension (right side)</th>\
         <th>Hook height differential</th></tr></thead><tbody>"
            .to_string(),
    );
    out.push(format!(
        "<tr><td>0° (level lift)</td>\
         <td>{}</td>\
         <td>{}</td>\
         <td>0.000 m (level)</td></tr>",
        fmt(Some(result.tension_a_level / 1000.0), 2, "t"),
        fmt(Some(result.tension_b_level / 1000.0), 2, "t")
    ));
    out.push(format!(
        "<tr><td>{tilt0}° (slanted lift)</td>\
         <td>{}</td>\
         <td>{}</td>\
         <td>{} (crane 1 higher)</td></tr>",
        fmt(Some(result.tension_a_tilt / 1000.0), 2, "t"),
        fmt(Some(result.tension_b_tilt / 1000.0), 2, "t"),
        fmt(Some(result.height_difference), 3, "m")
    ));
    out.push("</tbody></table>".to_string());
    let transfer = (result.share_b_tilt - result.share_b_level).abs();
    if result.full_rotation {
        out.push(p(
            &format!(
                "Declared tilt direction: crane 1 lifts faster, so the cargo slants \
                 downwards towards crane 2. That transfers {} \
                 onto crane 2 and releases the same weight from crane 1. The hooks \
                 must finish {} apart in \
                 height to hold the declared tilt. On this configuration the \
                 transfer continues if the rotation is carried past {tilt0}°: at \
                 90° the whole {} sits on crane 2.",
                fmt(Some(transfer), 1, "kg"),
                fmt(Some(result.height_difference), 3, "m"),
                fmt(Some(result.load_weight), 0, "kg")
            ),
            "",
        ));
    } else {
        out.push(p(
            &format!(
                "Declared tilt direction: crane 1 lifts faster, so the cargo slants \
                 downwards towards crane 2. That transfers {} \
                 onto crane 2 and releases the same weight from crane 1, and the \
                 span closing by {} does the same \
                 thing by shortening crane 2's lever arm. The hooks must finish \
                 {} apart in height to hold \
                 the declared tilt.",
                fmt(Some(transfer), 1, "kg"),
                fmt(Some(span - result.span_new), 3, "m"),
                fmt(Some(result.height_difference), 3, "m")
            ),
            "",
        ));
    }
    out
}

/// Escape helper re-exported for callers that assemble report fragments.
pub fn escape_note(text: &str) -> String {
    escape(text)
}

/// The words for a governing case, exposed for the UI summary line.
pub fn governing_words(case: Option<&str>) -> String {
    governing_case_words(case)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nonuniform::{
        example_nonuniform_inputs, solve_nonuniform, Asym2LegResult, NonuniformResult,
    };

    fn asym() -> Asym2LegResult {
        match solve_nonuniform(&example_nonuniform_inputs("asym_2leg_shortening").unwrap(), 2000.0)
            .unwrap()
        {
            NonuniformResult::Asym(result) => result,
            _ => panic!("expected the asymmetric bridle"),
        }
    }

    fn tandem() -> TandemResult {
        match solve_nonuniform(&example_nonuniform_inputs("tandem_aligned_pivot").unwrap(), 60000.0)
            .unwrap()
        {
            NonuniformResult::Tandem(result) => result,
            _ => panic!("expected the tandem lift"),
        }
    }

    fn asym_html() -> String {
        asym_sections(&asym(), 1, "Nonuniform Load").join("")
    }

    fn tandem_html() -> String {
        tandem_sections(&tandem(), 1, "Tandem Lift").join("")
    }

    #[test]
    fn the_asymmetric_bridle_report_matches_the_reference() {
        let html = asym_html();
        for expected in [
            "<h2>1. NONUNIFORM LOAD</h2>",
            "<h2>2. GEOMETRIC &amp; HEADROOM CALCULATIONS (SHORTENING METHOD WITH ELEVATION DELTA)</h2>",
            "<tr><th>Vertical height difference between pick points</th><td>0.500 m (pick point 2 higher)</td></tr>",
            "\\[\\text{X}_2 = \\text{Total Span} - \\text{X}_1 = 5.000 - 3.500 = 1.500\\ \\text{m}\\]",
            "\\[\\text{Length}_{1} = \\text{Standard Length} = 7.000\\ \\text{m}\\]",
            "\\[\\text{Y}_{1} = \\sqrt{\\text{Length}^2 - \\text{X}_{1}^2} = \\sqrt{7.000^2 - 3.500^2} = 6.062\\ \\text{m}\\]",
            "\\[\\text{Y}_{2} = \\text{Y}_{1} - \\text{Height Difference} = 6.062 - 0.500 = 5.562\\ \\text{m}\\]",
            "\\[\\text{Length}_{2} = \\sqrt{\\text{Y}_{2}^2 + \\text{X}_{2}^2} = \\sqrt{5.562^2 + 1.500^2} = 5.761\\ \\text{m}\\]",
            "\\[\\text{Shortening} = \\text{Standard Length} - \\text{Required Length} = 7.000 - 5.761 = 1.239\\ \\text{m}\\]",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_per_leg_tension_and_required_wll_match_the_reference() {
        let html = asym_html();
        for expected in [
            "\\[\\text{Weight}_2 = \\text{Load} \\times \\frac{\\text{X}_1}{\\text{Total Span}} = 2{,}000.0 \\times \\frac{3.500}{5.000} = 1{,}400.0\\ \\text{kg}\\]",
            "Tension in Sling 1 (Longer Leg):",
            "Tension in Sling 2 (Shorter Leg):",
            "\\[\\text{Tension}_2 = \\frac{\\text{Weight}_2}{\\text{AF}_2} = \\frac{1{,}400.0}{0.966} = 1{,}449.3\\ \\text{kg}\\]",
            "Minimum Required Sling WLL for Sling 1 (Tag Rating):",
            "Minimum Required Sling WLL for Sling 2 (Tag Rating):",
            "\\[\\text{Min WLL}_{2} = \\frac{\\text{Weight}_{2}}{\\text{RF} \\times \\text{AF}_{2}} = \\frac{1{,}400.0}{1.00 \\times 0.966} = 1{,}449.3\\ \\text{kg}\\]",
            "<tr><th>Governing leg</th><td>Sling 2 \u{2014} 1,449.3 kg</td></tr>",
            "\\[\\text{Setup SWL} = \\text{Load Weight} = 2{,}000\\ \\text{kg}\\]",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn a_missing_tag_says_the_check_was_not_done_rather_than_passing_it() {
        let html = asym_html();
        assert!(html.contains("Installed Tag Rating Check:"));
        assert!(html.contains(
            "Not calculated \u{2014} enter the installed tag WLL for each leg on the Nonuniform Load tab to complete the equipment check."
        ));
        assert!(!html.contains("<span class='pass'>"));
    }

    #[test]
    fn the_tandem_geometry_matches_the_reference() {
        let html = tandem_html();
        for expected in [
            "<h2>1. TANDEM LIFT</h2>",
            "<h2>2. GEOMETRIC &amp; HEADROOM CALCULATIONS (DYNAMIC TILT METHOD)</h2>",
            "\\[\\text{Angle}_1 = \\text{Angle}_2 = 0.00^\\circ\\ \\text{from vertical}\\]",
            "\\[\\text{ND}_1 = \\text{h} \\times \\sin\\theta + \\text{D}_1 \\times \\cos\\theta = 2.000 \\times \\sin(35.00^\\circ) + 3.000 \\times \\cos(35.00^\\circ) = 3.605\\ \\text{m}\\]",
            "\\[\\text{ND}_2 = \\text{D}_2 \\times \\cos\\theta = 5.000 \\times \\cos(35.00^\\circ) = 4.096\\ \\text{m}\\]",
            "\\[\\text{L}_{new} = \\text{ND}_1 + \\text{ND}_2 = 3.605 + 4.096 = 7.701\\ \\text{m}\\]",
            "\\[\\text{Height Difference} = \\text{Total Span} \\times \\sin\\theta = 8.000 \\times \\sin(35.00^\\circ) = 4.589\\ \\text{m}\\]",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn all_three_cases_are_worked_and_the_vertical_one_is_not_skipped() {
        let html = tandem_html();
        for expected in [
            "Case A \u{2014} Sling Line Tension at 0\u{00b0} Level Lift:",
            "Case B \u{2014} Sling Line Tension at 35\u{00b0} Dynamic Slant Lift:",
            "Case C \u{2014} Weight Distribution at 90\u{00b0} Vertical Rotation:",
            "\\[\\text{Tension}_1 = 0\\ \\text{kg}, \\qquad \\text{Tension}_2 = \\frac{\\text{Weight}_2}{\\text{AF}_2} = \\frac{60{,}000.0}{1.000} = 60{,}000.0\\ \\text{kg}\\]",
            "<tr><th>Tension, crane 1 (90\u{00b0} vertical)</th><td>0.0 kg</td></tr>",
            "<tr><th>Tension, crane 2 (90\u{00b0} vertical)</th><td>60,000.0 kg</td></tr>",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn each_crane_takes_the_highest_of_its_own_cases() {
        let html = tandem_html();
        for expected in [
            "Minimum Required Sling WLL for Crane 1 (Left Hook Path):",
            "Minimum Required Sling WLL for Crane 2 (Right Hook Path):",
            "\\[\\text{Min WLL}_{2} = \\max\\left(\\frac{\\text{Weight}_{2,\\,0^\\circ}}{1.00 \\times 1.000}, \\frac{\\text{Weight}_{2,\\,35^\\circ}}{1.00 \\times 1.000}, \\frac{\\text{Weight}_{2,\\,90^\\circ}}{1.00 \\times 1.000}\\right) = \\max\\left(\\frac{22{,}500.0}{1.00 \\times 1.000}, \\frac{28{,}087.3}{1.00 \\times 1.000}, \\frac{60{,}000.0}{1.00 \\times 1.000}\\right) = 60{,}000.0\\ \\text{kg}\\]",
            "<tr><th>Minimum required sling WLL, crane 1</th><td>37,500.0 kg \u{2014} governed by the 0\u{00b0} level baseline case</td></tr>",
            "<tr><th>Minimum required sling WLL, crane 2</th><td>60,000.0 kg \u{2014} governed by the 90\u{00b0} vertical rotation case</td></tr>",
            "<tr><th>Governing crane</th><td>Crane 2 \u{2014} 60,000.0 kg</td></tr>",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn a_crane_with_no_chart_capacity_says_so_rather_than_showing_zero_percent() {
        let html = tandem_html();
        assert!(html.contains("Crane Rated Capacity Check (75% Allowable):"));
        assert!(html.contains(
            "Not calculated \u{2014} enter each crane's load chart capacity on the Tandem tab to complete the 75% allowable check. A tandem lift is carried by two independent cranes, so each hook path is checked against its own booked rating."
        ));
        assert!(!html.contains("\\text{Usage}"));
    }

    #[test]
    fn the_tandem_block_closes_with_the_documents_comparison_note() {
        let html = tandem_html();
        for expected in [
            "<h2>5. NOTE</h2>",
            "Effect of the Declared Tilt on Each Hook",
            "<td>37.50 t</td>",
            "<td>22.50 t</td>",
            "<td>31.91 t</td>",
            "<td>28.09 t</td>",
            "<td>0.000 m (level)</td>",
            "<td>4.589 m (crane 1 higher)</td>",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn a_block_that_was_never_set_up_says_which_tab_to_complete() {
        let html = nonuniform_sections(None, 1, "Nonuniform Load", None).join("");
        assert!(html.contains(
            "No nonuniform load has been set up. Complete the Nonuniform Load tab to include it in this plan."
        ));
        let html = nonuniform_sections(
            None,
            1,
            "Nonuniform Load",
            Some("The COG must sit between the two pick points."),
        )
        .join("");
        assert!(html.contains(
            "The COG must sit between the two pick points. Complete the Nonuniform Load tab to include it in this plan."
        ));
        let html = nonuniform_sections(None, 4, "Tandem Lift", None).join("");
        assert!(html.contains("<h2>4. TANDEM LIFT</h2>"));
        assert!(html.contains(
            "No tandem lift has been set up. Complete the Tandem Lift tab to include it in this plan."
        ));
    }

    #[test]
    fn a_solved_example_with_no_real_load_is_refused() {
        let html = nonuniform_sections(
            Some(&NonuniformResult::Asym(asym())),
            1,
            "Nonuniform Load",
            None,
        )
        .join("");
        assert!(html.contains("1. NONUNIFORM LOAD"));

        let unloaded = solve_nonuniform(
            &example_nonuniform_inputs("asym_2leg_shortening").unwrap(),
            0.0,
        )
        .unwrap();
        let html = nonuniform_sections(Some(&unloaded), 1, "Nonuniform Load", None).join("");
        assert!(html.contains(
            "No nonuniform load has been set up. Enter a load weight on the Overall Weight tab and complete the Nonuniform Load tab to include it in this plan."
        ));
        assert!(!html.contains("Height Difference"));
    }

    #[test]
    fn the_two_blocks_number_themselves_from_wherever_they_start() {
        let from_six = tandem_sections(&tandem(), 6, "Tandem Lift");
        assert!(from_six
            .first()
            .unwrap()
            .contains("<h2>6. TANDEM LIFT</h2>"));
        assert!(from_six.join("").contains("<h2>10. NOTE</h2>"));
        assert_eq!(
            from_six
                .iter()
                .filter(|part| part.starts_with("<h2>"))
                .count(),
            5
        );
    }
}
