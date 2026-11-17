//! Checking GitHub for a newer release of this app.
//!
//! The AppImage is only *reminded* — its operator updates it their own way —
//! while the Windows build can fetch and start the release's installer. Both
//! can open the repository and the release page in the desktop's browser.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The Ed25519 public key that signs each release's `SHA256SUMS.txt`, as 64 hex
/// characters. Empty until the maintainer runs `scripts/update-signing-key.sh`;
/// once set, an installer without a valid signature is never run. See
/// `docs/releases/README.md` for the ceremony.
const UPDATE_PUBLIC_KEY: &str = include_str!("../update-public-key.txt");

/// The repository every check and download is kept to.
pub const REPO_URL: &str = "https://github.com/zhafribs/lifting-plan-calculator";
const RELEASES_API: &str = "https://api.github.com/repos/zhafribs/lifting-plan-calculator/releases/latest";
const RELEASES_FEED: &str = "https://github.com/zhafribs/lifting-plan-calculator/releases.atom";
const USER_AGENT: &str = "lifting-plan-calculator";
const CHECK_TIMEOUT: Duration = Duration::from_secs(12);
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
    /// The Linux AppImage the app can fetch and swap in, when present.
    pub appimage: Option<InstallerAsset>,
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

/// Ask GitHub's API for the newest release and compare it with `current`.
fn check_api(current: &str) -> Result<UpdateInfo, String> {
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
    let appimage = if cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some() {
        let assets: Vec<&serde_json::Value> = json
            .get("assets")
            .and_then(|value| value.as_array())
            .map(|assets| assets.iter().collect())
            .unwrap_or_default();
        assets.iter().find_map(|asset| {
            let name = asset.get("name")?.as_str()?;
            if !name.ends_with("-linux-x86_64.AppImage") {
                return None;
            }
            let url = asset.get("browser_download_url")?.as_str()?;
            Some(InstallerAsset {
                name: name.to_string(),
                url: url.to_string(),
            })
        })
    } else {
        None
    };

    Ok(UpdateInfo {
        current: current.to_string(),
        newer: is_newer(current, &latest),
        latest,
        release_url,
        repo_url: REPO_URL.to_string(),
        installer,
        appimage,
    })
}

/// The first stable release tag in GitHub's releases Atom feed.
fn latest_tag_from_atom(xml: &str) -> Option<String> {
    for entry in xml.split("<entry>").skip(1) {
        let Some(index) = entry.find("/releases/tag/") else {
            continue;
        };
        let rest = &entry[index + "/releases/tag/".len()..];
        let tag: String = rest
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
            .collect();
        if tag.starts_with('v') && !tag.contains('-') && version_numbers(&tag).is_some() {
            return Some(tag);
        }
    }
    None
}

/// Build the update answer from a tag when the API is rate-limited.
fn update_from_tag(current: &str, tag: &str) -> UpdateInfo {
    let latest = tag.trim_start_matches(['v', 'V']).to_string();
    let base = format!(
        "https://github.com/zhafribs/lifting-plan-calculator/releases/download/{tag}"
    );
    let installer = Some(InstallerAsset {
        name: format!("Lifting-Plan-Calculator-{latest}-windows-x86_64-setup.exe"),
        url: format!("{base}/Lifting-Plan-Calculator-{latest}-windows-x86_64-setup.exe"),
    });
    let appimage = if cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some() {
        Some(InstallerAsset {
            name: format!("Lifting-Plan-Calculator-{latest}-linux-x86_64.AppImage"),
            url: format!("{base}/Lifting-Plan-Calculator-{latest}-linux-x86_64.AppImage"),
        })
    } else {
        None
    };
    UpdateInfo {
        current: current.to_string(),
        newer: is_newer(current, &latest),
        latest,
        release_url: format!(
            "https://github.com/zhafribs/lifting-plan-calculator/releases/tag/{tag}"
        ),
        repo_url: REPO_URL.to_string(),
        installer,
        appimage,
    }
}

