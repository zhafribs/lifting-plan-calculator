#!/usr/bin/env bash
# Verify the vendored third-party code is the copy that was reviewed.
#
# The PDF page view runs a minified PDF parser inside the app, so the two files
# are pinned by hash: a different file is a failure here, not an update.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Every third-party file the app ships, each against the manifest committed
# beside it. The manifests are in the tree, so this detects drift and makes a
# replacement visible in review; it is not an external trust anchor.
for dir in ui/vendor/pdfjs ui/vendor/katex ui/fonts; do
  if [ ! -f "$dir/CHECKSUMS.txt" ]; then
    echo "error: $dir has no CHECKSUMS.txt" >&2
    exit 1
  fi
  if ! (cd "$dir" && sha256sum --check --status CHECKSUMS.txt); then
    echo "error: $dir does not match its CHECKSUMS.txt" >&2
    (cd "$dir" && sha256sum --check CHECKSUMS.txt) >&2 || true
    echo "       replace the files deliberately, then regenerate that manifest" >&2
    exit 1
  fi
  echo "vendored $(basename "$dir"): OK"
done
