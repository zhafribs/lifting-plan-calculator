#!/usr/bin/env bash
# Build the Lifting Plan Calculator AppImage on this machine.
#
# NOTE: this links against the host's glibc and bundles the host's GTK/WebKitGTK
# stack, so the result runs on distributions at least as new as the host. For a
# file that runs on Ubuntu 22.04+ and any current Fedora/Arch, use
# `./build-appimage-portable.sh` instead (it builds inside an Ubuntu 22.04
# userspace with bubblewrap).
#
# Usage:  ./build-appimage.sh
# Result: dist/Lifting-Plan-Calculator-<version>-linux-x86_64.AppImage
#         dist/Lifting-Plan-Calculator-<version>-linux-x86_64.AppImage.zsync

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$REPO_ROOT/src-tauri"

if ! command -v cargo >/dev/null 2>&1; then
  echo "==> ERROR: cargo is not on PATH. Install Rust first: https://rustup.rs" >&2
  exit 1
fi

# The Tauri CLI is the runner; a plain `cargo tauri` works once installed.
if ! cargo tauri --version >/dev/null 2>&1; then
  echo "==> Installing the Tauri CLI (cargo install tauri-cli --locked)"
  cargo install tauri-cli --locked
fi

# The PDF export renders with the bundle's own headless Chromium. A slim build
# (BUNDLE_PDF_RENDERER=0) leaves it out and uses a system Chromium/Chrome
# instead; that is roughly 100 MB smaller.
if [ "${BUNDLE_PDF_RENDERER:-1}" = "1" ]; then
  "$REPO_ROOT/scripts/fetch-pdf-renderer.sh"
else
  echo "==> PDF renderer: not bundled (slim build); a system Chromium/Chrome is used"
  RENDERER_DIR="$REPO_ROOT/src-tauri/pdf-renderer/chrome-headless-shell-linux64"
  SLIM_BACKUP="$REPO_ROOT/src-tauri/.pdf-renderer-slim-backup"
  if [ -d "$RENDERER_DIR" ]; then
    mv "$RENDERER_DIR" "$SLIM_BACKUP"
    trap 'mv "$SLIM_BACKUP" "$RENDERER_DIR"' EXIT
  fi
fi

echo "==> Building the AppImage (release)"
cargo tauri build --bundles appimage

VERSION="$(python3 -c "import json; print(json.load(open('tauri.conf.json'))['version'])")"
BUNDLE_DIR="target/release/bundle/appimage"
OUT_DIR="$REPO_ROOT/dist"
mkdir -p "$OUT_DIR"

# The bundler names the file after the product ("Lifting Plan Calculator_x.y.z_amd64.AppImage");
# the dist copy uses the project's own convention.
SOURCE="$(find "$BUNDLE_DIR" -maxdepth 1 -name '*.AppImage' | head -1)"
if [ -z "$SOURCE" ]; then
  echo "==> ERROR: no AppImage was produced under $BUNDLE_DIR" >&2
  exit 1
fi
ASSET="Lifting-Plan-Calculator-${VERSION}-linux-x86_64.AppImage"
OUT="$OUT_DIR/$ASSET"
cp "$SOURCE" "$OUT"
chmod +x "$OUT"

echo "==> Built $OUT"
python3 - "$OUT" <<'PY'
import hashlib, os, sys
path = sys.argv[1]
print("size:", f"{os.path.getsize(path) / 1048576:.1f} MB")
print("sha256:", hashlib.sha256(open(path, "rb").read()).hexdigest())
PY

# Delta-update sidecar (used by AppImageUpdate / appimageupdatetool).
"$REPO_ROOT/packaging/make-zsync.sh" "$OUT" "$VERSION"
