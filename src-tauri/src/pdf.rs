//! The printed plan's footer.
//!
//! The report is printed by the app's own web view, and WebKitGTK does not draw
//! CSS `@page` margin boxes, so the footer cannot be part of the HTML there. It
//! is written onto the finished PDF instead: the plan reference on the left, the
//! credit line in the centre, and `n / total` on the right, on every page.
//!
//! Drawing it here rather than in the renderer also makes it independent of the
//! renderer's version: older Chromium builds silently drop margin boxes, which
//! would have taken the reference and page counter off a signed plan.
//!
//! The footer text is operator input — a plan reference may be a name — so the
//! font is embedded rather than relying on a PDF base font: Liberation Serif
//! covers Latin with diacritics, Greek, Cyrillic and Vietnamese, and it is the
//! face the Chromium build used, so a plan printed by either looks the same.

use std::collections::BTreeMap;
use std::path::Path;

use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use ttf_parser::Face;

/// The footer's typeface, embedded in the binary so the export needs nothing
/// installed on the machine.
const FONT: &[u8] = include_bytes!("../assets/fonts/LiberationSerif-Regular.ttf");
const FONT_NAME: &str = "LiberationSerif";
/// The font resource the footer is drawn with, as a PDF name.
const FONT_KEY: &str = "LPCFooter";
/// 25.4 mm: the report's own 1-inch content margin.
pub const MARGIN_PT: f32 = report_margin_pt();
/// The baseline, 11.6 mm up from the page's bottom edge — where the Chromium
/// build put the footer, so the plan keeps the same look.
const BASELINE_PT: f32 = 33.0;
/// 6 pt, the same size the report's margin boxes used.
const SIZE_PT: f32 = 6.0;
/// `#555`, the footer's grey.
const GREY: f32 = 0.333;

/// The report's page margin, converted from the millimetres its CSS declares to
/// the points this module draws in. One number, so the two cannot drift.
const fn report_margin_pt() -> f32 {
    (crate::report::PRINT_MARGIN_MM as f32) * 72.0 / 25.4
}

/// The text that runs along the bottom of every page.
#[derive(Debug, Clone)]
pub struct Footer {
    /// The plan's own reference, or "No reference" when none was entered.
    pub reference: String,
    /// The credit line printed in the centre of the page.
    pub center: String,
}

/// Draw the footer onto every page of the PDF at `path`, in place.
pub fn stamp(path: &Path, footer: &Footer) -> Result<(), String> {
    let mut doc = Document::load(path)
        .map_err(|error| format!("The saved PDF could not be reopened for its footer: {error}"))?;
    let pages: Vec<ObjectId> = doc.get_pages().into_values().collect();
    if pages.is_empty() {
        return Err("The saved PDF has no pages, so the footer could not be written.".to_string());
    }
    let total = pages.len();

    // What each page's footer says, before anything is written.
    let mut lines: Vec<(String, String, String)> = Vec::with_capacity(total);
    let mut bounds: Vec<(f32, f32, f32)> = Vec::with_capacity(total);
    for (index, page) in pages.iter().enumerate() {
        let (left, right) = page_bounds(&doc, *page)?;
        bounds.push((left, right, baseline(&doc, *page)?));
        lines.push((
            footer.reference.clone(),
            footer.center.clone(),
            format!("{} / {}", index + 1, total),
        ));
    }

    let font = embed_font(&mut doc, lines.iter().flat_map(|(reference, center, counter)| {
        [reference.as_str(), center.as_str(), counter.as_str()]
    }))?;

    for ((page, (left, right, baseline)), (reference, center, counter)) in
        pages.iter().zip(bounds).zip(lines)
    {
        attach_font(&mut doc, *page, font)?;
        prepend_content(
            &mut doc,
            *page,
            footer_content(left, right, baseline, &reference, &center, &counter),
        )?;
    }
    doc.save(path)
        .map_err(|error| format!("The PDF footer could not be written: {error}"))?;
    Ok(())
}