/// The Atom fallback: read the latest stable tag without the API, so a
/// rate-limited GitHub API does not disable the update button.
fn check_feed(current: &str) -> Result<UpdateInfo, String> {
    let body = ureq::get(RELEASES_FEED)
        .set("User-Agent", USER_AGENT)
        .timeout(CHECK_TIMEOUT)
        .call()
        .map_err(|error| format!("the release feed could not be read: {error}"))?
        .into_string()
        .map_err(|error| format!("the release feed could not be read: {error}"))?;
    let tag = latest_tag_from_atom(&body)
        .ok_or_else(|| "the release feed carried no stable release tag".to_string())?;
    Ok(update_from_tag(current, &tag))
}

/// Ask GitHub for the newest release. The API is tried first; its Atom feed is
/// the fallback when the API is rate-limited or blocked.
pub fn check(current: &str) -> Result<UpdateInfo, String> {
    match check_api(current) {
        Ok(info) => Ok(info),
        Err(api_error) => match check_feed(current) {
            Ok(info) => Ok(info),
            Err(feed_error) => Err(format!(
                "GitHub could not be reached: {api_error}. {feed_error}. \
                 If the API is rate-limited, open the release page instead."
            )),
        },
    }
}

/// The SHA-256 of some bytes, lowercase hex.
fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The digest a release's `SHA256SUMS.txt` records for `name`.
///
/// The file is `sha256sum ./*` output, so the name may carry a `./` prefix.
fn expected_digest(manifest: &str, name: &str) -> Option<String> {
    manifest.lines().find_map(|line| {
        let (hash, file) = line.split_once(char::is_whitespace)?;
        let file = file.trim().trim_start_matches("./");
        (file == name && hash.len() == 64).then(|| hash.to_ascii_lowercase())
    })
}

/// Whether this build requires a signed checksum manifest.
fn signing_required() -> bool {
    !UPDATE_PUBLIC_KEY.trim().is_empty()
}

/// Verify a 64-byte Ed25519 signature (hex) over the manifest's bytes.
fn verify_signature(manifest: &[u8], signature_hex: &str) -> Result<(), String> {
    if !signing_required() {
        return Ok(());
    }
    verify_with(UPDATE_PUBLIC_KEY, manifest, signature_hex)
}

