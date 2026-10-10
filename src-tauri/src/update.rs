//! Checking GitHub for a newer release of this app.
//!
//! The AppImage is only *reminded* — its operator updates it their own way —
//! while the Windows build can fetch and start the release's installer. Both
//! can open the repository and the release page in the desktop's browser.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// The repository every check and download is kept to.
pub const REPO_URL: &str = "https://github.com/zhafribs/lifting-plan-calculator";
const RELEASES_API: &str = "https://api.github.com/repos/zhafribs/lifting-plan-calculator/releases/latest";
#[cfg(target_os = "windows")]
const DOWNLOAD_PREFIX: &str =
    "https://github.com/zhafribs/lifting-plan-calculator/releases/download/";
const USER_AGENT: &str = "lifting-plan-calculator";
const CHECK_TIMEOUT: Duration = Duration::from_secs(12);
#[cfg(target_os = "windows")]
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

/// What the app tells the operator about the newest release.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UpdateInfo {
    pub current: String,
    /// The newest release's version, without its leading `v`.
    pub latest: String,
    /// Whether `latest` is newer than the running `current`.
    pub newer: bool,
    /// The release page on GitHub.
    pub release_url: String,
    /// The repository page.
    pub repo_url: String,
    /// The Windows installer the app can fetch, when the release carries one.
    pub installer: Option<InstallerAsset>,
}

/// One downloadable installer from the release.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallerAsset {
    pub name: String,
    pub url: String,
}

/// `v1.2.3` / `1.2.3` as `[1, 2, 3]`; a `-suffix` / `+build` is ignored.
fn version_numbers(text: &str) -> Option<Vec<u64>> {
    let trimmed = text.trim().trim_start_matches(['v', 'V']);
    let core = trimmed.split(['-', '+']).next().unwrap_or(trimmed);
    if core.is_empty() {
        return None;
    }
    core.split('.').map(|part| part.parse::<u64>().ok()).collect()
}

/// Whether `latest` names a newer release than `current`.
///
/// Missing trailing components count as zero, so `1.2` and `1.2.0` agree.
pub fn is_newer(current: &str, latest: &str) -> bool {
    let (Some(mut current), Some(mut latest)) = (version_numbers(current), version_numbers(latest))
    else {
        return false;
    };
    while current.len() < latest.len() {
        current.push(0);
    }
    while latest.len() < current.len() {
        latest.push(0);
    }
    latest > current
}

/// Ask GitHub for the newest release and compare it with `current`.
pub fn check(current: &str) -> Result<UpdateInfo, String> {
    let body = ureq::get(RELEASES_API)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .timeout(CHECK_TIMEOUT)
        .call()
        .map_err(|error| format!("GitHub could not be reached: {error}"))?
        .into_string()
        .map_err(|error| format!("GitHub's answer could not be read: {error}"))?;
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|error| format!("GitHub's answer was not understood: {error}"))?;

    let tag = json
        .get("tag_name")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let latest = tag.trim_start_matches(['v', 'V']).to_string();
    let release_url = json
        .get("html_url")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(REPO_URL)
        .to_string();

    // The NSIS setup comes first when the release has both, then the MSI.
    let installer = {
        let assets: Vec<&serde_json::Value> = json
            .get("assets")
            .and_then(|value| value.as_array())
            .map(|assets| assets.iter().collect())
            .unwrap_or_default();
        let pick = |wanted: fn(&str) -> bool| {
            assets.iter().find_map(|asset| {
                let name = asset.get("name")?.as_str()?;
                if !wanted(name) {
                    return None;
                }
                let url = asset.get("browser_download_url")?.as_str()?;
                Some(InstallerAsset {
                    name: name.to_string(),
                    url: url.to_string(),
                })
            })
        };
        pick(|name| name.ends_with("-setup.exe")).or_else(|| pick(|name| name.ends_with(".msi")))
    };

    Ok(UpdateInfo {
        current: current.to_string(),
        newer: is_newer(current, &latest),
        latest,
        release_url,
        repo_url: REPO_URL.to_string(),
        installer,
    })
}