/// The footer as PDF operators, drawn over the page's own content.
fn footer_content(
    left: f32,
    right: f32,
    baseline: f32,
    reference: &str,
    center: &str,
    counter: &str,
) -> String {
    let mut ops = String::new();
    ops.push_str("q\n");
    ops.push_str(&format!(
        "BT /{FONT_KEY} {SIZE_PT} Tf {GREY} {GREY} {GREY} rg\n"
    ));
    let centre_x = (left + right) / 2.0 - text_width(center, SIZE_PT) / 2.0;
    let right_x = right - text_width(counter, SIZE_PT);
    for (x, text) in [(left, reference), (centre_x, center), (right_x, counter)] {
        if text.trim().is_empty() {
            continue;
        }
        ops.push_str(&format!(
            "1 0 0 1 {x:.2} {baseline:.2} Tm <{}> Tj\n",
            glyph_ids(text)
                .iter()
                .map(|glyph| format!("{glyph:04X}"))
                .collect::<String>()
        ));
    }
    ops.push_str("ET\nQ\n");
    ops
}

/// The glyph the embedded font has for a character, or `.notdef` when it has
/// none — the same box the report's own font stack would show.
fn glyph(child: char) -> u16 {
    static FACE: std::sync::OnceLock<Option<Face<'static>>> = std::sync::OnceLock::new();
    let face = FACE.get_or_init(|| Face::parse(FONT, 0).ok());
    face.as_ref()
        .and_then(|face| face.glyph_index(child))
        .map(|glyph| glyph.0)
        .unwrap_or(0)
}

fn glyph_ids(text: &str) -> Vec<u16> {
    text.chars().map(glyph).collect()
}

/// The drawn width of `text` at `size`, from the font's own metrics.
fn text_width(text: &str, size: f32) -> f32 {
    static METRICS: std::sync::OnceLock<Option<(u16, Vec<u16>)>> = std::sync::OnceLock::new();
    let Some((units_per_em, widths)) = METRICS
        .get_or_init(|| {
            let face = Face::parse(FONT, 0).ok()?;
            Some((face.units_per_em(), (0..face.number_of_glyphs()).map(|glyph| face.glyph_hor_advance(ttf_parser::GlyphId(glyph)).unwrap_or(0)).collect()))
        })
        .as_ref()
    else {
        return 0.0;
    };
    let advance: u32 = glyph_ids(text)
        .iter()
        .map(|glyph| widths.get(*glyph as usize).copied().unwrap_or(0) as u32)
        .sum();
    advance as f32 / *units_per_em as f32 * size
}

