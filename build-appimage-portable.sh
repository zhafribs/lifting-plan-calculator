#!/usr/bin/env bash
# Build the AppImage inside an Ubuntu 22.04 userspace with bubblewrap, so the
# result runs on **Ubuntu 22.04+, Fedora 36+ and Arch** rather than only on a
# host as new as this one.
#
# Why this exists: a plain `./build-appimage.sh` links against the *host's*
# glibc (and bundles the host's GTK/WebKitGTK, which were themselves built
# against it), so an AppImage built on a rolling-release system refuses to run
# on older LTS distributions — "version `GLIBC_2.44' not found". Building
# inside an Ubuntu 22.04 (glibc 2.35) userspace fixes the floor.
#
# Requirements: `bwrap` (bubblewrap) and an internet connection for the first
# run. No root and no Docker needed.
#
# Usage:  ./build-appimage-portable.sh
# Result: dist/lifting-plan-calculator-V<version>.appimage
#
# (A Dockerfile that does the same thing is in packaging/Dockerfile for
#  environments where Docker is preferred.)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/lifting-plan-calculator-build"
ROOTFS="$CACHE_DIR/ubuntu-22.04-rootfs"
ROOTFS_TARBALL="$CACHE_DIR/ubuntu-base-22.04-base-amd64.tar.gz"
BUILD_DIR="$CACHE_DIR/build"
ROOTFS_URL="https://cdimage.ubuntu.com/ubuntu-base/releases/22.04/release/ubuntu-base-22.04-base-amd64.tar.gz"

if ! command -v bwrap >/dev/null 2>&1; then
  echo "==> ERROR: bwrap (bubblewrap) is not installed." >&2
  echo "    Install it, or build with Docker using packaging/Dockerfile." >&2
  exit 1
fi

mkdir -p "$CACHE_DIR" "$BUILD_DIR"

if [ ! -d "$ROOTFS/usr" ]; then
  echo "==> Fetching the Ubuntu 22.04 base rootfs (one time, ~30 MB)"
  curl -L --fail -o "$ROOTFS_TARBALL" "$ROOTFS_URL"
  mkdir -p "$ROOTFS"
  tar -xzf "$ROOTFS_TARBALL" -C "$ROOTFS"
fi

echo "==> Building inside Ubuntu 22.04 (glibc 2.35)"

# A tiny stub for the ownership tools: inside a bubblewrap user namespace no
# group beyond gid 0 is mapped, so `chown` and `dpkg-statoverride` fail with
# EINVAL on the maintainer scripts that set up setuid helpers (dbus' launcher,
# for example) even though those helpers are never used by a build. The stub
# makes them succeed silently; every file ends up owned by the invoking user.
STUB="$(mktemp)"
printf '#!/bin/sh\nexit 0\n' > "$STUB"
chmod +x "$STUB"
trap 'rm -f "$STUB"' EXIT

bwrap \
  --bind "$ROOTFS" / \
  --dev /dev \
  --proc /proc \
  --tmpfs /tmp \
  --tmpfs /run \
  --bind "$BUILD_DIR" /build \
  --bind "$REPO_ROOT" /work \
  --ro-bind "$STUB" /bin/chown \
  --ro-bind "$STUB" /usr/bin/chown \
  --ro-bind "$STUB" /usr/sbin/dpkg-statoverride \
  --ro-bind /etc/resolv.conf /etc/resolv.conf \
  --unshare-user --uid 0 --gid 0 \
  --unshare-pid \
  --die-with-parent \
  bash -euxo pipefail -c '
    export DEBIAN_FRONTEND=noninteractive
    export HOME=/root
    # The apt sandbox drops privileges to the _apt user, which is not possible
    # inside a bubblewrap user namespace; run apt as root here. And the user
    # namespace maps no groups beyond gid 0, so the ownership changes dpkg
    # would make fail with EINVAL anyway — `--force-not-root` makes it skip
    # them (the files end up owned by the invoking user either way).
    apt-get -o APT::Sandbox::User=root update
    apt-get -o APT::Sandbox::User=root -o Dpkg::Options::=--force-not-root \
      install -y --no-install-recommends \
      build-essential curl wget git ca-certificates pkg-config file patchelf \
      python3 xz-utils \
      libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libssl-dev \
      zlib1g-dev libsoup-3.0-dev

    if [ ! -x /root/.cargo/bin/rustup ]; then
      curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable
    fi
    . /root/.cargo/env

    if [ ! -x /root/.cargo/bin/cargo-tauri ]; then
      cargo install tauri-cli --locked
    fi

    export CARGO_TARGET_DIR=/build/target
    cd /work/src-tauri
    cargo tauri build --bundles appimage
  '

VERSION="$(python3 -c "import json; print(json.load(open('$REPO_ROOT/src-tauri/tauri.conf.json'))['version'])")"
SOURCE="$(find "$BUILD_DIR/target/release/bundle/appimage" -maxdepth 1 -name '*.AppImage' | head -1)"
if [ -z "$SOURCE" ]; then
  echo "==> ERROR: no AppImage was produced." >&2
  exit 1
fi
mkdir -p "$REPO_ROOT/dist"
OUT="$REPO_ROOT/dist/lifting-plan-calculator-V${VERSION}.appimage"
cp "$SOURCE" "$OUT"
chmod +x "$OUT"

echo "==> Built $OUT"
python3 - "$OUT" <<'PY'
import hashlib, os, sys
path = sys.argv[1]
print("size:", f"{os.path.getsize(path) / 1048576:.1f} MB")
print("sha256:", hashlib.sha256(open(path, "rb").read()).hexdigest())
PY
