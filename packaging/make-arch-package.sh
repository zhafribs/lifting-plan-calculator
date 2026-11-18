#!/usr/bin/env bash
# Build the Arch Linux package (x86_64.pkg.tar.zst) from the release binary.
#
# The binary is the one `cargo tauri build` produced: this packages it, it does
# not compile. The package depends on the distribution's own GTK/WebKit, the
# same way the .deb and .rpm do.
#
# Usage:  ./packaging/make-arch-package.sh
# Result: dist/Lifting-Plan-Calculator-<version>-linux-x86_64.pkg.tar.zst

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(python3 -c "import json; print(json.load(open('$ROOT/src-tauri/tauri.conf.json'))['version'])")"
BIN="$ROOT/src-tauri/target/release/lifting-plan-calculator"
OUT="$ROOT/dist/Lifting-Plan-Calculator-${VERSION}-linux-x86_64.pkg.tar.zst"

if ! command -v makepkg >/dev/null 2>&1; then
  echo "==> ERROR: makepkg is required (Arch Linux or an archlinux container)." >&2
  exit 1
fi
if [ ! -x "$BIN" ]; then
  echo "==> ERROR: $BIN is missing; build the release binary first." >&2
  exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cp "$ROOT/packaging/arch/PKGBUILD" "$WORK/PKGBUILD"
cp "$BIN" "$WORK/lifting-plan-calculator"
cp "$ROOT/packaging/lifting-plan-calculator.desktop" "$WORK/"
cp "$ROOT/src-tauri/icons/128x128.png" "$WORK/lifting-plan-calculator.png"
cp "$ROOT/LICENSE" "$ROOT/THIRD_PARTY_NOTICES.md" "$WORK/"
sed -i "s/^pkgver=.*/pkgver=${VERSION}/" "$WORK/PKGBUILD"

echo "==> Building the Arch package for ${VERSION}"
(cd "$WORK" && makepkg -f --nocheck)

mkdir -p "$ROOT/dist"
cp "$WORK/lifting-plan-calculator-${VERSION}-1-x86_64.pkg.tar.zst" "$OUT"
echo "==> Built $OUT"
