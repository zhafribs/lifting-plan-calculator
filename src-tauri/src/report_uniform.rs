//! The uniform-lift report sections, ported from `ReportUniform.kt`.

use crate::report::{equation, fmt, heading, p, plain, subheading, table, tex_deg, tex_num};
use crate::sling::{
    HitchSpec, SecondaryGroup, SlingResult, ANGLE_BASIS_EXTERNAL,
};

// ---------------------------------------------------------------------------
// The nested systems' per-group labels
// ---------------------------------------------------------------------------

/// One transverse leg group of a nested arrangement.
#[derive(Debug, Clone, Copy)]
pub struct SecondaryLabel {
    pub name: &'static str,
    pub caption: &'static str,
    pub token: &'static str,
    pub phrase: &'static str,
}

fn secondary_group_labels(nested: i64) -> &'static [SecondaryLabel] {
    const THREE: [SecondaryLabel; 2] = [
        SecondaryLabel {
            name: "Outer",
            caption: "Outer Secondary Legs 1 & 3",
            token: "Outer",
            phrase: "the Sloped Outer Secondary Legs",
        },
        SecondaryLabel {
            name: "Middle",
            caption: "Middle Secondary Leg 2",
            token: "Mid",
            phrase: "the Straight Middle Secondary Legs",
        },
    ];
    const FOUR: [SecondaryLabel; 2] = [
        SecondaryLabel {
            name: "Outer",
            caption: "Outer Secondary Legs 1 & 4",
            token: "Outer",
            phrase: "the Sloped Outer Secondary Legs (Piles 1 & 4)",
        },
        SecondaryLabel {
            name: "Inner",
            caption: "Inner Secondary Legs 2 & 3",
            token: "Inner",
            phrase: "the Sloped Inner Secondary Legs (Piles 2 & 3)",
        },
    ];
    match nested {
        3 => &THREE,
        4 => &FOUR,
        _ => &[],
    }
}

/// The nested documents state the total headroom with these captions.
fn nested_height_caption(nested: i64) -> &'static str {
    match nested {
        4 => "Total Vertical Height (Hook to Top of RC Piles):",
        _ => "Total Vertical Height (Hook to Top of RC Piles Headroom):",
    }
}

/// The per-group labels for a nested arrangement, outermost first.
///
/// Empty when the group count does not match the arrangement's caption list,
/// which is the source's own guard.
pub fn secondary_labels(
    result: &SlingResult,
    nested: i64,
) -> Vec<(SecondaryGroup, SecondaryLabel)> {
    let groups = result.secondary_groups.clone().unwrap_or_default();
    let labels = secondary_group_labels(nested);
    if groups.len() != labels.len() || groups.len() < 2 {
        return Vec::new();
    }
    groups.into_iter().zip(labels.iter().copied()).collect()
}

// ---------------------------------------------------------------------------
// Section 1 -- lift input parameters
// ---------------------------------------------------------------------------

/// How the arrangement reads as a sling configuration.
pub fn configuration_label(spec: &HitchSpec) -> String {
    let legs = format!(
        "{} Effective Leg{}",
        spec.effective_legs,
        if spec.effective_legs != 1 { "s" } else { "" }
    );
    if spec.leg_count > 1 {
        format!("{} Hitch ({legs})", spec.label)
    } else {
        format!("1-Leg Sling {} Hitch ({legs})", spec.label)
    }
}

/// The rows describing the load or the pick points.
pub fn dimension_rows(result: &SlingResult, spec: &HitchSpec) -> Vec<(String, String)> {
    if spec.round_load {
        let diameter = result.diameter;
        return vec![(
            "Load Dimensions".to_string(),
            format!(
                "Round (Diameter = {} m / Radius = {} m)",
                plain(Some(diameter), 1),
                plain(Some(diameter / 2.0), 1)
            ),
        )];
    }
    if spec.rectangular_load {
        return vec![(
            "Load Dimensions".to_string(),
            format!(
                "Rectangular (Side = {} m, Bottom = {} m)",
                plain(Some(result.side), 1),
                plain(Some(result.bottom), 1)
            ),
        )];
    }
    if spec.shape == "bridle3" {
        return vec![(
            "Distance Between Pick Points (Uniform)".to_string(),
            format!("{} m", plain(Some(result.pick_distance), 1)),
        )];
    }
    if spec.shape == "bridle4" {
        return vec![
            (
                "Distance Between Pick Points (Length)".to_string(),
                format!("{} m", plain(Some(result.pick_length), 1)),
            ),
            (
                "Distance Between Pick Points (Width)".to_string(),
                format!("{} m", plain(Some(result.pick_width), 1)),
            ),
        ];
    }
    if spec.nested != 0 {
        return vec![(
            "Load Dimensions".to_string(),
            "Not required for the nested sling system".to_string(),
        )];
    }
    vec![(
        "Load Dimensions".to_string(),
        "Not required for a direct vertical hitch".to_string(),
    )]
}

/// Section 1: what the sling is and what it is carrying.
pub fn section_inputs(result: &SlingResult, spec: &HitchSpec) -> String {
    let weight = result.load_weight;
    let length = result.sling_length;
    let mut parts = vec![heading(1, "Lift Input Parameters")];
    let mut rows: Vec<(String, String)> = vec![
        ("Load Weight".to_string(), fmt(Some(weight), 1, "kg")),
        (
            "Sling Configuration".to_string(),
            configuration_label(spec),
        ),
        (
            "Sling assembly count".to_string(),
            if spec.bridle {
                "1 bridle sling assembly".to_string()
            } else if spec.leg_count == 1 {
                "1 continuous sling".to_string()
            } else {
                format!("{}-sling bridle set", spec.leg_count)
            },
        ),
        (
            "Sling Total Length".to_string(),
            fmt(Some(length), 1, "m"),
        ),
    ];
    rows.extend(dimension_rows(result, spec));
    if spec.needs_pick_distance && !spec.bridle {
        rows.push((
            "Distance Between Pick Points".to_string(),
            fmt(Some(result.pick_distance), 1, "m"),
        ));
    }
    if result.nested != 0 {
        // A secondary system of more than two inline legs is described by its
        // pile count, as in the reference documents.
        let piles = result.secondary_legs.unwrap_or(0);
        if piles > 2 {
            rows.push(("No of Piles".to_string(), piles.to_string()));
        }
        rows.push((
            "Second Sling Total Length".to_string(),
            fmt(Some(result.secondary_length), 3, "m"),
        ));
        rows.push((
            "Secondary Pick Spacing".to_string(),
            fmt(Some(result.secondary_spread), 3, "m"),
        ));
        if piles > 2 {
            rows.push(("Sling Legs per Pile".to_string(), "2".to_string()));
        }
    }
    if spec.needs_angle {
        let internal = result.internal_angle;
        let external = result.external_angle;
        if result.angle_basis == ANGLE_BASIS_EXTERNAL {
            rows.push((
                "Choke Angle (External)".to_string(),
                format!("{} - from hook to the load", fmt(external, 0, "\u{00b0}")),
            ));
        } else {
            rows.push((
                "Choke Internal Angle".to_string(),
                format!("{} from vertical", fmt(internal, 0, "\u{00b0}")),
            ));
        }
    }
    parts.push(table(&rows));
    parts.push(p(
        "Load Weight is the gross load at hook. The 75% allowable window \
         applies to the crane load chart only and is shown in the crane section.",
        "",
    ));
    parts.join("")
}

