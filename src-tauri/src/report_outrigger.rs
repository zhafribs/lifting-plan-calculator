//! The outrigger pad section of the printable plan.
//!
//! The formula is printed in full so a reviewer can follow the area from the
//! ground value, the crane mass and the lift load. The result is the minimum
//! pad and the thickness guidance, not a pass/fail check of one already chosen.

use crate::outrigger::OutriggerResult;
use crate::report::{equation, escape, fmt, heading, p, subheading, tex_num};

/// Build the outrigger section for the report.
pub fn outrigger_section(result: &OutriggerResult) -> String {
    let mut out = vec![heading(2, "Outrigger Pad Size")];

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

    if let (Some(width), Some(length)) = (result.pad_width_m, result.minimum_length_m) {
        out.push(subheading("Minimum rectangular pad"));
        out.push(equation(&format!(
            "L_{{\\text{{min}}}} = \\frac{{A}}{{W}} = {}\\ \\text{{m}}",
            tex_num(Some(length), 2)
        )));
        out.push(p(
            &format!(
                "<b>Minimum pad:</b> {} × {} m at the chosen {} m width.",
                fmt(Some(length), 2, ""),
                fmt(Some(width), 2, ""),
                fmt(Some(width), 2, "")
            ),
            "",
        ));
    } else {
        out.push(subheading("Minimum square pad"));
        out.push(equation(&format!(
            "\\text{{Side}} = \\sqrt{{A}} = {}\\ \\text{{m}}",
            tex_num(Some(result.recommended_square_m), 3)
        )));
        out.push(p(
            &format!(
                "<b>Minimum pad:</b> {} × {} m, rounded up from {} m.",
                fmt(Some(result.recommended_square_rounded_m), 2, ""),
                fmt(Some(result.recommended_square_rounded_m), 2, ""),
                fmt(Some(result.recommended_square_m), 3, "")
            ),
            "",
        ));
    }

    let thickness = &result.thickness;
    let recommended = if thickness.recommended_max_mm > thickness.recommended_min_mm {
        format!(
            "{}–{} mm",
            fmt(Some(thickness.recommended_min_mm), 0, ""),
            fmt(Some(thickness.recommended_max_mm), 0, "")
        )
    } else {
        fmt(Some(thickness.recommended_min_mm), 0, "mm")
    };
    out.push(subheading("Pad thickness"));
    out.push(p(
        &format!(
            "<b>Material:</b> {} <span class=\"faint\">| Crane class: {}</span>",
            if thickness.material == "timber" {
                "Hardwood timber"
            } else {
                "Steel plate"
            },
            escape(&thickness.crane_class)
        ),
        "",
    ));
    out.push(p(
        &format!(
            "<b>Recommended thickness:</b> {recommended}. \
             <span class=\"faint\">Governing check: {}</span>",
            escape(&thickness.governing)
        ),
        "",
    ));
    out.push(p(
        &format!(
            "Steel plate range: {}. Timber range: {}.",
            escape(&thickness.steel_range_mm),
            escape(&thickness.timber_range_mm)
        ),
        "",
    ));
    if thickness.cross_layer {
        out.push(p("<b>Cross-layering:</b> required for this pad size.", ""));
    }
    if thickness.stiffeners {
        out.push(p(
            "<b>Stiffeners:</b> required, or use a 40–50 mm plate.",
            "",
        ));
    }
    if !thickness.notes.is_empty() {
        out.push(p(&escape(&thickness.notes), "color:#64748d;font-size:8.5pt;"));
    }

    out.push(subheading("Ground"));
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
    out.push(p(
        &format!("<b>Field note:</b> {}", escape(&result.ground_notes)),
        "",
    ));

    out.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outrigger::{example_outrigger_inputs, solve_outrigger};

    #[test]
    fn the_pad_section_prints_the_formula_the_minimum_square_and_the_thickness() {
        let result = solve_outrigger(&example_outrigger_inputs()).unwrap();
        let html = outrigger_section(&result);
        assert!(html.contains("OUTRIGGER PAD SIZE"));
        assert!(html.contains("0.65 \\times (CM + L)"));
        assert!(html.contains("1.950"));
        assert!(html.contains("1.40 × 1.40 m"));
        assert!(html.contains("Pad thickness"));
        assert!(!html.contains("Utilisation"));
        assert!(!html.contains("at 50 mm increments"));
    }

    #[test]
    fn the_ground_text_comes_after_the_pad_result() {
        let result = solve_outrigger(&example_outrigger_inputs()).unwrap();
        let html = outrigger_section(&result);
        let pad = html.find("Minimum pad:").unwrap();
        let ground = html.find("<b>Ground:</b>").unwrap();
        assert!(pad < ground);
    }

    #[test]
    fn a_rectangular_pad_prints_its_minimum_length() {
        let mut input = example_outrigger_inputs();
        input.pad_shape = "rectangular".to_string();
        input.pad_width_m = 2.0;
        let result = solve_outrigger(&input).unwrap();
        let html = outrigger_section(&result);
        assert!(html.contains("Minimum rectangular pad"));
        assert!(html.contains("1.00 × 2.00 m"));
        assert!(!html.contains("Utilisation"));
    }
}