/// Embed the footer's font as a Type0 font with the glyphs the footer uses, so
/// names outside Latin-1 print as they were typed.
fn embed_font<'a>(
    doc: &mut Document,
    texts: impl Iterator<Item = &'a str>,
) -> Result<ObjectId, String> {
    let face = Face::parse(FONT, 0)
        .map_err(|error| format!("The footer font could not be read: {error}"))?;
    let units = face.units_per_em() as f32;
    let scale = |value: i16| (value as f32 / units * 1000.0).round() as i64;

    // Every glyph the footer needs, with its width and the character it stands for.
    let mut used: BTreeMap<u16, (u16, char)> = BTreeMap::new();
    for text in texts {
        for character in text.chars() {
            let id = glyph(character);
            used.entry(id).or_insert((
                face.glyph_hor_advance(ttf_parser::GlyphId(id)).unwrap_or(0),
                character,
            ));
        }
    }

    let mut widths: Vec<Object> = Vec::new();
    for (id, (advance, _)) in &used {
        widths.push(Object::Integer(*id as i64));
        widths.push(Object::Array(vec![
            Object::Integer((*advance as f32 / units * 1000.0).round() as i64),
        ]));
    }

    let mut to_unicode = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
         /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
         /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
         1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let entries: Vec<(&u16, &(u16, char))> = used.iter().collect();
    for chunk in entries.chunks(100) {
        to_unicode.push_str(&format!("{} beginbfchar\n", chunk.len()));
        for (id, (_, character)) in chunk {
            to_unicode.push_str(&format!("<{id:04X}> <{}>\n", utf16_hex(*character)));
        }
        to_unicode.push_str("endbfchar\n");
    }
    to_unicode.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");

    let bbox = face.global_bounding_box();
    let file = font_file(doc);
    let descriptor = doc.add_object(dictionary! {
        "Type" => "FontDescriptor",
        "FontName" => FONT_NAME,
        "Flags" => 4,
        "FontBBox" => vec![
            Object::Integer(scale(bbox.x_min)),
            Object::Integer(scale(bbox.y_min)),
            Object::Integer(scale(bbox.x_max)),
            Object::Integer(scale(bbox.y_max)),
        ],
        "ItalicAngle" => 0,
        "Ascent" => scale(face.ascender()),
        "Descent" => scale(face.descender()),
        "CapHeight" => scale(face.capital_height().unwrap_or(face.ascender())),
        "StemV" => 80,
        "FontFile2" => file,
    });

    let descendant = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "CIDFontType2",
        "BaseFont" => FONT_NAME,
        "CIDSystemInfo" => dictionary! {
            "Registry" => Object::string_literal("Adobe"),
            "Ordering" => Object::string_literal("Identity"),
            "Supplement" => 0,
        },
        "FontDescriptor" => descriptor,
        "CIDToGIDMap" => "Identity",
        "DW" => 1000,
        "W" => Object::Array(widths),
    });

    let to_unicode = doc.add_object({
        let mut stream = Stream::new(Dictionary::new(), to_unicode.into_bytes());
        let _ = stream.compress();
        stream
    });

    Ok(doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type0",
        "BaseFont" => FONT_NAME,
        "Encoding" => "Identity-H",
        "DescendantFonts" => vec![Object::Reference(descendant)],
        "ToUnicode" => to_unicode,
    }))
}

/// A character as UTF-16BE hex, which is what a `bfchar` entry takes. A BMP
/// character is one unit; anything above it is a surrogate pair.
fn utf16_hex(character: char) -> String {
    let mut units = [0u16; 2];
    character
        .encode_utf16(&mut units)
        .iter()
        .map(|unit| format!("{unit:04X}"))
        .collect()
}

fn font_file(doc: &mut Document) -> ObjectId {
    let mut stream = Stream::new(
        dictionary! {
            "Length1" => FONT.len() as i64,
        },
        FONT.to_vec(),
    );
    let _ = stream.compress();
    doc.add_object(stream)
}

/// The page's horizontal content edges, from its MediaBox, and its baseline.
///
/// A rotated page is refused rather than stamped in the wrong place: the report
/// is never printed rotated, and a footer across the side of a page would be
/// worse than a failed save.
fn page_bounds(doc: &Document, page: ObjectId) -> Result<(f32, f32), String> {
    let (llx, urx) = media_box(doc, page)?;
    Ok((llx + MARGIN_PT, urx - MARGIN_PT))
}

fn baseline(doc: &Document, page: ObjectId) -> Result<f32, String> {
    let mut current = page;
    for _ in 0..8 {
        let dict = read_dict(doc, current, "page")?;
        if let Ok(rotate) = dict.get(b"Rotate") {
            let degrees = rotate.as_i64().unwrap_or(0).rem_euclid(360);
            if degrees != 0 {
                return Err(format!(
                    "The saved PDF's pages are rotated by {degrees}°, so the footer \
                     would not sit on the page edge. The plan was not saved."
                ));
            }
        }
        match dict.get(b"MediaBox") {
            Ok(_) => return Ok(BASELINE_PT),
            _ => {
                current = dict
                    .get(b"Parent")
                    .and_then(Object::as_reference)
                    .map_err(|_| {
                        "The saved PDF's page size could not be found, so the footer was not written."
                            .to_string()
                    })?;
            }
        }
    }
    Err("The saved PDF's page size could not be found, so the footer was not written.".to_string())
}