// ---------------------------------------------------------------------------
// Section 2 -- geometry and headroom
// ---------------------------------------------------------------------------

/// The choke-to-top-of-load height, stating its derivation.
fn choke_height_equation(
    key: &str,
    result: &SlingResult,
    gap: f64,
    internal: f64,
    external: f64,
) -> String {
    if external < 90.0 {
        return equation(
            "\\text{Height}_{Choke-Load} = 0.000\\ \\text{m}\\quad\
             (\\text{external angle} < 90^\\circ\\text{, choke flush to the load})",
        );
    }
    if key == "two_leg_round_choke" {
        let radius = result.diameter / 2.0;
        return equation(&format!(
            "\\text{{Height}}_{{Choke-Load}} = \\frac{{\\text{{Radius}}}}{{\\sin(\\theta)}}\
             - \\text{{Radius}} = \\frac{{{}}}{{\\sin({}^\\circ)}} - \
             {} = {}\\ \\text{{m}}",
            tex_num(Some(radius), 3),
            tex_num(Some(internal), 0),
            tex_num(Some(radius), 3),
            tex_num(Some(gap), 3)
        ));
    }
    let bottom = result.bottom;
    equation(&format!(
        "\\text{{Height}}_{{Choke-Load}} = \\frac{{\\text{{Bottom}}/2}}{{\\tan(\\theta)}} = \
         \\frac{{{}}}{{\\tan({}^\\circ)}} = {}\\ \\text{{m}}",
        tex_num(Some(bottom / 2.0), 3),
        tex_num(Some(internal), 0),
        tex_num(Some(gap), 3)
    ))
}