/// The verification itself, against a given key, so it can be tested with a
/// keypair the test owns.
fn verify_with(public_hex: &str, manifest: &[u8], signature_hex: &str) -> Result<(), String> {
    use ed25519_compact::{PublicKey, Signature};

    let hex_decode = |text: &str| -> Option<Vec<u8>> {
        // Bytes, not characters: the signature comes from the network and a
        // non-ASCII byte must be rejected, not panic on a UTF-8 boundary.
        let bytes = text.trim().as_bytes();
        if bytes.len() % 2 != 0 {
            return None;
        }
        bytes
            .chunks(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
            .collect()
    };
    let key = hex_decode(public_hex).ok_or_else(|| {
        "This build's update key is malformed; updates are disabled.".to_string()
    })?;
    let signature = hex_decode(signature_hex).ok_or_else(|| {
        "The release signature could not be read.".to_string()
    })?;
    let key = PublicKey::from_slice(&key)
        .map_err(|_| "This build's update key is malformed; updates are disabled.".to_string())?;
    let signature = Signature::from_slice(&signature)
        .map_err(|_| "The release signature could not be read.".to_string())?;
    key.verify(manifest, &signature)
        .map_err(|_| "The release checksums are not signed by this application's key.".to_string())
}

/// Fetch a release's manifest, prove its signature, and find the installer in it.
///
/// The signature is checked before any digest is read, so an unsigned or
/// wrongly signed manifest cannot vouch for anything.
fn verified_manifest(
    asset_url: &str,
    name: &str,
    fetch: impl Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<String, String> {
    let url = manifest_url(asset_url).ok_or_else(|| {
        "That download does not belong to a release with checksums.".to_string()
    })?;
    let manifest = fetch(&url)?;
    if signing_required() {
        let signature = fetch(&format!("{url}.sig"))?;
        let signature = String::from_utf8(signature)
            .map_err(|_| "The release signature could not be read.".to_string())?;
        verify_signature(&manifest, &signature)?;
    }
    let text = String::from_utf8(manifest)
        .map_err(|_| "The release checksums could not be read.".to_string())?;
    if expected_digest(&text, name).is_none() {
        return Err("That installer is not listed in its release's checksums.".to_string());
    }
    Ok(text)
}

/// Whether a URL is one of this repository's release assets.
fn is_release_asset_url(url: &str) -> bool {
    url::Url::parse(url)
        .map(|parsed| {
            parsed.scheme() == "https"
                && parsed.host_str() == Some("github.com")
                && parsed
                    .path()
                    .starts_with("/zhafribs/lifting-plan-calculator/releases/download/")
        })
        .unwrap_or(false)
}

/// The tag an asset URL belongs to, e.g. `v2.10.0`.
fn tag_of_asset(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    let rest = parsed.path().split("/releases/download/").nth(1)?;
    let tag = rest.split('/').next()?;
    (!tag.is_empty()).then(|| tag.to_string())
}

/// The release's checksum manifest, next to the asset itself.
fn manifest_url(asset_url: &str) -> Option<String> {
    let (base, _) = asset_url.rsplit_once('/')?;
    Some(format!("{base}/SHA256SUMS.txt"))
}

/// Fetch a small release file (the manifest or its signature).
fn fetch_bytes(url: &str, timeout: Duration) -> Result<Vec<u8>, String> {
    use std::io::Read;

    let mut bytes = Vec::new();
    ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .timeout(timeout)
        .call()
        .map_err(|error| format!("The release file could not be read: {error}"))?
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|error| format!("The release file could not be read: {error}"))?;
    Ok(bytes)
}

/// Download an asset into `target`, proving the release manifest and digest
/// first. This is shared by the Windows installer and the Linux AppImage.
fn verified_download(asset: &InstallerAsset, target: &std::path::Path) -> Result<(), String> {
    if !is_release_asset_url(&asset.url) {
        return Err("That download is not from this project's releases.".to_string());
    }
    let name = std::path::Path::new(&asset.name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| *name == asset.name)
        .ok_or_else(|| "That update names a file outside its release.".to_string())?
        .to_string();
    let tag = tag_of_asset(&asset.url)
        .ok_or_else(|| "That download has no release tag.".to_string())?;
    if !is_newer(env!("CARGO_PKG_VERSION"), tag.trim_start_matches(['v', 'V'])) {
        return Err(format!(
            "That release ({tag}) is not newer than this build ({}).",
            env!("CARGO_PKG_VERSION")
        ));
    }

    let manifest = verified_manifest(&asset.url, &name, |url| fetch_bytes(url, CHECK_TIMEOUT))?;
    let expected = expected_digest(&manifest, &name)
        .ok_or_else(|| "That file is not listed in its release's checksums.".to_string())?;

    let mut reader = ureq::get(&asset.url)
        .set("User-Agent", USER_AGENT)
        .timeout(DOWNLOAD_TIMEOUT)
        .call()
        .map_err(|error| format!("The update could not be downloaded: {error}"))?
        .into_reader();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(|error| format!("The update could not be saved: {error}"))?;
    let copied = std::io::copy(&mut reader, &mut file);
    drop(file);
    copied.map_err(|error| {
        let _ = std::fs::remove_file(target);
        format!("The update could not be saved: {error}")
    })?;

    let bytes = std::fs::read(target).map_err(|error| {
        let _ = std::fs::remove_file(target);
        format!("The downloaded update could not be read back: {error}")
    })?;
    if digest(&bytes) != expected {
        let _ = std::fs::remove_file(target);
        return Err("The downloaded update does not match its release checksum. \
                    It was not installed."
            .to_string());
    }
    Ok(())
}

/// Replace the running AppImage after the app exits, then start the new one.
///
/// The old file is moved aside while the new one is put in place; only after the
/// new AppImage has stayed alive for a few seconds is the old file deleted. If
/// the new file does not start, the old one is put back.
const APPIMAGE_UPDATE_SCRIPT: &str = r#"#!/bin/sh
set -u
pid="$1"
target="$2"
replacement="$3"
log="${TMPDIR:-/tmp}/lifting-plan-calculator-update.log"
exec >>"$log" 2>&1
echo "--- $(date) updating $target"
while kill -0 "$pid" 2>/dev/null; do
  sleep 0.2
done
sleep 0.5
chmod +x "$replacement" || exit 1
backup="${target}.old"
rm -f "$backup"
mv "$target" "$backup" || exit 1
if ! mv "$replacement" "$target"; then
  mv "$backup" "$target"
  exit 1
fi
"$target" >/dev/null 2>&1 &
newpid=$!
sleep 3
state=$(ps -p "$newpid" -o stat= 2>/dev/null || true)
case "$state" in
  Z*|"")
    echo "new AppImage did not start; restoring the old file"
    rm -f "$target"
    mv "$backup" "$target"
    "$target" >/dev/null 2>&1 &
    ;;
  *)
    rm -f "$backup"
    echo "update ok"
    ;;
