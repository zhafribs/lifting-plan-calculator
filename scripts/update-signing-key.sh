#!/usr/bin/env bash
# Create the key that signs each release's SHA256SUMS.txt, and say how to hand it
# to CI.
#
# The private key never leaves this machine or GitHub's secret store: it is
# written to your config directory with mode 0600, and only the public half is
# printed and committed.
#
# Run once:
#   ./scripts/update-signing-key.sh
# Then:
#   gh secret set UPDATE_SIGNING_KEY < "$KEY_PATH"
#   git add src-tauri/update-public-key.txt && git commit
#
# From that release on, an installer whose manifest is not signed by this key is
# refused by the app.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KEY_PATH="${XDG_CONFIG_HOME:-$HOME/.config}/lifting-plan-calculator/update-signing.key"
PUBLIC_FILE="$ROOT/src-tauri/update-public-key.txt"

if [ -s "$KEY_PATH" ]; then
  echo "error: $KEY_PATH already exists." >&2
  echo "       Deleting it invalidates every published signature; only do that" >&2
  echo "       deliberately, and publish a new app build with the new key." >&2
  exit 1
fi

mkdir -p "$(dirname "$KEY_PATH")"
PUBLIC="$(cd "$ROOT" && cargo run --quiet --locked \
  --manifest-path tools/sign-manifest/Cargo.toml -- generate "$KEY_PATH")"

printf '%s\n' "$PUBLIC" > "$PUBLIC_FILE"
echo "Private key: $KEY_PATH (mode 0600, never commit it)"
echo "Public key:  $PUBLIC_FILE"
echo
echo "Next:"
echo "  gh secret set UPDATE_SIGNING_KEY < \"$KEY_PATH\""
echo "  git add src-tauri/update-public-key.txt && git commit -m \"build: sign releases\""
echo
echo "Keep a backup of $KEY_PATH somewhere safe: without it you cannot sign the"
echo "next release, and every installed copy will refuse an unsigned manifest."