fn media_box(doc: &Document, page: ObjectId) -> Result<(f32, f32), String> {
    let mut current = page;
    for _ in 0..8 {
        let dict = read_dict(doc, current, "page")?;
        match dict.get(b"MediaBox") {
            Ok(Object::Array(box_)) if box_.len() == 4 => {
                let value = |index: usize| -> f32 {
                    match &box_[index] {
                        Object::Integer(number) => *number as f32,
                        Object::Real(number) => *number,
                        _ => 0.0,
                    }
                };
                return Ok((value(0), value(2)));
            }
            _ => {
                current = dict
                    .get(b"Parent")
                    .and_then(Object::as_reference)
                    .map_err(|_| {
                        "The saved PDF's page size could not be found, so the footer was not written."
                            .to_string()
                    })?;
            }
        }
    }
    Err("The saved PDF's page size could not be found, so the footer was not written.".to_string())
}

fn read_dict<'a>(doc: &'a Document, id: ObjectId, what: &str) -> Result<&'a Dictionary, String> {
    doc.get_object(id)
        .and_then(|object| object.as_dict())
        .map_err(|error| format!("The saved PDF's {what} could not be read: {error}"))
}

/// Where the page's `/Font` dictionary actually lives, so the font can be added
/// to that dictionary and nothing else. A page may inherit its resources from
/// the page tree, and the `/Font` dictionary may itself be an indirect object.
fn font_home(doc: &Document, page: ObjectId) -> Result<FontHome, String> {
    let mut current = page;
    for _ in 0..8 {
        let dict = read_dict(doc, current, "page")?;
        match dict.get(b"Resources") {
            Ok(Object::Reference(id)) => {
                let resources = read_dict(doc, *id, "page resources")?;
                return match resources.get(b"Font") {
                    Ok(Object::Reference(font)) => Ok(FontHome::Dict(*font)),
                    _ => Ok(FontHome::Resources(*id)),
                };
            }
            Ok(Object::Dictionary(_)) => {
                let resources = dict
                    .get(b"Resources")
                    .and_then(Object::as_dict)
                    .map_err(|error| format!("The saved PDF's page resources could not be read: {error}"))?;
                return match resources.get(b"Font") {
                    Ok(Object::Reference(font)) => Ok(FontHome::Dict(*font)),
                    _ => Ok(FontHome::PageResources(current)),
                };
            }
            _ => {
                current = dict
                    .get(b"Parent")
                    .and_then(Object::as_reference)
                    .map_err(|_| {
                        "The saved PDF's page has no resources, so the footer was not written."
                            .to_string()
                    })?;
            }
        }
    }
    Err("The saved PDF's page resources could not be found, so the footer was not written.".to_string())
}

enum FontHome {
    /// The page has its own `/Resources`, inline.
    PageResources(ObjectId),
    /// The page's `/Resources` is an indirect object.
    Resources(ObjectId),
    /// The page's `/Font` dictionary is an indirect object.
    Dict(ObjectId),
}

