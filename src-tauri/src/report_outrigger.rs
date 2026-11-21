//! The outrigger pad section of the printable plan.
//!
//! The formula is printed in full so a reviewer can follow the area from the
//! ground value, the crane mass and the lift load. The result is the minimum
//! pad and the thickness guidance, not a pass/fail check of one already chosen.

use crate::outrigger::OutriggerResult;
use crate::report::{equation, escape, fmt, heading, p, subheading, tex_num, working_pair};

/// Build the outrigger section for the report.
pub fn outrigger_section(result: &OutriggerResult) -> String {
    let mut out = vec![heading(2, "Outrigger Pad Size")];

    let design = format!(
        "<div class=\"work-label\">Design load</div>{}{}",
        equation("P = 0.65 \\times (CM + L)"),
        equation(&format!(
            "P = 0.65 \\times ({} + {}) = {}\\ \\text{{kN}}",
            tex_num(Some(result.crane_mass_kn), 2),
            tex_num(Some(result.lift_load_kn), 2),
            tex_num(Some(result.design_load_kn), 2)
        ))
    );
    let area = format!(
        "<div class=\"work-label\">Required bearing area</div>{}",
        equation(&format!(
            "A = \\frac{{0.65 \\times (CM + L)}}{{V}} = \\frac{{{}}}{{{}}} = {}\\ \\text{{m}}^2",
            tex_num(Some(result.design_load_kn), 2),
            tex_num(Some(result.bearing_kpa), 2),
            tex_num(Some(result.required_area_m2), 3)
        ))
    );
    out.push(working_pair(&design, &area));

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
        // The subheading and the working share a line: the square's side is
        // one short result, and A4 has better uses for the vertical room.
        out.push(format!(
            "<div class=\"pad-square-line\">\
             <span class=\"pad-square-k\">Minimum square pad</span>\
             <span class=\"pad-square-eq\">\\(\\text{{Side}} = \\sqrt{{A}} = {}\\ \\text{{m}} \\approx {}\\ \\text{{m}}\\)</span>\
             </div>",
            tex_num(Some(result.recommended_square_m), 3),
            tex_num(Some(result.recommended_square_rounded_m), 2)
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
    let thickness_cell = format!(
        "<div class=\"work-label\">Pad thickness</div>\
         <p><b>Material:</b> {} <span class=\"faint\">| Crane class: {}</span></p>\
         <p><b>Recommended thickness:</b> {recommended}. \
         <span class=\"faint\">Governing check: {}</span></p>\
         <p>Steel plate range: {}. Timber range: {}.</p>{}{}{}",
        if thickness.material == "timber" {
            "Hardwood timber"
        } else {
            "Steel plate"
        },
        escape(&thickness.crane_class),
        escape(&thickness.governing),
        escape(&thickness.steel_range_mm),
        escape(&thickness.timber_range_mm),
        if thickness.cross_layer {
            "<p><b>Cross-layering:</b> required for this pad size.</p>"
        } else {
            ""
        },
        if thickness.stiffeners {
            "<p><b>Stiffeners:</b> required, or use a 40–50 mm plate.</p>"
        } else {
            ""
        },
        if thickness.notes.is_empty() {
            String::new()
        } else {
            format!(
                "<p style=\"color:#64748d;font-size:8.5pt;\">{}</p>",
                escape(&thickness.notes)
            )
        }
    );

    let table_value = if result.ground_value_label == "Not applicable" {
        "Not applicable".to_string()
    } else {
        format!("{} kPa", result.ground_value_label)
    };
    let ground_cell = format!(
        "<div class=\"work-label\">Ground</div>\
         <p><b>Ground:</b> {} - {} <span class=\"faint\">(Table value: {})</span></p>\
         <p><b>Field note:</b> {}</p>",
        escape(&result.ground_category),
        escape(&result.ground_description),
        escape(&table_value),
        escape(&result.ground_notes)
    );
    out.push(working_pair(&thickness_cell, &ground_cell));

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
        assert!(html.contains("\\approx 1.40"));
        assert!(!html.contains("Minimum pad:"));
        assert!(html.contains("Pad thickness"));
        assert!(!html.contains("Utilisation"));
        assert!(!html.contains("at 50 mm increments"));
    }

    #[test]
    fn the_ground_text_comes_after_the_pad_result() {
        let result = solve_outrigger(&example_outrigger_inputs()).unwrap();
        let html = outrigger_section(&result);
        let pad = html.find("Minimum square pad").unwrap();
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
