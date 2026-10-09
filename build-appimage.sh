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
# Result: dist/lifting-plan-calculator-V<version>.appimage

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

echo "==> Building the AppImage (release)"
cargo tauri build --bundles appimage

VERSION="$(python3 -c "import json; print(json.load(open('tauri.conf.json'))['version'])")"
BUNDLE_DIR="target/release/bundle/appimage"
OUT_DIR="$REPO_ROOT/dist"
mkdir -p "$OUT_DIR"

# The bundler names the file after the product ("Lifting Plan Calculator_x.y.z_amd64.AppImage");
# the dist copy uses the project's own lowercase naming.
SOURCE="$(find "$BUNDLE_DIR" -maxdepth 1 -name '*.AppImage' | head -1)"
if [ -z "$SOURCE" ]; then
  echo "==> ERROR: no AppImage was produced under $BUNDLE_DIR" >&2
  exit 1
fi
OUT="$OUT_DIR/lifting-plan-calculator-V${VERSION}.appimage"
cp "$SOURCE" "$OUT"
chmod +x "$OUT"

echo "==> Built $OUT"
python3 - "$OUT" <<'PY'
import hashlib, sys
path = sys.argv[1]
print("size:", f"{__import__('os').path.getsize(path) / 1048576:.1f} MB")
print("sha256:", hashlib.sha256(open(path, "rb").read()).hexdigest())
PY