/// Section 2: the wrap, the free leg, the sling angle and the headroom.
#[allow(clippy::too_many_lines)]
pub fn geometry_section(result: &SlingResult, spec: &HitchSpec) -> String {
    let key = spec.key;
    let length = result.sling_length;
    let wrap = result.wrap_length;
    let free = result.free_leg_length;
    let angle = result.angle;
    let af = result.angle_factor;
    let headroom = result.hook_to_load;
    let mut out = vec![heading(2, "Geometric & Headroom Calculations")];

    // --- The bridles -------------------------------------------------------
    if key == "three_leg_direct" || key == "four_leg_direct" {
        let radius_spread: f64;
        let radius_latex: String;
        if key == "three_leg_direct" {
            let pick_distance = result.pick_distance;
            radius_spread = pick_distance / 3.0f64.sqrt();
            radius_latex = format!(
                "\\text{{Radius Spread}} = \\frac{{\\text{{Pick Distance}}}}{{\\sqrt{{3}}}} = \
                 \\frac{{{}}}{{\\sqrt{{3}}}} = {}\\ \\text{{m}}",
                tex_num(Some(pick_distance), 3),
                tex_num(Some(radius_spread), 3)
            );
        } else {
            let pick_length = result.pick_length;
            let pick_width = result.pick_width;
            radius_spread = (pick_length / 2.0).hypot(pick_width / 2.0);
            radius_latex = format!(
                "\\text{{Radius Spread}} = \
                 \\sqrt{{\\left(\\frac{{\\text{{Length}}}}{{2}}\\right)^2 + \
                 \\left(\\frac{{\\text{{Width}}}}{{2}}\\right)^2}} = \
                 \\sqrt{{\\left(\\frac{{{}}}{{2}}\\right)^2 + \
                 \\left(\\frac{{{}}}{{2}}\\right)^2}} = {}\\ \\text{{m}}",
                tex_num(Some(pick_length), 1),
                tex_num(Some(pick_width), 1),
                tex_num(Some(radius_spread), 3)
            );
        }
        out.push(subheading(
            "Centroid Radius (Horizontal Distance from Center to Any Pick Point):",
        ));
        out.push(equation(&radius_latex));
        out.push(subheading("Sling Contact Perimeter Wrap:"));
        out.push(equation("\\text{Wrap Length} = 0.000\\ \\text{m}"));
        out.push(subheading(if key == "four_leg_direct" {
            "Free Leg Length (Standing Sloped Line per Sling):"
        } else {
            "Free Leg Length:"
        }));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}} = \\text{{Total Length}} - \\text{{Wrap Length}} = \
             {} - 0.000 = {}\\ \\text{{m}}",
            tex_num(Some(length), 1),
            tex_num(Some(free), 3)
        )));
        out.push(subheading("Sling Leg Angle from Vertical:"));
        out.push(equation(&format!(
            "\\text{{Angle}} = \\sin^{{-1}}\\left(\\frac{{\\text{{Radius Spread}}}}\
             {{\\text{{Free Leg Length}}}}\\right) = \
             \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
            tex_num(Some(radius_spread), 3),
            tex_num(Some(free), 3),
            tex_deg(Some(angle), 2)
        )));
        out.push(subheading(if key == "four_leg_direct" {
            "Vertical Height (Hook to Top of Load Headroom):"
        } else {
            "Vertical Height (Hook to Top of Load):"
        }));
        out.push(equation(&format!(
            "\\text{{Height}}_{{Hook-Load}} = \\text{{Free Leg Length}} \\times \
             \\cos({}) = {} \\times {} = {}\\ \\text{{m}}",
            tex_deg(Some(angle), 2),
            tex_num(Some(free), 3),
            tex_num(Some(af), 3),
            tex_num(Some(headroom), 3)
        )));
        return out.join("");
    }

    // --- The nested systems ------------------------------------------------
    if result.nested != 0 {
        let nested_n = result.nested;
        let sec_len = result.secondary_length;
        let sec_height = result.secondary_height.unwrap_or(0.0);
        // The height is the vertical projection of the governing (outermost)
        // leg, so it carries that group's own angle.
        let sec_height_angle = result
            .secondary_height_angle
            .or(result.secondary_angle)
            .unwrap_or(0.0);
        let labelled = secondary_labels(result, nested_n);
        let height_token = if labelled.len() > 1 { "Outer" } else { "" };
        let height_latex = format!(
            "\\text{{Height}}_{{Sling2}} = \\text{{Free Leg Length}}_2 \\times {} = \
             {} \\times {} = {}\\ \\text{{m}}",
            if !height_token.is_empty() {
                "\\cos(\\text{Angle}_{Trans\\,Outer})"
            } else {
                "\\cos(\\text{Angle}_{Trans})"
            },
            tex_num(Some(sec_len), 3),
            tex_num(Some(sec_height_angle.to_radians().cos()), 3),
            tex_num(Some(sec_height), 3)
        );

        out.push(subheading("Sling Contact Perimeter Wrap:"));
        out.push(equation("\\text{Wrap Length} = 0.000\\ \\text{m}"));
        out.push(subheading("Free Leg Length (First / Master Sling):"));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}}_1 = {}\\ \\text{{m}}",
            tex_num(Some(free), 3)
        )));
        out.push(subheading("Free Leg Length (Second / Secondary Sling):"));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}}_2 = {}\\ \\text{{m}}",
            tex_num(Some(result.secondary_length), 3)
        )));

        if !labelled.is_empty() {
            // The 3- and 4-leg nested documents state one transverse angle per
            // leg group, outermost first.
            for (group, label) in &labelled {
                out.push(subheading(&format!(
                    "Transverse Sling Leg Angle from Vertical ({}):",
                    label.caption
                )));
                out.push(equation(&format!(
                    "\\text{{Angle}}_{{\\text{{Trans {}}}}} = \
                     \\sin^{{-1}}\\left(\\frac{{\\text{{Offset}}}}\
                     {{\\text{{Free Leg Length}}_2}}\\right) = \
                     \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
                    label.token,
                    tex_num(Some(group.offset), 3),
                    tex_num(Some(sec_len), 3),
                    tex_deg(Some(group.angle), 2)
                )));
            }
        } else {
            // The source's operator precedence leaves the nested-2 heading
            // without its closing parenthesis; kept verbatim.
            let group = if nested_n == 2 {
                "Second Sling"
            } else {
                "Outer Secondary Legs):"
            };
            out.push(subheading(&format!(
                "Transverse Sling Leg Angle from Vertical ({group}"
            )));
            out.push(equation(&format!(
                "\\text{{Angle}}_{{Trans}} = \
                 \\sin^{{-1}}\\left(\\frac{{\\text{{Offset}}}}\
                 {{\\text{{Free Leg Length}}_2}}\\right) = \
                 \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
                tex_num(
                    Some(result.secondary_length * (result.secondary_angle.unwrap_or(0.0)).to_radians().sin()),
                    3
                ),
                tex_num(Some(result.secondary_length), 3),
                tex_deg(result.secondary_angle, 2)
            )));
        }

        if nested_n == 4 {
            let inner_offset = labelled.last().map(|(group, _)| group.offset).unwrap_or(0.0);
            let outer_offset = labelled.first().map(|(group, _)| group.offset).unwrap_or(0.0);
            out.push(subheading("Rigging Arrangement & Pile Level Tolerance:"));
            out.push(p(
                &format!(
                    "There are 4 piles in a row, so the master link hangs between Pile 2 \
                     and Pile 3 and there is no vertical centre leg. The 4 secondary \
                     legs form two pairs: an inner pair at a {:.1} m offset (Piles 2 & 3) \
                     and an outer pair at a {:.1} m offset (Piles 1 & 4). Every leg is \
                     cut to the same free length and all four terminate on that one \
                     master link, so the pile tops cannot be level: the outer piles must \
                     stand higher than the inner piles by the amount derived below. That \
                     difference is what lets one leg length serve all four legs. The \
                     outer pair is the reference for the secondary height, because it \
                     carries the highest tension and gives the lowest angle factor, so it \
                     is the conservative case for the master legs.",
                    inner_offset, outer_offset
                ),
                "",
            ));
        } else if nested_n == 3 {
            let outer_offset = labelled.first().map(|(group, _)| group.offset).unwrap_or(0.0);
            out.push(subheading("Rigging Arrangement & Pile Level Tolerance:"));
            out.push(p(
                &format!(
                    "There are 3 piles in a row, so the master link hangs over the middle \
                     pile. The middle secondary leg is vertical and the outer pair \
                     slopes at a {:.1} m offset (Piles 1 & 3). All three legs are cut to \
                     the same free length and all three terminate on that one master \
                     link, so the pile tops cannot be level: the outer piles must stand \
                     higher than the middle pile by the amount derived below. The outer \
                     pair is the reference for the secondary height, because it carries \
                     the highest tension and gives the lowest angle factor.",
                    outer_offset
                ),
                "",
            ));
        } else {
            out.push(subheading("Rigging Arrangement:"));
            out.push(p(
                "A 2-leg master bridle splits into two 2-leg secondary slings. All \
                 four legs reach the one master link at the same offset either \
                 side of it, so all four pile tops are level and the secondary \
                 height is the same above every pile.",
                "",
            ));
        }

        let governing_piles = match nested_n {
            3 => Some("Piles 1 & 3"),
            4 => Some("Piles 1 & 4"),
            _ => None,
        };
        let mut height_caption =
            "Sling 2 Vertical Height (Top of Load to Master Link)".to_string();
        if let Some(piles) = governing_piles {
            height_caption += &format!(
                ": (Governed by the outer sloped leg, {piles}, the highest piles)"
            );
        }
        out.push(subheading(&height_caption));
        out.push(equation(&height_latex));

        let level_gap = result.pile_level_difference.unwrap_or(0.0);
        if level_gap > 0.0 && labelled.len() > 1 {
            let inner_af = labelled.last().map(|(group, _)| group.af).unwrap_or(0.0);
            let outer_af = labelled.first().map(|(group, _)| group.af).unwrap_or(0.0);
            let inner_token = if nested_n == 3 { "Mid" } else { "Inner" };
            out.push(subheading(
                "Required Pile Level Difference (all legs cut to the same free length):",
            ));
            out.push(equation(&format!(
                "\\text{{Level Difference}} = \\text{{Free Leg Length}}_2 \\times \
                 \\left(\\cos(\\text{{Angle}}_{{Trans\\,{inner_token}}}) - \
                 \\cos(\\text{{Angle}}_{{Trans\\,Outer}})\\right) = \
                 {} \\times ({} - {}) = {}\\ \\text{{m}}",
                tex_num(Some(sec_len), 3),
                tex_num(Some(inner_af), 3),
                tex_num(Some(outer_af), 3),
                tex_num(Some(level_gap), 3)
            )));
        }

        out.push(subheading("Total Combined Length for Longitudinal Calculation:"));
        out.push(equation(&format!(
            "\\text{{Total Combined}} = \\text{{Free Leg Length}}_1 + \
             \\text{{Height}}_{{Sling2}} = {} + {} = {}\\ \\text{{m}}",
            tex_num(Some(free), 3),
            tex_num(result.secondary_height, 3),
            tex_num(result.total_combined_length, 3)
        )));
        out.push(subheading("Longitudinal Sling Leg Angle from Vertical:"));
        out.push(equation(&format!(
            "\\text{{Angle}}_{{Long}} = \\sin^{{-1}}\\left(\\frac{{\\text{{Master Spread}}/2}}\
             {{\\text{{Total Combined}}}}\\right) = \
             \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
            tex_num(Some(result.pick_distance / 2.0), 3),
            tex_num(result.total_combined_length, 3),
            tex_deg(Some(angle), 2)
        )));
        out.push(subheading(
            nested_height_caption(nested_n),
        ));
        out.push(equation(&format!(
            "\\text{{Height}}_{{Hook-Load}} = \\text{{Total Combined}} \\times \
             \\cos({}) = {} \\times {} = {}\\ \\text{{m}}",
            tex_deg(Some(angle), 2),
            tex_num(result.total_combined_length, 3),
            tex_num(Some(af), 3),
            tex_num(Some(headroom), 3)
        )));
        return out.join("");
    }

    // --- The 2-leg direct bridle -------------------------------------------
    if key == "two_leg_direct" {
        let pick = result.pick_distance;
        out.push(subheading("Sling Contact Perimeter Wrap:"));
        out.push(equation("\\text{Wrap Length} = 0.000\\ \\text{m}"));
        out.push(subheading("Free Leg Length (Each Side):"));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}} = \\text{{Total Length}} - \\text{{Wrap Length}} = \
             {} - 0.000 = {}\\ \\text{{m}}",
            tex_num(Some(length), 1),
            tex_num(Some(free), 3)
        )));
        out.push(subheading("Sling Leg Angle from Vertical:"));
        out.push(equation(&format!(
            "\\text{{Angle}} = \\sin^{{-1}}\\left(\\frac{{\\text{{Distance}}/2}}\
             {{\\text{{Free Leg Length}}}}\\right) = \
             \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
            tex_num(Some(pick / 2.0), 3),
            tex_num(Some(free), 3),
            tex_deg(Some(angle), 2)
        )));
        out.push(subheading("Vertical Height (Hook to Top of Load):"));
        out.push(equation(&format!(
            "\\text{{Height}}_{{Hook-Load}} = \\text{{Free Leg Length}} \\times \
             \\cos({}) = {} \\times {} = {}\\ \\text{{m}}",
            tex_deg(Some(angle), 2),
            tex_num(Some(free), 3),
            tex_num(Some(af), 3),
            tex_num(Some(headroom), 3)
        )));
        return out.join("");
    }

    // --- The 2-leg chokes --------------------------------------------------
    if key == "two_leg_round_choke" || key == "two_leg_rect_choke" {
        let internal = result.internal_angle.unwrap_or(0.0);
        let external = result.external_angle.unwrap_or(0.0);
        let gap = result.choke_to_load.unwrap_or(0.0);
        let lead = result.lead.unwrap_or(0.0);
        let corner = result.corner_length.unwrap_or(0.0);
        let pick = result.pick_distance;
        out.push(subheading("Vertical Angle Outside Choke:"));
        out.push(equation(&format!(
            "\\text{{Choke External Angle}} = 180^\\circ - \\text{{Choke Internal Angle}} = \
             180^\\circ - {}^\\circ = {}^\\circ",
            tex_num(Some(internal), 0),
            tex_num(Some(external), 0)
        )));
        if key == "two_leg_rect_choke" {
            out.push(subheading("Choke-to-Corner Sling Length:"));
            out.push(equation(&format!(
                "\\text{{Length}} = \\frac{{\\text{{Bottom}}/2}}{{\\sin(\\theta)}} = \
                 \\frac{{{}}}{{\\sin({}^\\circ)}} = {}\\ \\text{{m}}",
                tex_num(Some(result.bottom / 2.0), 3),
                tex_num(Some(internal), 0),
                tex_num(Some(corner), 3)
            )));
        }
        out.push(subheading("Total Sling Perimeter Wrap:"));
        out.push(equation(&format!(
            "{}{}\\ \\text{{m}}",
            if key == "two_leg_round_choke" {
                format!(
                    "\\text{{Perimeter}} = \\pi \\times \\text{{Diameter}} = \
                     \\pi \\times {} = ",
                    tex_num(Some(result.diameter), 3)
                )
            } else {
                format!(
                    "\\text{{Perimeter}} = \\text{{Bottom}} + 2\\text{{Side}} + \
                     2\\text{{Corner}} = {} + 2({}) + 2({}) = ",
                    tex_num(Some(result.bottom), 3),
                    tex_num(Some(result.side), 3),
                    tex_num(Some(corner), 3)
                )
            },
            tex_num(Some(wrap), 3)
        )));
        out.push(subheading("Vertical Height (Choke to Top of Load):"));
        out.push(choke_height_equation(key, result, gap, internal, external));
        out.push(subheading("Free Leg Length (Each Sling):"));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}} = \\text{{Total Length}} - \\text{{Perimeter}} = \
             {} - {} = {}\\ \\text{{m}}",
            tex_num(Some(length), 1),
            tex_num(Some(wrap), 3),
            tex_num(Some(free), 3)
        )));
        out.push(subheading("Sling Leg Angle from Vertical:"));
        out.push(equation(&format!(
            "\\text{{Angle}} = \\sin^{{-1}}\\left(\\frac{{\\text{{Distance}}/2}}\
             {{\\text{{Free Leg Length}} + \\text{{Choke-Load}}}}\\right) = \
             \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
            tex_num(Some(pick / 2.0), 3),
            tex_num(Some(free + gap), 3),
            tex_deg(Some(angle), 2)
        )));
        out.push(subheading("Vertical Height (Hook to Choke Point):"));
        out.push(equation(&format!(
            "\\text{{Height}}_{{Hook-Choke}} = \\text{{Free Leg Length}} \\times \
             \\cos({}) = {} \\times {} = {}\\ \\text{{m}}",
            tex_deg(Some(angle), 2),
            tex_num(Some(free), 3),
            tex_num(Some(af), 3),
            tex_num(Some(headroom), 3)
        )));
        out.push(subheading("Hook-to-Contact / Lead:"));
        out.push(equation(&format!(
            "\\text{{Lead}} = \\text{{Free Leg Length}} = {}\\ \\text{{m}}",
            tex_num(Some(lead), 3)
        )));
        return out.join("");
    }

    // --- The 2-leg baskets -------------------------------------------------
    if key == "two_leg_round_basket"
        || key == "two_leg_vertical_round_basket"
        || key == "two_leg_rect_basket"
        || key == "two_leg_vertical_rect_basket"
    {
        let height_suffix: &str;
        let height_value: f64;
        let trans_numerator: f64;
        if key.ends_with("vertical_round_basket") || key == "two_leg_round_basket" {
            height_suffix = " - \\text{Radius}";
            height_value = result.diameter / 2.0;
            trans_numerator = height_value;
        } else if key.ends_with("vertical_rect_basket") {
            height_suffix = " - \\text{Side}";
            height_value = result.side;
            trans_numerator = result.bottom / 2.0;
        } else {
            height_suffix = "";
            height_value = 0.0;
            trans_numerator = result.bottom / 2.0;
        }

        let basket_radius = result.diameter / 2.0;
        let basket_bottom = result.bottom;
        let basket_side = result.side;
        let wrap_equation = if key.ends_with("round_basket") {
            format!(
                "\\text{{Wrap Length}} = \\pi \\times \\text{{Radius}} = \
                 \\pi \\times {} = {}\\ \\text{{m}}",
                tex_num(Some(basket_radius), 3),
                tex_num(Some(wrap), 3)
            )
        } else {
            format!(
                "\\text{{Wrap Length}} = \\text{{Bottom}} + 2\\text{{Side}} = \
                 {} + 2({}) = {}\\ \\text{{m}}",
                tex_num(Some(basket_bottom), 3),
                tex_num(Some(basket_side), 3),
                tex_num(Some(wrap), 3)
            )
        };

        // The height's substituted values have to cover every term of its
        // formula, including the `- Radius` (or `- Side`).
        let mut height_terms = format!(
            "{} \\times {} \\times {}",
            tex_num(Some(free), 3),
            tex_num(result.af_long, 3),
            tex_num(result.af_trans, 3)
        );
        if height_value != 0.0 {
            height_terms += &format!(" - {}", tex_num(Some(height_value), 3));
        }

        out.push(subheading("Sling Contact Perimeter Wrap:"));
        out.push(equation(&wrap_equation));
        out.push(subheading("Free Leg Length (Each Side):"));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}} = \\frac{{\\text{{Total Length}} - \\text{{Wrap Length}}}}\
             {{\\text{{Effective Legs}} / \\text{{Sling Legs}}}} = \
             \\frac{{{} - {}}}{{2}} = {}\\ \\text{{m}}",
            tex_num(Some(length), 1),
            tex_num(Some(wrap), 3),
            tex_num(Some(free), 3)
        )));
        out.push(subheading("Longitudinal Sling Angle from Vertical:"));
        out.push(equation(&format!(
            "\\text{{Angle}}_{{Long}} = \\sin^{{-1}}\\left(\\frac{{\\text{{Distance}}/2}}\
             {{\\text{{Free Leg Length}}}}\\right) = \
             \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
            tex_num(Some(result.pick_distance / 2.0), 3),
            tex_num(Some(free), 3),
            tex_deg(result.angle_long, 2)
        )));
        out.push(subheading("Transverse Basket Angle from Vertical:"));
        out.push(equation(&format!(
            "\\text{{Angle}}_{{Trans}} = \
             \\sin^{{-1}}\\left(\\frac{{\\text{{Radius or Bottom}}/2}}\
             {{\\text{{Free Leg Length}}}}\\right) = \
             \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
            tex_num(Some(trans_numerator), 3),
            tex_num(Some(free), 3),
            tex_deg(result.angle_trans, 2)
        )));
        out.push(subheading("Vertical Height (Hook to Top of Load):"));
        out.push(equation(&format!(
            "\\text{{Height}}_{{Hook-Load}} = \\text{{Free Leg Length}} \\times \
             \\cos(\\text{{Angle}}_{{Long}}) \\times \\cos(\\text{{Angle}}_{{Trans}})\
             {height_suffix} = {height_terms} = {}\\ \\text{{m}}",
            tex_num(Some(headroom), 3)
        )));
        return out.join("");
    }

    // --- The single-leg direct hook ----------------------------------------
    if key == "direct_hook" {
        out.push(subheading("Sling Contact Perimeter Wrap:"));
        out.push(equation("\\text{Wrap Length} = 0.000\\ \\text{m}"));
        out.push(subheading("Free Leg Length (Standing Lifting Line):"));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}} = \\text{{Total Length}} - \\text{{Wrap Length}} = \
             {} - 0.000 = {}\\ \\text{{m}}",
            tex_num(Some(length), 1),
            tex_num(Some(free), 3)
        )));
        out.push(subheading("Sling Leg Angle from Vertical:"));
        out.push(equation("\\text{Angle} = 0.0^\\circ"));
        out.push(subheading("Vertical Height (Hook to Top of Load):"));
        out.push(equation(&format!(
            "\\text{{Height}}_{{Hook-Load}} = \\text{{Free Leg Length}} = {}\\ \\text{{m}}",
            tex_num(Some(free), 3)
        )));
        return out.join("");
    }

    // --- The single-leg round baskets --------------------------------------
    if key == "round_basket" || key == "vertical_round_basket" {
        let radius = result.diameter / 2.0;
        out.push(subheading("Sling Contact (Perimeter Wrap):"));
        out.push(equation(&format!(
            "\\text{{Arc Length}} = \\pi \\times \\text{{Radius}} = \
             \\pi \\times {} = {}\\ \\text{{m}}",
            tex_num(Some(radius), 3),
            tex_num(Some(wrap), 3)
        )));
        out.push(subheading("Free Leg Length (Each Side):"));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}} = \\frac{{\\text{{Total Length}} - \\text{{Arc Length}}}}\
             {{2}} = \\frac{{{} - {}}}{{2}} = {}\\ \\text{{m}}",
            tex_num(Some(length), 1),
            tex_num(Some(wrap), 3),
            tex_num(Some(free), 3)
        )));
        if key == "round_basket" {
            out.push(subheading("Sling Leg Angle from Vertical:"));
            out.push(equation(&format!(
                "\\text{{Angle}} = \\sin^{{-1}}\\left(\\frac{{\\text{{Radius}}}}\
                 {{\\text{{Free Leg Length}}}}\\right) = \
                 \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
                tex_num(Some(radius), 3),
                tex_num(Some(free), 3),
                tex_deg(Some(angle), 2)
            )));
            out.push(subheading("Vertical Height (Hook to Top of Load):"));
            out.push(equation(&format!(
                "\\text{{Height}}_{{Hook-Center}} = \\text{{Free Leg Length}} \\times \
                 \\cos({}) - \\text{{Radius}} = {} \\times {} - {} = {}\\ \\text{{m}}",
                tex_deg(Some(angle), 2),
                tex_num(Some(free), 3),
                tex_num(Some(af), 3),
                tex_num(Some(radius), 3),
                tex_num(Some(headroom), 3)
            )));
        } else {
            out.push(subheading("Sling Leg Angle from Vertical:"));
            out.push(equation(
                "\\text{Angle} = 0.0^\\circ\\ \\text{(legs go straight up parallel \
                 to a spreader bar)}",
            ));
            out.push(subheading("Vertical Height (Hook to Top of Load):"));
            out.push(equation(&format!(
                "\\text{{Height}}_{{Hook-Load}} = \\text{{Free Leg Length}} - \
                 \\text{{Radius}} = {} - {} = {}\\ \\text{{m}}",
                tex_num(Some(free), 3),
                tex_num(Some(radius), 3),
                tex_num(Some(headroom), 3)
            )));
        }
        return out.join("");
    }

    // --- The single-leg rectangular baskets --------------------------------
    if key == "rect_basket" || key == "vertical_rect_basket" {
        let side = result.side;
        let bottom = result.bottom;
        out.push(subheading("Sling Contact Perimeter Wrap:"));
        out.push(equation(&format!(
            "\\text{{Wrap Length}} = \\text{{Bottom}} + 2\\text{{Side}} = \
             {} + 2({}) = {}\\ \\text{{m}}",
            tex_num(Some(bottom), 3),
            tex_num(Some(side), 3),
            tex_num(Some(wrap), 3)
        )));
        out.push(subheading("Free Leg Length (Each Side):"));
        out.push(equation(&format!(
            "\\text{{Free Leg Length}} = \\frac{{\\text{{Total Length}} - \\text{{Wrap Length}}}}\
             {{2}} = \\frac{{{} - {}}}{{2}} = {}\\ \\text{{m}}",
            tex_num(Some(length), 1),
            tex_num(Some(wrap), 3),
            tex_num(Some(free), 3)
        )));
        if key == "rect_basket" {
            out.push(subheading("Sling Leg Angle from Vertical:"));
            out.push(equation(&format!(
                "\\text{{Angle}} = \\sin^{{-1}}\\left(\\frac{{\\text{{Bottom}}/2}}\
                 {{\\text{{Free Leg Length}}}}\\right) = \
                 \\sin^{{-1}}\\left(\\frac{{{}}}{{{}}}\\right) = {}",
                tex_num(Some(bottom / 2.0), 3),
                tex_num(Some(free), 3),
                tex_deg(Some(angle), 2)
            )));
            out.push(subheading("Vertical Height (Hook to Top of Load):"));
            out.push(equation(&format!(
                "\\text{{Height}}_{{Hook-Load}} = \\text{{Free Leg Length}} \\times \
                 \\cos({}) = {} \\times {} = {}\\ \\text{{m}}",
                tex_deg(Some(angle), 2),
                tex_num(Some(free), 3),
                tex_num(Some(af), 3),
                tex_num(Some(headroom), 3)
            )));
        } else {
            out.push(subheading("Sling Leg Angle from Vertical:"));
            out.push(equation(
                "\\text{Angle} = 0.0^\\circ\\ \\text{(legs go straight up parallel \
                 to a spreader bar)}",
            ));
            out.push(subheading("Vertical Height (Hook to Top of Load):"));
            out.push(equation(&format!(
                "\\text{{Height}}_{{Hook-Load}} = \\text{{Free Leg Length}} - \
                 \\text{{Side}} = {} - {} = {}\\ \\text{{m}}",
                tex_num(Some(free), 3),
                tex_num(Some(side), 3),
                tex_num(Some(headroom), 3)
            )));
        }
        return out.join("");
    }

    // --- The single-leg round choke ----------------------------------------
    if key == "round_choke" {
        let radius = result.diameter / 2.0;
        let internal = result.internal_angle.unwrap_or(0.0);
        let external = result.external_angle.unwrap_or(0.0);
        let lead = result.lead.unwrap_or(0.0);
        let gap = result.choke_to_load;
        out.push(subheading("Vertical Angle Outside Choke:"));
        out.push(equation(&format!(
            "\\text{{Choke External Angle}} = 180^\\circ - \\text{{Choke Internal Angle}} = \
             180^\\circ - {}^\\circ = {}^\\circ",
            tex_num(Some(internal), 0),
            tex_num(Some(external), 0)
        )));
        out.push(subheading("Total Sling Perimeter Wrap:"));
        out.push(equation(&format!(
            "\\text{{Perimeter}} = \\pi \\times \\text{{Diameter}} = \\pi \\times \
             {} = {}\\ \\text{{m}}",
            tex_num(Some(result.diameter), 3),
            tex_num(Some(wrap), 3)
        )));
        if external < 90.0 {
            out.push(subheading("Operational Rule Declared:"));
            out.push(p(
                "As noted by the inspector/supervisor, if the angle of choke falls \
                 below 90 degrees, the sling wraps the load tightly under tension. \
                 The choke point is forced flush against the cylinder surface, \
                 meaning there is zero floating gap or air space above the cargo.",
                "",
            ));
            out.push(subheading("Vertical Height (Choke to Top of Load):"));
            out.push(equation("\\text{Height}_{Choke-Load} = 0.000\\ \\text{m}"));
        } else {
            let y_value = if internal != 0.0 {
                radius / internal.to_radians().sin()
            } else {
                radius
            };
            out.push(subheading("Vertical Height (Choke to Top of Load):"));
            out.push(equation(&format!(
                "Y = \\frac{{\\text{{Radius}}}}{{\\sin(\\text{{Choke Internal Angle}})}} = \
                 \\frac{{{}}}{{\\sin({}^\\circ)}} = {}\\ \\text{{m}}",
                tex_num(Some(radius), 3),
                tex_num(Some(internal), 0),
                tex_num(Some(y_value), 3)
            )));
            out.push(equation(&format!(
                "\\text{{Height}}_{{Choke-Load}} = Y - \\text{{Radius}} = \
                 {} - {} = {}\\ \\text{{m}}",
                tex_num(Some(y_value), 3),
                tex_num(Some(radius), 3),
                tex_num(gap, 3)
            )));
        }
        out.push(subheading("Vertical Height (Hook to Choke):"));
        out.push(equation(&format!(
            "\\text{{Height}}_{{Hook-Choke}} = \\text{{Total Length}} - \\text{{Perimeter}} = \
             {} - {} = {}\\ \\text{{m}}",
            tex_num(Some(length), 1),
            tex_num(Some(wrap), 3),
            tex_num(Some(lead), 3)
        )));
        return out.join("");
    }

    // --- The rectangular choke (the remaining arrangement) ------------------
    let side = result.side;
    let bottom = result.bottom;
    let internal = result.internal_angle.unwrap_or(0.0);
    let external = result.external_angle.unwrap_or(0.0);
    let corner = result.corner_length.unwrap_or(0.0);
    let lead = result.lead.unwrap_or(0.0);
    let gap = result.choke_to_load.unwrap_or(0.0);
    out.push(subheading("Vertical Angle Outside Choke:"));
    out.push(equation(&format!(
        "\\text{{Choke External Angle}} = 180^\\circ - \\text{{Choke Internal Angle}} = \
         180^\\circ - {}^\\circ = {}^\\circ",
        tex_num(Some(internal), 0),
        tex_num(Some(external), 0)
    )));
    out.push(subheading("Choke-to-Corner Sling Length:"));
    out.push(equation(&format!(
        "\\text{{Length}} = \\frac{{\\text{{Bottom}}/2}}{{\\sin(\\theta)}} = \
         \\frac{{{}}}{{\\sin({}^\\circ)}} = {}\\ \\text{{m}}",
        tex_num(Some(bottom / 2.0), 3),
        tex_num(Some(internal), 0),
        tex_num(Some(corner), 3)
    )));
    out.push(subheading("Total Sling Perimeter Wrap:"));
    out.push(equation(&format!(
        "\\text{{Perimeter}} = \\text{{Bottom}} + 2\\text{{Side}} + \
         2\\text{{Choke-to-Corner}} = {} + 2({}) + 2({}) = {}\\ \\text{{m}}",
        tex_num(Some(bottom), 3),
        tex_num(Some(side), 3),
        tex_num(Some(corner), 3),
        tex_num(Some(wrap), 3)
    )));
    out.push(subheading("Vertical Height (Choke to Top of Load):"));
    out.push(equation(&format!(
        "\\text{{Height}}_{{Choke-Load}} = \\frac{{\\text{{Bottom}}/2}}{{\\tan(\\theta)}} = \
         \\frac{{{}}}{{\\tan({}^\\circ)}} = {}\\ \\text{{m}}",
        tex_num(Some(bottom / 2.0), 3),
        tex_num(Some(internal), 0),
        tex_num(Some(gap), 3)
    )));
    out.push(subheading("Vertical Height (Hook to Choke):"));
    out.push(equation(&format!(
        "\\text{{Height}}_{{Hook-Choke}} = \\text{{Total Length}} - \\text{{Perimeter}} = \
         {} - {} = {}\\ \\text{{m}}",
        tex_num(Some(length), 1),
        tex_num(Some(wrap), 3),
        tex_num(Some(lead), 3)
    )));
    out.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::next_section_number;
    use crate::sling::{hitch_spec, solve_defaults};

    fn geometry(hitch: &str) -> String {
        let result = solve_defaults(hitch, 2000.0).unwrap();
        geometry_section(&result, hitch_spec(&result.hitch).unwrap())
    }

    fn inputs(hitch: &str) -> String {
        let result = solve_defaults(hitch, 2000.0).unwrap();
        section_inputs(&result, hitch_spec(&result.hitch).unwrap())
    }

    #[test]
    fn the_input_section_matches_the_reference() {
        let html = inputs("direct_hook");
        for expected in [
            "<h2>1. LIFT INPUT PARAMETERS</h2>",
            "<tr><th>Sling Configuration</th><td>1-Leg Sling Direct hook Hitch (1 Effective Leg)</td></tr>",
            "<tr><th>Sling assembly count</th><td>1 continuous sling</td></tr>",
            "<tr><th>Sling Total Length</th><td>7.0 m</td></tr>",
            "<tr><th>Load Dimensions</th><td>Not required for a direct vertical hitch</td></tr>",
            "Load Weight is the gross load at hook. The 75% allowable window applies to the crane load chart only and is shown in the crane section.",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn a_nested_input_section_names_its_piles_and_its_second_sling() {
        let html = inputs("two_leg_nested_3leg");
        for expected in [
            "<tr><th>Sling Configuration</th><td>3-Leg Sling in Each Leg of 2-Leg Sling Hitch (6 Effective Legs)</td></tr>",
            "<tr><th>Sling assembly count</th><td>2-sling bridle set</td></tr>",
            "<tr><th>No of Piles</th><td>3</td></tr>",
            "<tr><th>Second Sling Total Length</th><td>0.600 m</td></tr>",
            "<tr><th>Secondary Pick Spacing</th><td>0.200 m</td></tr>",
            "<tr><th>Sling Legs per Pile</th><td>2</td></tr>",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn a_two_leg_nested_system_has_no_pile_row() {
        let html = inputs("two_leg_nested_2leg");
        assert!(html.contains("<tr><th>Second Sling Total Length</th><td>0.600 m</td></tr>"));
        assert!(!html.contains("No of Piles"));
        assert!(!html.contains("Sling Legs per Pile"));
    }

    #[test]
    fn the_direct_hook_geometry_matches_the_reference() {
        let html = geometry("direct_hook");
        for expected in [
            "<h2>2. GEOMETRIC &amp; HEADROOM CALCULATIONS</h2>",
            "\\[\\text{Wrap Length} = 0.000\\ \\text{m}\\]",
            "\\[\\text{Free Leg Length} = \\text{Total Length} - \\text{Wrap Length} = 7.0 - 0.000 = 7.000\\ \\text{m}\\]",
            "\\[\\text{Angle} = 0.0^\\circ\\]",
            "\\[\\text{Height}_{Hook-Load} = \\text{Free Leg Length} = 7.000\\ \\text{m}\\]",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_round_basket_geometry_matches_the_reference() {
        let html = geometry("round_basket");
        for expected in [
            "\\[\\text{Arc Length} = \\pi \\times \\text{Radius} = \\pi \\times 0.500 = 1.571\\ \\text{m}\\]",
            "\\[\\text{Angle} = \\sin^{-1}\\left(\\frac{\\text{Radius}}{\\text{Free Leg Length}}\\right) = \\sin^{-1}\\left(\\frac{0.500}{2.715}\\right) = 10.61^\\circ\\]",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_basket_height_carries_every_term_of_its_formula() {
        let html = geometry("round_basket");
        assert!(html.contains(
            "\\[\\text{Height}_{Hook-Center} = \\text{Free Leg Length} \\times \
             \\cos(10.61^\\circ) - \\text{Radius} = 2.715 \\times 0.983 - 0.500 = 2.168\\ \\text{m}\\]"
        ));
    }

    #[test]
    fn the_round_choke_height_states_its_derivation() {
        let html = geometry("round_choke");
        for expected in [
            "\\[Y = \\frac{\\text{Radius}}{\\sin(\\text{Choke Internal Angle})} = \\frac{0.500}{\\sin(60^\\circ)} = 0.577\\ \\text{m}\\]",
            "\\[\\text{Height}_{Choke-Load} = Y - \\text{Radius} = 0.577 - 0.500 = 0.077\\ \\text{m}\\]",
            "\\[\\text{Height}_{Hook-Choke} = \\text{Total Length} - \\text{Perimeter} = 7.0 - 3.142 = 3.858\\ \\text{m}\\]",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_two_leg_direct_geometry_matches_the_reference() {
        let html = geometry("two_leg_direct");
        for expected in [
            "\\[\\text{Angle} = \\sin^{-1}\\left(\\frac{\\text{Distance}/2}{\\text{Free Leg Length}}\\right) = \\sin^{-1}\\left(\\frac{2.500}{7.000}\\right) = 20.92^\\circ\\]",
            "\\[\\text{Height}_{Hook-Load} = \\text{Free Leg Length} \\times \\cos(20.92^\\circ) = 7.000 \\times 0.934 = 6.538\\ \\text{m}\\]",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn the_three_leg_bridle_geometry_matches_the_reference() {
        let html = geometry("three_leg_direct");
        assert!(html.contains(
            "\\[\\text{Radius Spread} = \\frac{\\text{Pick Distance}}{\\sqrt{3}} = \\frac{5.400}{\\sqrt{3}} = 3.118\\ \\text{m}\\]"
        ));
        assert!(html.contains("Sling Contact Perimeter Wrap:"));
    }

    #[test]
    fn the_nested_geometry_matches_the_reference() {
        let html = geometry("two_leg_nested_3leg");
        for expected in [
            "\\[\\text{Free Leg Length}_1 = 7.000\\ \\text{m}\\]",
            "\\[\\text{Free Leg Length}_2 = 0.600\\ \\text{m}\\]",
            "\\[\\text{Angle}_{\\text{Trans Outer}} = \\sin^{-1}\\left(\\frac{\\text{Offset}}{\\text{Free Leg Length}_2}\\right) = \\sin^{-1}\\left(\\frac{0.200}{0.600}\\right) = 19.47^\\circ\\]",
            "\\[\\text{Angle}_{\\text{Trans Mid}} = \\sin^{-1}\\left(\\frac{\\text{Offset}}{\\text{Free Leg Length}_2}\\right) = \\sin^{-1}\\left(\\frac{0.000}{0.600}\\right) = 0.00^\\circ\\]",
            "Transverse Sling Leg Angle from Vertical (Outer Secondary Legs 1 &amp; 3):",
            "Transverse Sling Leg Angle from Vertical (Middle Secondary Leg 2):",
            "\\[\\text{Height}_{Sling2} = \\text{Free Leg Length}_2 \\times \\cos(\\text{Angle}_{Trans\\,Outer}) = 0.600 \\times 0.943 = 0.566\\ \\text{m}\\]",
            "\\[\\text{Level Difference} = \\text{Free Leg Length}_2 \\times \\left(\\cos(\\text{Angle}_{Trans\\,Mid}) - \\cos(\\text{Angle}_{Trans\\,Outer})\\right) = 0.600 \\times (1.000 - 0.943) = 0.034\\ \\text{m}\\]",
            "\\[\\text{Total Combined} = \\text{Free Leg Length}_1 + \\text{Height}_{Sling2} = 7.000 + 0.566 = 7.566\\ \\text{m}\\]",
            "Total Vertical Height (Hook to Top of RC Piles Headroom):",
            "\\[\\text{Height}_{Hook-Load} = \\text{Total Combined} \\times \\cos(20.91^\\circ) = 7.566 \\times 0.934 = 7.067\\ \\text{m}\\]",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn every_arrangement_produces_a_geometry_section_of_its_own() {
        for spec in crate::sling::HITCH_SPECS {
            let html = geometry(spec.key);
            assert!(
                html.contains("<h2>2. GEOMETRIC &amp; HEADROOM CALCULATIONS</h2>"),
                "{} heading",
                spec.key
            );
            assert!(
                html.contains("<div class=\"equation\">\\["),
                "{} equations",
                spec.key
            );
            assert!(html.contains("Height}_{Hook"), "{} headroom", spec.key);
        }
    }

    #[test]
    fn the_section_number_of_the_next_block_is_counted_not_assumed() {
        assert_eq!(next_section_number(&[]), 1);
        assert_eq!(
            next_section_number(&["<h2>1. LIFT INPUT PARAMETERS</h2>".to_string()]),
            2
        );
        assert_eq!(
            next_section_number(&[
                "<h2>1. LIFT INPUT PARAMETERS</h2>".to_string(),
                "<h2>2. GEOMETRIC &amp; HEADROOM CALCULATIONS</h2>".to_string()
            ]),
            3
        );
    }
}
