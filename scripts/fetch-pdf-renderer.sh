#!/usr/bin/env bash
# Fetch the headless Chromium the PDF export renders with, so the AppImage
# carries its own renderer and needs no browser installed on the target.
#
# Windows ships Edge, so this is Linux-only and a no-op elsewhere. The version
# and the archive's checksum are pinned: a different download is a failure, not
# an "update".
#
# Usage:  ./scripts/fetch-pdf-renderer.sh
# Result: src-tauri/pdf-renderer/chrome-headless-shell-linux64/chrome-headless-shell

set -euo pipefail

VERSION="157.0.8103.78"
ARCHIVE_SHA256="f90329720fb61e5a76da9b6b509b6ad12a484c9c5df0359403e935177c6d153a"
ARCHIVE="chrome-headless-shell-linux64.zip"
URL="https://storage.googleapis.com/chrome-for-testing-public/${VERSION}/linux64/${ARCHIVE}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="$ROOT/src-tauri/pdf-renderer"
BIN="$DEST/chrome-headless-shell-linux64/chrome-headless-shell"

if [ "$(uname -s)" != "Linux" ]; then
  echo "==> PDF renderer: not Linux, using the platform browser instead"
  exit 0
fi

if [ -x "$BIN" ] && "$BIN" --version >/dev/null 2>&1; then
  echo "==> PDF renderer: $("$BIN" --version)"
  exit 0
fi

if ! command -v unzip >/dev/null 2>&1; then
  echo "==> ERROR: unzip is required to unpack the PDF renderer." >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "==> PDF renderer: fetching Chrome for Testing ${VERSION} (~128 MB)"
curl -sSfL -o "$TMP/$ARCHIVE" "$URL"
echo "$ARCHIVE_SHA256  $TMP/$ARCHIVE" | sha256sum -c - >/dev/null

mkdir -p "$DEST"
unzip -q "$TMP/$ARCHIVE" -d "$DEST"
chmod +x "$BIN"

echo "==> PDF renderer: $("$BIN" --version) at $BIN"