/// Fetch the installer and start it, on Windows. Returns the saved path; the
/// caller then quits so the installer can replace the running files.
#[cfg(target_os = "windows")]
pub fn install(asset: &InstallerAsset) -> Result<String, String> {
    if !asset.url.starts_with(DOWNLOAD_PREFIX) {
        return Err("That download is not from this project's releases.".to_string());
    }
    if !(asset.name.ends_with("-setup.exe") || asset.name.ends_with(".msi")) {
        return Err("That file is not a Windows installer.".to_string());
    }
    let directory = std::env::temp_dir().join("lifting-plan-calculator-update");
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("The installer could not be saved: {error}"))?;
    let target = directory.join(&asset.name);
    let mut reader = ureq::get(&asset.url)
        .set("User-Agent", USER_AGENT)
        .timeout(DOWNLOAD_TIMEOUT)
        .call()
        .map_err(|error| format!("The installer could not be downloaded: {error}"))?
        .into_reader();
    let mut file = std::fs::File::create(&target)
        .map_err(|error| format!("The installer could not be saved: {error}"))?;
    std::io::copy(&mut reader, &mut file)
        .map_err(|error| format!("The installer could not be saved: {error}"))?;
    drop(file);

    let mut command = if asset.name.ends_with(".msi") {
        let mut command = std::process::Command::new("msiexec");
        command.arg("/i").arg(&target);
        command
    } else {
        std::process::Command::new(&target)
    };
    command
        .spawn()
        .map_err(|error| format!("The installer could not be started: {error}"))?;
    Ok(target.to_string_lossy().to_string())
}

/// Not Windows: the reminder is the whole feature.
#[cfg(not(target_os = "windows"))]
pub fn install(_asset: &InstallerAsset) -> Result<String, String> {
    Err("The in-app installer is only for Windows — use the release page.".to_string())
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    // QA hook (mirrors --screen=): compare against a chosen version so the
    // reminder banner can be smoke-tested against an older build.
    let current = std::env::args()
        .find_map(|argument| argument.strip_prefix("--update-check-version=").map(str::to_string))
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    tauri::async_runtime::spawn_blocking(move || check(&current))
        .await
        .map_err(|error| format!("The update check failed: {error}"))?
}

#[tauri::command]
pub async fn install_update(asset: InstallerAsset) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || install(&asset))
        .await
        .map_err(|error| format!("The update failed: {error}"))?
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Only web links can be opened.".to_string());
    }
    crate::commands::open_with_desktop(&url)
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_component_by_component() {
        assert!(is_newer("2.3.0", "2.4.0"));
        assert!(is_newer("2.3.0", "2.3.1"));
        assert!(is_newer("2.3.0", "v2.3.1"));
        assert!(is_newer("2.3.0", "3.0.0"));
        assert!(is_newer("2.3.0", "2.10.0"));
        assert!(!is_newer("2.3.0", "2.3.0"));
        assert!(!is_newer("2.3.0", "2.2.9"));
        assert!(!is_newer("2.3.0", "1.0.0"));
    }

    #[test]
    fn missing_trailing_components_count_as_zero() {
        assert!(!is_newer("2.3.0", "2.3"));
        assert!(is_newer("2.3", "2.3.1"));
        assert!(!is_newer("2.3", "2.3.0"));
    }

    #[test]
    fn a_suffix_does_not_count_as_a_version() {
        assert!(!is_newer("2.3.0", "2.3.0-beta.1"));
        assert!(is_newer("2.3.0", "2.3.1-rc2"));
    }

    #[test]
    fn nonsense_never_reports_an_update() {
        assert!(!is_newer("2.3.0", "not-a-version"));
        assert!(!is_newer("", "2.4.0"));
        assert!(!is_newer("2.3.0", ""));
    }
}
