#!/usr/bin/env bash
# Write the .zsync delta-update sidecar for a built AppImage.
#
# zsyncmake records the URL the updater should fetch the *whole* file from, so
# AppImageUpdate / appimageupdatetool can later download only the blocks that
# changed between two releases instead of the whole ~78 MB.
#
# Usage: packaging/make-zsync.sh <path/to/AppImage> <version>
#   <version> is the release version without the leading "v" (e.g. 2.2.0); it
#   is used to build the GitHub release download URL.

set -euo pipefail

APPIMAGE="${1:?usage: make-zsync.sh <AppImage> <version>}"
VERSION="${2:?usage: make-zsync.sh <AppImage> <version>}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ ! -f "$APPIMAGE" ]; then
  echo "==> ERROR: no such AppImage: $APPIMAGE" >&2
  exit 1
fi

if ! command -v zsyncmake >/dev/null 2>&1; then
  echo "==> zsyncmake not found; skipping the .zsync sidecar." >&2
  echo "    Install it with your package manager (Debian/Ubuntu: 'zsync')." >&2
  exit 0
fi

# Derive owner/repo from the git remote, e.g.
#   git@github.com:owner/repo.git     -> owner/repo
#   https://github.com/owner/repo.git -> owner/repo
REMOTE="$(git -C "$REPO_ROOT" remote get-url origin 2>/dev/null || true)"
SLUG="$(printf '%s' "$REMOTE" | sed -E 's#^(git@github.com:|https://github.com/)##; s#\.git$##')"
if [ -z "$SLUG" ]; then
  SLUG="zhafribs/lifting-plan-calculator"
fi

ASSET="$(basename "$APPIMAGE")"
URL="https://github.com/${SLUG}/releases/download/v${VERSION}/${ASSET}"

zsyncmake -u "$URL" -o "$APPIMAGE.zsync" "$APPIMAGE"

echo "==> Built $APPIMAGE.zsync"
echo "    target: $URL"
