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
  # A file nobody listed would ship unreviewed: the manifest must name every
  # file in the directory, not only the ones it happens to check.
  # The auto-render wrapper is ours; it sits beside KaTeX because the report
  # loads it from the same folder, and it is reviewed as app code. Exempt it in
  # that one directory only: the same name anywhere else is still unlisted.
  listed="$(awk '{ $1 = ""; sub(/^[ \t]+/, ""); sub(/^\.\//, ""); print }' "$dir/CHECKSUMS.txt" | sort)"
  if [ "$dir" = "ui/vendor/katex" ]; then
    actual="$(cd "$dir" && find . -type f ! -name CHECKSUMS.txt ! -name lpc-autorender.js \
      | sed 's|^\./||' | sort)"
  else
    actual="$(cd "$dir" && find . -type f ! -name CHECKSUMS.txt | sed 's|^\./||' | sort)"
  fi
  if [ "$listed" != "$actual" ]; then
    echo "error: $dir has files its CHECKSUMS.txt does not list, or lists missing files" >&2
    diff <(printf '%s\n' "$listed") <(printf '%s\n' "$actual") >&2 || true
    exit 1
  fi
  echo "vendored $(basename "$dir"): OK"
done
