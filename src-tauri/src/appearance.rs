//! Appearance: the operator's scheme preference and the desktop's accent.
//!
//! The UI paints from tokens; this module only answers two questions: did the
//! operator choose a scheme, and does the desktop publish an accent colour? The
//! preference is a tiny file under `$XDG_CONFIG_HOME`, because a plan is never
//! stored between launches but a display choice is expected to stick.

use serde::{Deserialize, Serialize};

pub const PREFERENCES: [&str; 3] = ["system", "light", "dark"];

#[derive(Debug, Serialize)]
pub struct Appearance {
    pub preference: String,
    pub accent: Option<String>,
    /// Where the accent came from: "wallpaper", "desktop" or "default".
    pub accent_source: String,
    /// The desktop's own light/dark choice, when it publishes one.
    pub scheme: Option<String>,
    /// The desktop's own surfaces, when it publishes them.
    pub palette: Option<DesktopPalette>,
}

/// The colours the desktop itself uses, so the shell can match them exactly
/// instead of approximating them from one accent.
#[derive(Debug, Clone, Serialize)]
pub struct DesktopPalette {
    pub background: String,
    pub surface: String,
    pub text: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Stored {
    #[serde(default)]
    preference: String,
}

fn normalise(value: &str) -> String {
    let value = value.trim();
    if PREFERENCES.contains(&value) {
        value.to_string()
    } else {
        "system".to_string()
    }
}

fn config_path() -> Option<std::path::PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|home| std::path::PathBuf::from(home).join(".config"))
        })?;
    Some(base.join("lifting-plan-calculator").join("appearance.json"))
}

pub fn preference() -> String {
    config_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<Stored>(&text).ok())
        .map(|stored| normalise(&stored.preference))
        .unwrap_or_else(|| "system".to_string())
}

pub fn set_preference(value: &str) -> Result<(), String> {
    let path = config_path().ok_or_else(|| "There is no home folder to save the setting in.".to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("The setting folder could not be created: {error}"))?;
    }
    let body = serde_json::to_string(&Stored { preference: normalise(value) })
        .map_err(|error| format!("The setting could not be written: {error}"))?;
    std::fs::write(&path, body).map_err(|error| format!("The setting could not be saved: {error}"))
}

pub fn current() -> Appearance {
    let wallpaper = wallpaper_accent();
    let accent = wallpaper.clone().or_else(desktop_accent);
    let accent_source = if wallpaper.is_some() {
        "wallpaper"
    } else if accent.is_some() {
        "desktop"
    } else {
        "default"
    };
    Appearance {
        preference: preference(),
        accent,
        accent_source: accent_source.to_string(),
        scheme: desktop_scheme(),
        palette: desktop_palette(),
    }
}

/// The palette a wallpaper-aware desktop wrote, when there is one. Hyprland
/// setups commonly run pywal or wallust, which publish their 16 colours under
/// a fixed cache path; that palette is the desktop's colour in practice, so it
/// is preferred over a static accent setting.
#[cfg(not(target_os = "windows"))]
pub fn wallpaper_accent() -> Option<String> {
    let cache = cache_home()?;
    for relative in ["wal/colors.json", "wallust/colors.json"] {
        if let Ok(text) = std::fs::read_to_string(cache.join(relative)) {
            if let Some(colour) = accent_from_palette_json(&text) {
                return Some(colour);
            }
        }
    }

    // The generated GTK stylesheet carries the roles every GTK application
    // uses, so it is the closest thing to "the desktop's colour".
    if let Some(text) = gtk_theme_text() {
        if let Some(colour) = gtk_colour(&text, "accent_color")
            .or_else(|| gtk_colour(&text, "accent_bg_color"))
        {
            return Some(colour);
        }
    }

    // Desktop shells that theme every application write the palette into the
    // application's own theme file. Noctalia (the usual Hyprland shell) does
    // this for kitty, so the terminal's palette is the desktop's palette.
    if let Some(config) = config_home() {
        for relative in ["kitty/themes/noctalia.conf", "kitty/themes/wal.conf"] {
            if let Ok(text) = std::fs::read_to_string(config.join(relative)) {
                if let Some(colour) = accent_from_kitty(&text) {
                    return Some(colour);
                }
            }
        }
    }

    std::fs::read_to_string(cache.join("wal/colors"))
        .ok()
        .and_then(|text| accent_from_colour_lines(&text))
}