esac
rm -f "$0"
"#;

/// Fetch the installer and start it, on Windows. Returns the saved path; the
/// caller then quits so the installer can replace the running files.
#[cfg(target_os = "windows")]
pub fn install(asset: &InstallerAsset) -> Result<String, String> {
    if !is_release_asset_url(&asset.url) {
        return Err("That download is not from this project's releases.".to_string());
    }
    // The name is a file name, never a path: `..\..\Startup\x-setup.exe` is
    // rejected rather than written outside the update folder.
    let name = std::path::Path::new(&asset.name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| *name == asset.name)
        .ok_or_else(|| "That update names a file outside its release.".to_string())?
        .to_string();
    if !(name.ends_with("-setup.exe") || name.ends_with(".msi")) {
        return Err("That file is not a Windows installer.".to_string());
    }

    let directory = std::env::temp_dir().join("lifting-plan-calculator-update");
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("The installer could not be saved: {error}"))?;
    let target = directory.join(&name);
    let _ = std::fs::remove_file(&target);
    verified_download(asset, &target)?;

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

/// Linux AppImage: download the new file, replace the running AppImage after
/// this process exits, and start the new one. Returns the AppImage path.
#[cfg(target_os = "linux")]
pub fn install(asset: &InstallerAsset) -> Result<String, String> {
    use std::process::Stdio;

    if !asset.name.ends_with("-linux-x86_64.AppImage") {
        return Err("That release file is not a Linux AppImage.".to_string());
    }
    let path = std::env::var("APPIMAGE").map_err(|_| {
        "This copy is not running as an AppImage. Download the new file from the release page."
            .to_string()
    })?;
    let target = std::fs::canonicalize(&path).map_err(|_| {
        "The running AppImage path could not be resolved. Download the new file from the release page."
            .to_string()
    })?;
    if !target.is_file() {
        return Err("The running AppImage is not a regular file.".to_string());
    }
    let parent = target
        .parent()
        .ok_or_else(|| "The running AppImage has no parent folder.".to_string())?;
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "The running AppImage has no file name.".to_string())?;
    let staged = parent.join(format!(".{file_name}.new"));
    let _ = std::fs::remove_file(&staged);
    verified_download(asset, &staged)?;

    let script = std::env::temp_dir().join(format!(
        "lifting-plan-calculator-update-{}.sh",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&script);
    let script_result = (|| -> Result<(), String> {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o700)
            .open(&script)
            .map_err(|error| format!("The update helper could not be created: {error}"))?;
        file.write_all(APPIMAGE_UPDATE_SCRIPT.as_bytes())
            .map_err(|error| format!("The update helper could not be written: {error}"))?;
        Ok(())
    })();
    if let Err(error) = script_result {
        let _ = std::fs::remove_file(&staged);
        return Err(error);
    }

    let child = std::process::Command::new("nohup")
        .arg("sh")
        .arg(&script)
        .arg(std::process::id().to_string())
        .arg(&target)
        .arg(&staged)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if let Err(error) = child {
        let _ = std::fs::remove_file(&staged);
        let _ = std::fs::remove_file(&script);
        return Err(format!("The update helper could not be started: {error}"));
    }
    Ok(target.to_string_lossy().to_string())
}

