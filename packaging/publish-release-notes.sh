#!/usr/bin/env bash
# Republish the body of existing GitHub releases from docs/releases/<tag>.md.
#
# This exists for one job: the 13 releases cut before per-tag notes existed are
# republished so every page is its own body in the current shape. Two of them
# repeated earlier notes before the first run: v2.5.0 carried 2.4.0, and v2.5.1
# carried 2.5.0 and 2.4.0.
# Running this rewrites each page's body from its own file. It never touches
# assets, names or tags, and it is idempotent.
#
#   packaging/publish-release-notes.sh              # show what would change
#   packaging/publish-release-notes.sh --yes        # rewrite the 13 bodies
#   packaging/publish-release-notes.sh --yes v2.5.1 # one release
#
# Requires an authenticated `gh` with write access to the repository.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NOTES_DIR="$REPO_ROOT/docs/releases"
REPO="${REPO:-zhafribs/lifting-plan-calculator}"

APPLY=0
TAGS=()

for arg in "$@"; do
  case "$arg" in
    --yes) APPLY=1 ;;
    -h | --help)
      awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    v*) TAGS+=("$arg") ;;
    *)
      echo "error: unknown argument '$arg' (expected --yes, --help, or a vX.Y.Z tag)" >&2
      exit 2
      ;;
  esac
done

cd "$REPO_ROOT"

if ! command -v gh >/dev/null 2>&1; then
  echo "error: gh is not installed" >&2
  exit 1
fi

# Default to every tag that has a notes file, oldest first.
if [ "${#TAGS[@]}" -eq 0 ]; then
  while IFS= read -r tag; do
    TAGS+=("$tag")
  done < <(git tag --list 'v*' --sort=v:refname)
fi

changed=0
skipped=0

for tag in "${TAGS[@]}"; do
  file="$NOTES_DIR/$tag.md"
  if [ ! -s "$file" ]; then
    echo "skip $tag: $file is missing or empty" >&2
    skipped=$((skipped + 1))
    continue
  fi

  if ! gh release view "$tag" --repo "$REPO" >/dev/null 2>&1; then
    echo "skip $tag: no release exists for this tag" >&2
    skipped=$((skipped + 1))
    continue
  fi

  current="$(gh release view "$tag" --repo "$REPO" --json body -q .body)"
  if [ "$current" = "$(cat "$file")" ]; then
    echo "ok   $tag: body already matches $file"
    continue
  fi

  if [ "$APPLY" -eq 1 ]; then
    gh release edit "$tag" --repo "$REPO" --notes-file "$file"
    echo "done $tag: body replaced from docs/releases/$tag.md"
    changed=$((changed + 1))
  else
    echo "would $tag: replace the body with docs/releases/$tag.md"
    changed=$((changed + 1))
  fi
done

if [ "$APPLY" -eq 1 ]; then
  echo "republished $changed release page(s); skipped $skipped"
else
  echo "$changed release page(s) would change; skipped $skipped"
  echo "Re-run with --yes to write."
fi
