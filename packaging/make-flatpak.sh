#!/usr/bin/env bash
# Build the single-file Flatpak bundle from the release binary.
#
# The Flatpak uses the GNOME runtime, whose GTK 3 and WebKitGTK 4.1 are what the
# release binary links against, so nothing is compiled here. The runtime and SDK
# must already be installed (the release workflow installs them; a developer can
# run `flatpak install flathub org.gnome.Platform//48 org.gnome.Sdk//48`).
#
# Usage:  ./packaging/make-flatpak.sh
# Result: dist/Lifting-Plan-Calculator-<version>-linux-x86_64.flatpak

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(python3 -c "import json; print(json.load(open('$ROOT/src-tauri/tauri.conf.json'))['version'])")"
BIN="$ROOT/src-tauri/target/release/lifting-plan-calculator"
APP_ID="io.github.zhafribs.LiftingPlanCalculator"
OUT="$ROOT/dist/Lifting-Plan-Calculator-${VERSION}-linux-x86_64.flatpak"

# The Builder app carries flatpak-builder and appstream-compose together, so it
# is preferred: a distribution's flatpak-builder may have no appstream-compose to
# call (Ubuntu 22.04 ships neither), and the metainfo step then fails.
if flatpak info org.flatpak.Builder >/dev/null 2>&1; then
  BUILDER=(flatpak run --filesystem=/tmp org.flatpak.Builder)
elif command -v flatpak-builder >/dev/null 2>&1; then
  BUILDER=(flatpak-builder)
else
  echo "==> ERROR: install org.flatpak.Builder or flatpak-builder." >&2
  exit 1
fi
if [ ! -x "$BIN" ]; then
  echo "==> ERROR: $BIN is missing; build the release binary first." >&2
  exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cp "$ROOT/packaging/flatpak/io.github.zhafribs.LiftingPlanCalculator.yml" "$WORK/"
cp "$ROOT/packaging/flatpak/io.github.zhafribs.LiftingPlanCalculator.desktop" "$WORK/"
cp "$ROOT/packaging/flatpak/io.github.zhafribs.LiftingPlanCalculator.metainfo.xml" "$WORK/"
cp "$BIN" "$WORK/lifting-plan-calculator"
cp "$ROOT/src-tauri/icons/128x128.png" \
  "$WORK/io.github.zhafribs.LiftingPlanCalculator.png"
cp "$ROOT/LICENSE" "$ROOT/THIRD_PARTY_NOTICES.md" "$WORK/"

echo "==> Building the Flatpak for ${VERSION}"
"${BUILDER[@]}" --force-clean --state-dir="$WORK/state" --repo="$WORK/repo" \
  "$WORK/build" "$WORK/io.github.zhafribs.LiftingPlanCalculator.yml"

mkdir -p "$ROOT/dist"
flatpak build-bundle "$WORK/repo" "$OUT" "$APP_ID" --arch=x86_64
echo "==> Built $OUT"