/// Make the footer font reachable from a page's content, following the page's
/// inherited resources rather than replacing them.
fn attach_font(doc: &mut Document, page: ObjectId, font: ObjectId) -> Result<(), String> {
    match font_home(doc, page)? {
        FontHome::Dict(id) => {
            let dict = doc
                .get_object_mut(id)
                .and_then(|object| object.as_dict_mut())
                .map_err(|error| format!("The saved PDF's font dictionary could not be read: {error}"))?;
            dict.set(FONT_KEY, font);
            Ok(())
        }
        FontHome::PageResources(owner) => {
            let dict = doc
                .get_object_mut(owner)
                .and_then(|object| object.as_dict_mut())
                .map_err(|error| format!("The saved PDF's page resources could not be read: {error}"))?;
            if !dict.has(b"Resources") {
                dict.set("Resources", Dictionary::new());
            }
            let resources = dict
                .get_mut(b"Resources")
                .and_then(|object| object.as_dict_mut())
                .map_err(|error| format!("The saved PDF's page resources could not be read: {error}"))?;
            insert_font_into(resources, font)
        }
        FontHome::Resources(resources) => {
            let dict = doc
                .get_object_mut(resources)
                .and_then(|object| object.as_dict_mut())
                .map_err(|error| format!("The saved PDF's page resources could not be read: {error}"))?;
            insert_font_into(dict, font)
        }
    }
}

/// Add the font to a resources dictionary that already holds the page's fonts.
fn insert_font_into(resources: &mut Dictionary, font: ObjectId) -> Result<(), String> {
    match resources.get(b"Font") {
        Ok(Object::Dictionary(_)) => {
            let fonts = resources
                .get_mut(b"Font")
                .and_then(|object| object.as_dict_mut())
                .map_err(|error| format!("The saved PDF's fonts could not be read: {error}"))?;
            fonts.set(FONT_KEY, font);
        }
        _ => {
            resources.set("Font", dictionary! { FONT_KEY => font });
        }
    }
    Ok(())
}