#[cfg(not(target_os = "windows"))]
fn config_home() -> Option<std::path::PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|home| std::path::PathBuf::from(home).join(".config"))
        })?;
    Some(base)
}

/// The desktop's light/dark choice. Noctalia keeps it in its config, and on a
/// wallpaper-themed desktop it is the authority the other applications follow.
#[cfg(not(target_os = "windows"))]
fn desktop_scheme() -> Option<String> {
    let text = std::fs::read_to_string(config_home()?.join("noctalia/config.toml")).ok()?;
    mode_from_noctalia(&text)
}

#[cfg(target_os = "windows")]
fn desktop_scheme() -> Option<String> {
    None
}

fn mode_from_noctalia(text: &str) -> Option<String> {
    let mut in_theme = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_theme = line == "[theme]";
            continue;
        }
        if !in_theme {
            continue;
        }
        let Some(rest) = line.strip_prefix("mode") else {
            continue;
        };
        let value = rest.trim_start_matches([' ', '=']).trim().trim_matches('"');
        if value == "dark" || value == "light" {
            return Some(value.to_string());
        }
    }
    None
}

/// A kitty theme file: `color1 #rrggbb` per line, background in `color0`.
fn accent_from_kitty(text: &str) -> Option<String> {
    let palette: Vec<String> = text
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let key = parts.next()?;
            if !key.starts_with("color") || key == "color0" {
                return None;
            }
            normalise_hex(parts.next()?)
        })
        .collect();
    pick_accent(palette.into_iter())
}

/// `@define-color name #rrggbb;` in a generated GTK stylesheet.
fn gtk_colour(text: &str, wanted: &str) -> Option<String> {
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("@define-color") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        if parts.next() != Some(wanted) {
            continue;
        }
        if let Some(colour) = parts
            .next()
            .and_then(|value| normalise_hex(value.trim_end_matches(';')))
        {
            return Some(colour);
        }
    }
    None
}

/// The GTK stylesheet and the files it imports. Noctalia writes `gtk.css` as a
/// single `@import` of `noctalia.css`, so the roles are one level down.
#[cfg(not(target_os = "windows"))]
fn gtk_theme_text() -> Option<String> {
    let config = config_home()?;
    for directory in ["gtk-3.0", "gtk-4.0"] {
        let base = config.join(directory);
        let Ok(text) = std::fs::read_to_string(base.join("gtk.css")) else {
            continue;
        };
        let mut combined = text.clone();
        for line in text.lines() {
            let Some(start) = line.find("@import") else {
                continue;
            };
            let Some(open) = line[start..].find('"').map(|index| start + index) else {
                continue;
            };
            let Some(close) = line[open + 1..].find('"').map(|index| open + 1 + index) else {
                continue;
            };
            let name = &line[open + 1..close];
            if name.contains("://") {
                continue;
            }
            if let Ok(imported) = std::fs::read_to_string(base.join(name)) {
                combined.push('\n');
                combined.push_str(&imported);
            }
        }
        if combined.contains("@define-color") {
            return Some(combined);
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn desktop_palette() -> Option<DesktopPalette> {
    let text = gtk_theme_text()?;
    let background = gtk_colour(&text, "window_bg_color")
        .or_else(|| gtk_colour(&text, "view_bg_color"))?;
    let text_colour = gtk_colour(&text, "window_fg_color")
        .or_else(|| gtk_colour(&text, "view_fg_color"))?;
    let surface = gtk_colour(&text, "headerbar_bg_color")
        .or_else(|| gtk_colour(&text, "view_bg_color"))
        .unwrap_or_else(|| background.clone());
    Some(DesktopPalette {
        background,
        surface,
        text: text_colour,
    })
}

#[cfg(target_os = "windows")]
fn desktop_palette() -> Option<DesktopPalette> {
    None
}

#[cfg(target_os = "windows")]
pub fn wallpaper_accent() -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
fn cache_home() -> Option<std::path::PathBuf> {
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|home| std::path::PathBuf::from(home).join(".cache"))
        })?;
    Some(base)
}

fn normalise_hex(value: &str) -> Option<String> {
    let text = value.trim().trim_start_matches('#');
    if text.len() != 6 || !text.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{}", text.to_ascii_lowercase()))
}

