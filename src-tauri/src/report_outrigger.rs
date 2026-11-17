//! The outrigger pad section of the printable plan.
//!
//! The formula is printed in full so a reviewer can follow the area from the
//! ground value, the crane mass and the lift load. The selected pad is then
//! checked against that area, not merely restated.

use crate::outrigger::OutriggerResult;
use crate::report::{equation, escape, fmt, heading, p, subheading, tex_num};

fn status_word(status: &str) -> &'static str {
    match status {
        "ok" => "Within limit",
        "caution" => "Caution",
        "over" => "Warning",
        _ => "Pad size not entered",
    }
}

fn status_colour(status: &str) -> &'static str {
    match status {
        "ok" => "color:#2e7d52;font-weight:bold;",
        "caution" => "color:#b06a1e;font-weight:bold;",
        "over" => "color:#b23b3b;font-weight:bold;",
        _ => "color:#64748d;font-weight:bold;",
    }
}

/// Build the outrigger section for the report.
pub fn outrigger_section(result: &OutriggerResult) -> String {
    let mut out = vec![heading(2, "Outrigger Pad Size")];
    out.push(p(
        "Presumed allowable bearing values from BS 8004 Table 1, adapted for Malaysian practice. \
         The calculation sizes the bearing area only; pad thickness, steel plate design and the \
         crane manufacturer's outrigger load are separate checks.",
        "",
    ));

    let table_value = if result.ground_value_label == "Not applicable" {
        "Not applicable".to_string()
    } else {
        format!("{} kPa", result.ground_value_label)
    };
    out.push(p(
        &format!(
            "<b>Ground:</b> {} - {} <span class=\"faint\">(Table value: {})</span>",
            escape(&result.ground_category),
            escape(&result.ground_description),
            escape(&table_value)
        ),
        "",
    ));
    out.push(p(&format!("<b>Field note:</b> {}", escape(&result.ground_notes)), ""));

    out.push(subheading("Design load"));
    out.push(equation(&format!(
        "P = 0.65 \\times (CM + L) = 0.65 \\times ({} + {}) = {}\\ \\text{{kN}}",
        tex_num(Some(result.crane_mass_kn), 2),
        tex_num(Some(result.lift_load_kn), 2),
        tex_num(Some(result.design_load_kn), 2)
    )));

    out.push(subheading("Required bearing area"));
    out.push(equation(&format!(
        "A = \\frac{{0.65 \\times (CM + L)}}{{V}} = \
         \\frac{{{}}}{{{}}} = {}\\ \\text{{m}}^2",
        tex_num(Some(result.design_load_kn), 2),
        tex_num(Some(result.bearing_kpa), 2),
        tex_num(Some(result.required_area_m2), 3)
    )));
    out.push(equation(&format!(
        "\\text{{Minimum square side}} = \\sqrt{{A}} = \
         {}\\ \\text{{m}}",
        tex_num(Some(result.recommended_square_m), 3)
    )));
    out.push(p(
        &format!(
            "Round up to <b>{}</b> at 50 mm increments.",
            fmt(Some(result.recommended_square_rounded_m), 2, "m")
        ),
        "",
    ));

    if let Some(pad_area) = result.pad_area_m2 {
        out.push(subheading("Selected pad"));
        out.push(equation(&format!(
            "\\text{{Pad area}} = {} \\times {} = {}\\ \\text{{m}}^2",
            tex_num(Some(result.pad_length_m), 2),
            tex_num(Some(result.pad_width_m), 2),
            tex_num(Some(pad_area), 3)
        )));
        if let Some(pressure) = result.applied_pressure_kpa {
            out.push(equation(&format!(
                "p_{{\\text{{applied}}}} = \\frac{{P}}{{A_{{\\text{{pad}}}}}} = \
                 \\frac{{{}}}{{{}}} = {}\\ \\text{{kPa}}",
                tex_num(Some(result.design_load_kn), 2),
                tex_num(Some(pad_area), 3),
                tex_num(Some(pressure), 2)
            )));
        }
        if let Some(use_percent) = result.utilisation_percent {
            out.push(equation(&format!(
                "\\text{{Utilisation}} = \\frac{{p_{{\\text{{applied}}}}}}{{V}} \\times 100\\% = \
                 {}\\%",
                tex_num(Some(use_percent), 2)
            )));
        }
    }

    out.push(p(
        &format!(
            "<b>{}:</b> {}",
            status_word(&result.status),
            escape(&result.message)
        ),
        status_colour(&result.status),
    ));
    out.push(p(
        "The pad is sized for the vertical design load. Confirm the crane's outrigger reaction, \
         the pad build-up and the ground profile before the lift.",
        "color:#64748d;font-size:8.5pt;",
    ));

    out.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outrigger::{example_outrigger_inputs, solve_outrigger};

    #[test]
    fn the_pad_section_prints_the_formula_and_the_selected_check() {
        let result = solve_outrigger(&example_outrigger_inputs()).unwrap();
        let html = outrigger_section(&result);
        assert!(html.contains("OUTRIGGER PAD SIZE"));
        assert!(html.contains("0.65 \\times (CM + L)"));
        assert!(html.contains("1.950"));
        assert!(html.contains("2.00 \\times 2.00"));
        assert!(html.contains("Within limit"));
    }

    #[test]
    fn a_missing_pad_size_is_reported_without_failing_the_section() {
        let mut input = example_outrigger_inputs();
        input.pad_length_m = 0.0;
        input.pad_width_m = 0.0;
        let result = solve_outrigger(&input).unwrap();
        let html = outrigger_section(&result);
        assert!(html.contains("Pad size not entered"));
        assert!(html.contains("1.40")); // the 1.396 m square rounds to 1.40 m
    }
}
