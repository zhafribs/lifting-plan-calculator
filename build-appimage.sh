#!/usr/bin/env bash
# Build the Linux packages on this machine: AppImage, .deb and .rpm.
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
#         dist/Lifting-Plan-Calculator-<version>-linux-amd64.deb
#         dist/Lifting-Plan-Calculator-<version>-linux-x86_64.rpm
#
# The Arch package and the Flatpak are built from the same release binary by
# `packaging/make-arch-package.sh` and `packaging/make-flatpak.sh`.

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

# The PDF export prints with the app's own WebKitGTK view, so the bundle carries
# no browser and needs none installed on the target machine.

echo "==> Building the AppImage, .deb and .rpm (release)"
# linuxdeploy is an AppImage itself; FUSE is not present on every machine.
export APPIMAGE_EXTRACT_AND_RUN=1
cargo tauri build --bundles appimage,deb,rpm

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

# The distribution packages, with the project's own naming: the bundler names
# them after the product ("Lifting Plan Calculator_x.y.z_amd64.deb").
copy_bundle() {
  local pattern="$1" target="$2"
  local source
  # The bundle directory keeps earlier versions too; take the newest file.
  source="$(find "$REPO_ROOT/src-tauri/target/release/bundle" -name "$pattern" \
    -printf '%T@\t%p\n' | sort -n | tail -1 | cut -f2-)"
  if [ -z "$source" ]; then
    echo "==> ERROR: $pattern was not produced." >&2
    exit 1
  fi
  cp "$source" "$OUT_DIR/$target"
  echo "==> Built $OUT_DIR/$target"
}
copy_bundle '*.deb' "Lifting-Plan-Calculator-${VERSION}-linux-amd64.deb"
copy_bundle '*.rpm' "Lifting-Plan-Calculator-${VERSION}-linux-x86_64.rpm"