/// pywal and wallust both write `{"colors": {"color0": "#..", ...}}`.
fn accent_from_palette_json(text: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let colours = value.get("colors")?.as_object()?;
    let mut palette: Vec<(u32, String)> = colours
        .iter()
        .filter_map(|(key, value)| {
            // color0 is the background; the rest are the usable palette.
            let index: u32 = key.strip_prefix("color")?.parse().ok()?;
            if index == 0 || index > 15 {
                return None;
            }
            Some((index, normalise_hex(value.as_str()?)?))
        })
        .collect();
    palette.sort_by_key(|(index, _)| *index);
    pick_accent(palette.into_iter().map(|(_, colour)| colour))
}

/// The plain pywal list: one `#rrggbb` per line, background first.
fn accent_from_colour_lines(text: &str) -> Option<String> {
    let palette: Vec<String> = text
        .lines()
        .skip(1)
        .filter_map(normalise_hex)
        .collect();
    pick_accent(palette.into_iter())
}

/// The most usable colour in a palette: vivid, and neither near-black nor
/// near-white, so it can sit on either surface.
fn pick_accent(colours: impl Iterator<Item = String>) -> Option<String> {
    // First wins on a tie: the palette's own order is the better guess when two
    // slots are equally vivid.
    let mut best: Option<(f32, String)> = None;
    for colour in colours {
        let score = accent_score(&colour);
        if best.as_ref().is_none_or(|(best_score, _)| score > *best_score) {
            best = Some((score, colour));
        }
    }
    best.map(|(_, colour)| colour)
}

fn accent_score(colour: &str) -> f32 {
    let channels: Vec<f32> = colour
        .trim_start_matches('#')
        .as_bytes()
        .chunks(2)
        .filter_map(|pair| std::str::from_utf8(pair).ok())
        .filter_map(|pair| u8::from_str_radix(pair, 16).ok())
        .map(|channel| f32::from(channel) / 255.0)
        .collect();
    if channels.len() != 3 {
        return 0.0;
    }
    let max = channels.iter().copied().fold(0.0_f32, f32::max);
    let min = channels.iter().copied().fold(1.0_f32, f32::min);
    let lightness = (max + min) / 2.0;
    let delta = max - min;
    let saturation = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * lightness - 1.0).abs())
    };
    // Vivid and mid-light: a saturated colour reads as an accent on either
    // surface, while the palette's background and text slots do not.
    (saturation * (1.0 - (lightness - 0.5).abs() / 0.5)).max(0.0)
}

/// The desktop's accent as `#rrggbb`, when it publishes one.
#[cfg(target_os = "windows")]
pub fn desktop_accent() -> Option<String> {
    crate::theme_windows::system_accent()
}

#[cfg(not(target_os = "windows"))]
pub fn desktop_accent() -> Option<String> {
    gnome_accent().or_else(kde_accent)
}

#[cfg(not(target_os = "windows"))]
fn gnome_accent() -> Option<String> {
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "accent-color"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    accent_from_name(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(not(target_os = "windows"))]
fn kde_accent() -> Option<String> {
    for tool in ["kreadconfig6", "kreadconfig5"] {
        let output = std::process::Command::new(tool)
            .args([
                "--file",
                "kdeglobals",
                "--group",
                "Colors:Selection",
                "--key",
                "BackgroundNormal",
            ])
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                if let Some(colour) = accent_from_kde(&String::from_utf8_lossy(&output.stdout)) {
                    return Some(colour);
                }
            }
        }
    }
    None
}

/// `'blue'` (as `gsettings` quotes it) to the accent the GNOME palette uses.
fn accent_from_name(raw: &str) -> Option<String> {
    let name = raw.trim().trim_matches('\'').trim_matches('"').to_ascii_lowercase();
    let hex = match name.as_str() {
        "blue" => "#3584e4",
        "teal" => "#2190a4",
        "green" => "#3a944a",
        "yellow" => "#c88800",
        "orange" => "#ed5b00",
        "red" => "#e62d42",
        "pink" => "#d56199",
        "purple" => "#9141ac",
        "slate" => "#6f8396",
        _ => return None,
    };
    Some(hex.to_string())
}