/// Other platforms: the release page is the update path.
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn install(_asset: &InstallerAsset) -> Result<String, String> {
    Err("In-app updating is only available on Windows and the Linux AppImage.".to_string())
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
    fn the_atom_feed_skips_a_prerelease_and_finds_the_stable_tag() {
        let xml = r#"<feed>
          <entry><link href="https://github.com/zhafribs/lifting-plan-calculator/releases/tag/v2.14.0-rc.1"/></entry>
          <entry><link href="https://github.com/zhafribs/lifting-plan-calculator/releases/tag/v2.13.3"/></entry>
        </feed>"#;
        assert_eq!(latest_tag_from_atom(xml).as_deref(), Some("v2.13.3"));
    }

    #[test]
    fn the_atom_fallback_builds_the_release_assets() {
        let info = update_from_tag("2.13.2", "v2.13.3");
        assert!(info.newer);
        assert_eq!(info.latest, "2.13.3");
        assert!(info.installer.is_some());
        assert!(info
            .installer
            .as_ref()
            .unwrap()
            .url
            .ends_with("/v2.13.3/Lifting-Plan-Calculator-2.13.3-windows-x86_64-setup.exe"));
    }

    #[test]
    fn only_this_projects_release_urls_are_accepted() {
        let good = "https://github.com/zhafribs/lifting-plan-calculator/releases/download/v2.10.0/x-setup.exe";
        assert!(is_release_asset_url(good));
        // Another host, another repository, plain http, and path tricks.
        for bad in [
            "https://evil.example/zhafribs/lifting-plan-calculator/releases/download/v1/x-setup.exe",
            "http://github.com/zhafribs/lifting-plan-calculator/releases/download/v1/x-setup.exe",
            "https://github.com/other/repo/releases/download/v1/x-setup.exe",
            "https://github.com/zhafribs/lifting-plan-calculator/releases/download/../../x",
            "not a url",
        ] {
            assert!(!is_release_asset_url(bad), "{bad}");
        }
    }

    #[test]
    fn the_tag_comes_from_the_asset_url() {
        let url = "https://github.com/zhafribs/lifting-plan-calculator/releases/download/v2.10.0/x-setup.exe";
        assert_eq!(tag_of_asset(url).as_deref(), Some("v2.10.0"));
        assert_eq!(tag_of_asset("https://github.com/x"), None);
    }

    #[test]
    fn a_malformed_signature_is_refused_not_panicked_on() {
        let public = "c2393429431fc55446c93650d725f89a8af16bc29bdb20d5860a4477b53370ab";
        for signature in ["0\u{4e00}", "zz", "", "00"] {
            assert!(verify_with(public, b"manifest", signature).is_err(), "{signature:?}");
        }
    }

    #[test]
    fn the_update_path_refuses_an_unsigned_or_altered_manifest() {
        let manifest = include_bytes!("../tests/fixtures/update-manifest.txt").to_vec();
        let signature = include_bytes!("../tests/fixtures/update-manifest.sig").to_vec();
        let name = "Lifting-Plan-Calculator-2.9.4-linux-x86_64.AppImage";
        let asset = "https://github.com/o/r/releases/download/v2.9.4/x.exe";

        let signed = |url: &str| -> Result<Vec<u8>, String> {
            if url.ends_with(".sig") { Ok(signature.clone()) } else { Ok(manifest.clone()) }
        };
        assert!(verified_manifest(asset, name, signed).is_ok());

        // A release with no signature is refused.
        let unsigned = |url: &str| -> Result<Vec<u8>, String> {
            if url.ends_with(".sig") { Err("404".to_string()) } else { Ok(manifest.clone()) }
        };
        assert!(verified_manifest(asset, name, unsigned).is_err());

        // A changed manifest no longer matches its signature.
        let altered = |url: &str| -> Result<Vec<u8>, String> {
            if url.ends_with(".sig") {
                Ok(signature.clone())
            } else {
                Ok(b"1e4acd312bebe5992bcb4cd563b16464355229602425f1de56bfd2f18b4290c1  ./other.exe\n".to_vec())
            }
        };
        assert!(verified_manifest(asset, name, altered).is_err());

        // A name that is not listed is refused even when the manifest is valid.
        assert!(verified_manifest(asset, "other-setup.exe", signed).is_err());
    }

    #[test]
    fn the_committed_key_verifies_the_manifest_that_signed_it() {
        // Signed by the maintainer's key when the key was created. If this
        // fails, the committed public key and the release signing key differ.
        let manifest = include_bytes!("../tests/fixtures/update-manifest.txt");
        let signature = include_str!("../tests/fixtures/update-manifest.sig");
        // Through the same entry point the installer uses.
        assert_eq!(verify_signature(manifest, signature), Ok(()));
    }

    #[test]
    fn a_signature_proves_the_manifest_came_from_the_key() {
        use ed25519_compact::{KeyPair, Noise, Seed};

        let key = KeyPair::from_seed(Seed::generate());
        let public = key.pk.as_ref().iter().map(|b| format!("{b:02x}")).collect::<String>();
        let manifest = b"abc  ./installer-setup.exe\n";
        let signature = key.sk.sign(manifest, Some(Noise::generate()));
        let signature_hex = signature.as_ref().iter().map(|b| format!("{b:02x}")).collect::<String>();

        assert_eq!(verify_with(&public, manifest, &signature_hex), Ok(()));
        // A changed manifest, another key, or a truncated signature all fail.
        assert!(verify_with(&public, b"abc  ./other-setup.exe\n", &signature_hex).is_err());
        let other = KeyPair::from_seed(Seed::generate());
        let other_public = other.pk.as_ref().iter().map(|b| format!("{b:02x}")).collect::<String>();
        assert!(verify_with(&other_public, manifest, &signature_hex).is_err());
        assert!(verify_with(&public, manifest, &signature_hex[..20]).is_err());
        // This checkout carries a key, so every installed copy requires one.
        assert!(signing_required());
    }

    #[test]
    fn a_release_manifest_names_its_installer_by_digest() {
        let manifest = "\n1e4acd312bebe5992bcb4cd563b16464355229602425f1de56bfd2f18b4290c1  ./Lifting-Plan-Calculator-2.9.3-linux-x86_64.AppImage\n820ae3c7580ca637cad15f65638960ee2a96e5a491e224691c86d6f25e6c9e8a  ./Lifting-Plan-Calculator-2.9.3-windows-x86_64-setup.exe\n";
        assert_eq!(
            expected_digest(manifest, "Lifting-Plan-Calculator-2.9.3-windows-x86_64-setup.exe").as_deref(),
            Some("820ae3c7580ca637cad15f65638960ee2a96e5a491e224691c86d6f25e6c9e8a")
        );
        // Not listed: the installer must not run.
        assert_eq!(expected_digest(manifest, "other-setup.exe"), None);
        // A malformed line is not a checksum.
        assert_eq!(expected_digest("abc  ./x-setup.exe", "x-setup.exe"), None);
    }

    #[test]
    fn an_update_download_must_come_from_this_projects_releases() {
        let asset = InstallerAsset {
            name: "Lifting-Plan-Calculator-9.9.9-linux-x86_64.AppImage".to_string(),
            url: "https://evil.example/releases/download/v9.9.9/app.AppImage".to_string(),
        };
        let target = std::env::temp_dir().join("lpc-should-not-exist.AppImage");
        let error = verified_download(&asset, &target).unwrap_err();
        assert!(error.contains("not from this project's releases"));
        assert!(!target.exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn the_appimage_updater_refuses_a_non_appimage() {
        let asset = InstallerAsset {
            name: "Lifting-Plan-Calculator-9.9.9-windows-x86_64-setup.exe".to_string(),
            url: "https://github.com/zhafribs/lifting-plan-calculator/releases/download/v9.9.9/x-setup.exe".to_string(),
        };
        let error = install(&asset).unwrap_err();
        assert!(error.contains("not a Linux AppImage"));
    }

    #[test]
    fn the_appimage_helper_keeps_the_old_file_until_the_new_one_starts() {
        assert!(APPIMAGE_UPDATE_SCRIPT.contains("backup=\"${target}.old\""));
        assert!(APPIMAGE_UPDATE_SCRIPT.contains("rm -f \"$backup\""));
        assert!(APPIMAGE_UPDATE_SCRIPT.contains("mv \"$backup\" \"$target\""));
    }

    #[test]
    fn the_manifest_for_an_asset_sits_beside_it() {
        assert_eq!(
            manifest_url("https://github.com/o/r/releases/download/v1/x-setup.exe").as_deref(),
            Some("https://github.com/o/r/releases/download/v1/SHA256SUMS.txt")
        );
        assert_eq!(manifest_url("no-slash"), None);
    }

    #[test]
    fn a_digest_is_lowercase_sha256() {
        // SHA-256 of the empty string.
        assert_eq!(digest(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

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
