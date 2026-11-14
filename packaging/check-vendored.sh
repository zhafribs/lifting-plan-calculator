#!/usr/bin/env bash
# Verify the vendored third-party code is the copy that was reviewed.
#
# The PDF page view runs a minified PDF parser inside the app, so the two files
# are pinned by hash: a different file is a failure here, not an update.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/ui/vendor/pdfjs"

if ! sha256sum --check --status CHECKSUMS.txt; then
  echo "error: ui/vendor/pdfjs does not match CHECKSUMS.txt" >&2
  sha256sum --check CHECKSUMS.txt >&2 || true
  echo "       replace the files deliberately, then regenerate CHECKSUMS.txt" >&2
  exit 1
fi

echo "vendored pdf.js: OK"