/// `49,116,181` (KDE stores colours as comma-separated channels).
fn accent_from_kde(raw: &str) -> Option<String> {
    let mut channels = raw.trim().split(',');
    let red: u8 = channels.next()?.trim().parse().ok()?;
    let green: u8 = channels.next()?.trim().parse().ok()?;
    let blue: u8 = channels.next()?.trim().parse().ok()?;
    if channels.next().is_some() {
        return None;
    }
    Some(format!("#{red:02x}{green:02x}{blue:02x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_preference_reads_as_system() {
        assert_eq!(normalise("dark"), "dark");
        assert_eq!(normalise(" light "), "light");
        assert_eq!(normalise("midnight"), "system");
        assert_eq!(normalise(""), "system");
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn gnome_accent_names_map_to_colours() {
        assert_eq!(accent_from_name("'blue'\n"), Some("#3584e4".to_string()));
        assert_eq!(accent_from_name("\"teal\""), Some("#2190a4".to_string()));
        assert_eq!(accent_from_name("'chartreuse'"), None);
    }

    #[test]
    fn a_kitty_theme_supplies_the_palette() {
        let text = "color0 #121318\ncolor1 #3a3f4b\ncolor2 #22d3ee\ncolor3 #ffb4ab\nbackground #121318\n";
        assert_eq!(accent_from_kitty(text), Some("#22d3ee".to_string()));
    }

    #[test]
    fn a_pastel_noctalia_palette_still_yields_one_of_its_colours() {
        // The real Noctalia output is all pastels; the point is that one of them
        // is chosen, not that a particular shade wins.
        let text = "color1 #ffb4ab\ncolor2 #b4c4ff\ncolor3 #c1c5dd\ncolor11 #ebcdef\n";
        let chosen = accent_from_kitty(text).expect("a palette colour");
        assert!(text.contains(&chosen), "chosen {chosen} is not in the palette");
    }

    #[test]
    fn a_generated_gtk_css_supplies_the_accent() {
        let text = "@define-color window_bg_color #121318;\n@define-color accent_color #ffb3b2;\n";
        assert_eq!(gtk_colour(text, "accent_color"), Some("#ffb3b2".to_string()));
        assert_eq!(gtk_colour(text, "theme_fg_color"), None);
    }

    #[test]
    fn noctalia_reports_its_own_scheme() {
        let text = "[theme]\nmode = \"light\"\nsource = \"wallpaper\"\n\n[bar]\nmode = \"dark\"\n";
        assert_eq!(mode_from_noctalia(text), Some("light".to_string()));
        assert_eq!(mode_from_noctalia("[theme]\nmode = \"system\"\n"), None);
        assert_eq!(mode_from_noctalia("[bar]\nmode = \"dark\"\n"), None);
    }

    #[test]
    fn a_palette_yields_its_most_usable_colour() {
        let text = r##"{"colors": {
            "color0": "#1a1b26", "color1": "#f7768e", "color2": "#9ece6a",
            "color3": "#e0af68", "color4": "#7aa2f7", "color5": "#bb9af7",
            "color6": "#7dcfff", "color7": "#a9b1d6", "color8": "#414868"
        }}"##;
        // Bright and saturated: the cyan wins over the dark background and the
        // near-grey text colour.
        assert_eq!(accent_from_palette_json(text), Some("#7dcfff".to_string()));
    }

    #[test]
    fn background_and_near_white_are_not_an_accent() {
        let text = r##"{"colors": {"color0": "#000000", "color7": "#f7f7f7", "color9": "#7aa2f7"}}"##;
        assert_eq!(accent_from_palette_json(text), Some("#7aa2f7".to_string()));
    }

    #[test]
    fn a_plain_pywal_list_skips_the_background_line() {
        let text = "#101010
#ff5500
#00aaff
";
        assert_eq!(accent_from_colour_lines(text), Some("#ff5500".to_string()));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn kde_accent_channels_become_a_hex_colour() {
        assert_eq!(accent_from_kde("49,116,181\n"), Some("#3174b5".to_string()));
        assert_eq!(accent_from_kde("0,0,0"), Some("#000000".to_string()));
        assert_eq!(accent_from_kde("49,116"), None);
        assert_eq!(accent_from_kde("49,116,181,4"), None);
        assert_eq!(accent_from_kde("300,0,0"), None);
    }
}