/// Put the footer's operators ahead of the page's own content, so they are drawn
/// in the page's default user space. Skia-built PDFs (WebKit and Chromium both)
/// leave a vertical flip in the graphics state when their content ends, and an
/// appended stream would inherit it and print the footer upside down at the top
/// of the page.
fn prepend_content(doc: &mut Document, page: ObjectId, content: String) -> Result<(), String> {
    let stream = doc.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
    let contents = {
        let dict = doc
            .get_object(page)
            .and_then(|object| object.as_dict())
            .map_err(|error| format!("The saved PDF's page could not be read: {error}"))?;
        dict.get(b"Contents").cloned().ok()
    };
    // A page's contents may be one stream, an array of streams, or a reference
    // to an array object; all three have to keep their existing pages.
    let mut streams = match contents {
        Some(Object::Reference(id)) => match doc.get_object(id).and_then(Object::as_array) {
            Ok(items) => items.clone(),
            Err(_) => vec![Object::Reference(id)],
        },
        Some(Object::Array(items)) => items,
        Some(other) => vec![other],
        None => Vec::new(),
    };
    streams.insert(0, Object::Reference(stream));

    let page = doc
        .get_object_mut(page)
        .and_then(|object| object.as_dict_mut())
        .map_err(|error| format!("The saved PDF's page could not be read: {error}"))?;
    page.set("Contents", Object::Array(streams));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A two-page PDF shaped like the renderer's own output: page content is a
    /// compressed stream and the resources are indirect.
    fn document() -> (Document, Vec<ObjectId>) {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let mut pages = Vec::new();
        for _ in 0..2 {
            let font = doc.add_object(dictionary! {
                "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Times-Roman",
            });
            let resources = doc.add_object(dictionary! {
                "Font" => dictionary! { "F1" => font },
            });
            let mut content = Stream::new(Dictionary::new(), b"0 0 m 10 10 l S".to_vec());
            let _ = content.compress();
            let content = doc.add_object(content);
            pages.push(doc.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
                "Resources" => resources,
                "Contents" => content,
            }));
        }
        doc.objects.insert(pages_id, Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => pages.iter().map(|page| Object::Reference(*page)).collect::<Vec<_>>(),
            "Count" => pages.len() as i64,
        }));
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        (doc, pages)
    }

    fn save(doc: &mut Document, name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("lpc-footer-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        doc.save(&path).unwrap();
        path
    }

    /// The page's own content streams, in order, as text.
    fn streams(doc: &Document, page: ObjectId) -> Vec<String> {
        let dict = doc.get_object(page).unwrap().as_dict().unwrap();
        let items: Vec<Object> = match dict.get(b"Contents").unwrap() {
            Object::Array(items) => items.clone(),
            object => vec![object.clone()],
        };
        items
            .iter()
            .filter_map(|object| object.as_reference().ok())
            .filter_map(|id| doc.get_object(id).ok().and_then(|object| object.as_stream().ok()))
            .map(|stream| String::from_utf8_lossy(&stream.content).to_string())
            .collect()
    }

    #[test]
    fn every_page_carries_the_reference_credit_and_counter() {
        let (mut doc, pages) = document();
        let path = save(&mut doc, "plan.pdf");

        stamp(&path, &Footer {
            reference: "LP-2026-014".to_string(),
            center: "Prepared by A. Rigger".to_string(),
        })
        .unwrap();

        let doc = Document::load(&path).unwrap();
        for (index, page) in pages.iter().enumerate() {
            let streams = streams(&doc, *page);
            assert_eq!(streams.len(), 2, "page {} gains exactly one stream", index + 1);
            let footer = &streams[0];
            assert!(footer.contains("BT /LPCFooter 6 Tf"), "{footer}");
            assert_eq!(footer.matches(" Tj").count(), 3, "{footer}");
            assert!(footer.contains("ET"), "{footer}");
            // The page's own content is kept, and after the footer.
            assert!(streams[1].contains("0 0 m 10 10 l S"));
            // The counter is the page's own number, of the whole document.
            let counter: String = format!("{}/{}", index + 1, pages.len());
            let expected: String = format!("{} / {}", index + 1, pages.len());
            assert!(footer.contains(&glyph_hex(&expected)), "page {} wants {counter}: {footer}", index + 1);
        }
    }

    fn glyph_hex(text: &str) -> String {
        glyph_ids(text)
            .iter()
            .map(|glyph| format!("{glyph:04X}"))
            .collect()
    }

    #[test]
    fn the_footer_font_reaches_the_page_through_its_resources() {
        let (mut doc, pages) = document();
        let path = save(&mut doc, "resources.pdf");
        stamp(&path, &Footer {
            reference: "REF".to_string(),
            center: "Credit".to_string(),
        })
        .unwrap();

        let doc = Document::load(&path).unwrap();
        let page = doc.get_object(pages[0]).unwrap().as_dict().unwrap();
        let resources = page
            .get(b"Resources")
            .and_then(Object::as_reference)
            .map(|id| doc.get_object(id).unwrap().as_dict().unwrap())
            .expect("the renderer's resources are indirect");
        let fonts = resources.get(b"Font").unwrap().as_dict().unwrap();
        // The page's own font is still there, beside the footer's.
        assert!(fonts.has(b"F1"));
        assert!(fonts.has(FONT_KEY.as_bytes()));
    }

    #[test]
    fn the_footer_sits_inside_the_one_inch_margin() {
        let content = footer_content(72.0, 523.0, 33.0, "REF", "Credit", "2 / 2");
        // Right-aligned to the content edge, not the page edge.
        assert!(content.contains("1 0 0 1 72.00 33.00 Tm"));
        let right: f32 = 523.0 - text_width("2 / 2", SIZE_PT);
        assert!(content.contains(&format!("1 0 0 1 {right:.2} 33.00 Tm")));
        // The centre is centred on the page, within a point.
        let centre = 297.5 - text_width("Credit", SIZE_PT) / 2.0;
        assert!(content.contains(&format!("1 0 0 1 {centre:.2} 33.00 Tm")));
    }

    #[test]
    fn the_margin_is_one_constant_across_the_report_and_the_footer() {
        // 25.4 mm is 72 pt; the CSS margin, the GTK page setup and the footer's
        // x position must all come from the same number.
        assert!((MARGIN_PT - 72.0).abs() < 0.01, "{MARGIN_PT}");
        assert!((MARGIN_PT - crate::report::PRINT_MARGIN_MM as f32 * 72.0 / 25.4).abs() < 0.01);
    }

    #[test]
    fn the_embedded_font_covers_the_names_a_plan_carries() {
        // Latin with diacritics, Vietnamese and Cyrillic all have glyphs, so a
        // reference written in those scripts still prints.
        for text in ["ü", "é", "ế", "ộ", "Nguyễn", "Ánh", "Ж", "я", "—"] {
            for character in text.chars() {
                assert_ne!(glyph(character), 0, "no glyph for {character:?}");
            }
        }
        // A character the face does not have falls back to .notdef, which is the
        // box the report's own font stack would show for it.
        assert_eq!(glyph('\u{2709}'), 0);
    }

    #[test]
    fn a_rotated_page_is_refused_rather_than_misplaced() {
        let (mut doc, pages) = document();
        let page = doc.get_object_mut(pages[0]).unwrap().as_dict_mut().unwrap();
        page.set("Rotate", 90);
        let path = save(&mut doc, "rotated.pdf");

        let error = stamp(&path, &Footer {
            reference: "REF".to_string(),
            center: "Credit".to_string(),
        })
        .expect_err("a rotated page must not be stamped");
        assert!(error.contains("rotated"), "{error}");
    }

    #[test]
    fn a_contents_reference_to_an_array_keeps_its_pages() {
        // Some writers make /Contents a reference to an array object.
        let (mut doc, pages) = document();
        let first = streams(&doc, pages[0]).len();
        assert_eq!(first, 1);
        let array = doc.add_object(Object::Array(vec![{
            let dict = doc.get_object(pages[0]).unwrap().as_dict().unwrap();
            dict.get(b"Contents").unwrap().clone()
        }]));
        let page = doc.get_object_mut(pages[0]).unwrap().as_dict_mut().unwrap();
        page.set("Contents", array);
        let path = save(&mut doc, "contents-array.pdf");

        stamp(&path, &Footer {
            reference: "REF".to_string(),
            center: "Credit".to_string(),
        })
        .unwrap();

        let doc = Document::load(&path).unwrap();
        let streams = streams(&doc, pages[0]);
        assert_eq!(streams.len(), 2, "the original stream is still there");
        assert!(streams[0].contains("LPCFooter"));
        assert!(streams[1].contains("0 0 m 10 10 l S"));
    }

    #[test]
    fn a_real_webkit_print_keeps_every_page_and_gains_the_footer() {
        // The committed fixture is unstamped output from the app's own view:
        // compressed content streams, indirect resources, xref streams — the
        // shape the synthetic PDF above does not have.
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/webkit-plan.pdf");
        let dir = std::env::temp_dir().join(format!("lpc-webkit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("plan.pdf");
        std::fs::copy(&source, &path).unwrap();

        let before = Document::load(&path).unwrap();
        let pages = before.get_pages();
        assert_eq!(pages.len(), 7, "the fixture is a seven-page plan");
        for page in pages.values() {
            assert!(
                !streams(&before, *page).iter().any(|content| content.contains("LPCFooter")),
                "the fixture must be unstamped"
            );
        }
        drop(before);

        stamp(&path, &Footer {
            reference: "Nguyễn Văn Ánh — LP/2026 (A)".to_string(),
            center: "Prepared by A. Rigger".to_string(),
        })
        .unwrap();

        let after = Document::load(&path).unwrap();
        let pages = after.get_pages();
        assert_eq!(pages.len(), 7, "no page is lost to the rewrite");
        for (index, page) in pages.values().enumerate() {
            let streams = streams(&after, *page);
            assert!(
                streams.iter().any(|content| content.contains("LPCFooter")),
                "page {} has no footer",
                index + 1
            );
            // The print's own content is still there, so nothing was replaced.
            assert!(streams.len() >= 2, "page {} lost its content", index + 1);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
