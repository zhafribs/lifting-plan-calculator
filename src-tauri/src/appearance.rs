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
    Appearance {
        preference: preference(),
        accent: desktop_accent(),
    }
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
