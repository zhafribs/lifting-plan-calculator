#!/usr/bin/env bash
# Check release notes: one well-formed, single-version file per tag.
#
#   packaging/check-release-notes.sh              # every tag vs docs/releases/
#   packaging/check-release-notes.sh --tag v2.5.1 # one tag, for the release job
#   packaging/check-release-notes.sh --self-test  # prove the checks still fail
#
# This exists because the repository once published cumulative notes: the
# v2.5.1 page carried 2.5.0 and 2.4.0 as well. A body that names another
# version is a failure here, not a style opinion.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NOTES_DIR="docs/releases"
DECISIONS_DIR="docs/council/decisions"

# vX.Y.Z, or a vX.Y.Z-alpha/beta/rc pre-release. Deliberately strict: the tag is
# used as a file name and as a GitHub release input.
TAG_RE='^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-(alpha|beta|rc)(\.[0-9A-Za-z]+)*)?$'
ASSET_RE='\.(AppImage|appimage|msi|exe)'
# A version as it appears in a file name, including a pre-release suffix, so
# `2.5.2-alpha.1` is one token rather than a mismatch against `2.5.2`.
VERSION_TOKEN='(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-(alpha|beta|rc)(\.[0-9A-Za-z]+)*)?'

FAILED=0
WARNED=0
CHECKED=0

fail() { echo "error: $*" >&2; FAILED=1; }
warn() { echo "warning: $*" >&2; WARNED=1; }

# The `## Download` section, without the heading itself.
download_section() {
  awk '/^## +Download/{inside=1; next} /^## /{inside=0} inside' "$1"
}

# Validate one notes file against the version it claims to describe.
validate_file() {
  local file="$1" version="$2" label="$3" heading section assets line token lineno
  local found_tokens before

  before="$FAILED"
  CHECKED=$((CHECKED + 1))

  if [ ! -s "$file" ]; then
    fail "$file is missing or empty"
    return
  fi

  heading="$(head -1 "$file")"
  if [ "$heading" != "# Lifting Plan Calculator $version" ]; then
    fail "$file must open with '# Lifting Plan Calculator $version' (found: $heading)"
  fi

  # A version heading below the title means another release's section was
  # carried in, whatever prose precedes the number ("## 2.5.0",
  # "## Release 2.4.0", "## v2.5.0 (superseded)"). This is the defect the file
  # exists to prevent.
  if lineno="$(grep -nE '^## +([^0-9]*[^0-9.])?[0-9]+\.[0-9]+\.[0-9]+' "$file" | head -1)"; then
    if [ -n "$lineno" ]; then
      fail "$file carries another release's section at line ${lineno%%:*} — one file per release, nothing carried in"
    fi
  fi

  if ! grep -qE '^## +Download' "$file"; then
    fail "$file has no '## Download' section"
    return
  fi

  section="$(download_section "$file")"

  assets="$(printf '%s\n' "$section" | grep -Ec "$ASSET_RE" || true)"
  if [ "${assets:-0}" -eq 0 ]; then
    fail "$file names no asset in its '## Download' section"
  fi

  # Every version named where the assets are named must be this release's, so a
  # copied download table is caught rather than shipped. The token includes a
  # pre-release suffix, so a `2.5.2-alpha.1` asset is not read as `2.5.2`.
  found_tokens="$(printf '%s\n' "$section" |
    grep -oE "(^|[^0-9.])${VERSION_TOKEN}" |
    sed -E 's/^[^0-9]*//' || true)"
  for token in $found_tokens; do
    if [ "$token" != "$version" ]; then
      fail "$file names version $token where it lists assets; this file is $version"
    fi
  done

  if [ "$FAILED" -eq "$before" ]; then
    echo "ok  $label"
  fi
}

# Every tag has a file; every file is named after a valid tag; every file passes.
validate_repo() {
  local root="$1" tag file name count_tags=0 count_files=0

  cd "$root"

  if ! git rev-parse --git-dir >/dev/null 2>&1; then
    echo "error: $root is not a git repository — run this from a clone whose tags are fetched" >&2
    exit 1
  fi

  if [ "$(git rev-parse --is-shallow-repository 2>/dev/null)" = "true" ]; then
    echo "error: shallow clone, so the tags cannot be checked. Use fetch-depth: 0, then re-run." >&2
    exit 1
  fi

  while IFS= read -r tag; do
    [ -n "$tag" ] || continue
    count_tags=$((count_tags + 1))
    if [ ! -s "$NOTES_DIR/$tag.md" ]; then
      fail "tag $tag has no release notes: expected $NOTES_DIR/$tag.md"
      continue
    fi
    validate_file "$NOTES_DIR/$tag.md" "${tag#v}" "$tag"
  done < <(git tag --list 'v*')

  for file in "$NOTES_DIR"/v*.md; do
    [ -e "$file" ] || continue
    count_files=$((count_files + 1))
    name="$(basename "$file" .md)"

    if ! printf '%s' "$name" | grep -Eq "$TAG_RE"; then
      fail "$file is not named after a release tag (vX.Y.Z, or vX.Y.Z-alpha/beta/rc)"
      continue
    fi

    # A notes file may land before its tag; the tag is cut later.
    if ! git rev-parse -q --verify "refs/tags/$name" >/dev/null; then
      warn "$file has no tag yet (expected while the release is being prepared)"
    fi
  done

  echo "checked $count_files notes file(s) against $count_tags tag(s)"

  [ "$FAILED" -eq 0 ]
}

# The release job checks one tag: its notes, its version, and that the council
# accepted the change that produced it.
validate_tag() {
  validate_tag_at "$REPO_ROOT" "$1"
}

# Same, against an explicit root, so the self-test can exercise it too. Returns
# non-zero instead of exiting, so one failing fixture does not end the run.
validate_tag_at() {
  local root="$1" tag="$2" version conf_version record

  if ! printf '%s' "$tag" | grep -Eq "$TAG_RE"; then
    fail "'$tag' is not a vX.Y.Z (or vX.Y.Z-alpha/beta/rc) tag; refusing to publish."
    return 1
  fi

  version="${tag#v}"

  cd "$root"

  conf_version="$(sed -n 's/.*"version": "\([^"]*\)".*/\1/p' src-tauri/tauri.conf.json | head -1)"
  if [ "$conf_version" != "$version" ]; then
    fail "tag $tag, but src-tauri/tauri.conf.json says $conf_version — the artifacts would be named $conf_version under a $tag release."
    return 1
  fi

  validate_file "$NOTES_DIR/$tag.md" "$version" "$tag"

  # A tag must carry an accepted council record for that release, so a change to
  # a figure cannot ship without the seat that judges figures. The release line
  # must name this tag exactly, not a prefix of a longer one.
  record="$(grep -rlE "^\\*\\*Release:\\*\\* ${tag//./\\.}$" "$DECISIONS_DIR" 2>/dev/null | head -1 || true)"
  if [ -z "$record" ]; then
    fail "no council record names '**Release:** $tag' in $DECISIONS_DIR — write and accept one before tagging"
  elif ! grep -qE '^\*\*Status:\*\* accepted' "$record"; then
    fail "$record is not accepted; a tag needs an accepted record"
  fi

  echo "checked 1 notes file and its council record for $tag"

  [ "$FAILED" -eq 0 ]
}

# Negative tests: every failure this script exists for, reproduced in a
# throwaway directory with a throwaway git history. Wired into CI, so an edit
# that quietly weakens a check goes red.
self_test() {
  local tmp failures=0

  tmp="$(mktemp -d)"
  # Expand the path now: `tmp` is local to this function, and the trap runs after
  # it has gone out of scope.
  trap "rm -rf '$tmp'" EXIT

  # fixture <tag> <file body> -> prints the fixture repository's path
  fixture() {
    local dir
    dir="$tmp/f$RANDOM$RANDOM"
    mkdir -p "$dir/$NOTES_DIR" "$dir/src-tauri" "$dir/$DECISIONS_DIR"
    printf '{\n  "version": "%s"\n}\n' "${1#v}" > "$dir/src-tauri/tauri.conf.json"
    printf '%s\n' "$2" > "$dir/$NOTES_DIR/$1.md"
    printf '# record\n\n**Release:** %s\n\n**Status:** accepted\n' "$1" > "$dir/$DECISIONS_DIR/2026-01-01-test.md"
    git -C "$dir" init -q
    git -C "$dir" -c user.email=t@t -c user.name=t add -A >/dev/null
    git -C "$dir" -c user.email=t@t -c user.name=t commit -qm base >/dev/null
    git -C "$dir" tag "$1"
    printf '%s' "$dir"
  }

  # note <version> -> a well-formed page body
  note() {
    cat <<EOF
# Lifting Plan Calculator $1

**Released 1 January 2026** · Linux

A summary.

## Highlights

- **A thing.** It does a thing.

## Download

| Platform | File |
| --- | --- |
| Linux | \`Lifting-Plan-Calculator-$1-linux-x86_64.AppImage\` |
| Any | \`SHA256SUMS.txt\`
EOF
  }

  # case_ <name> <pass|fail> <fixture dir>
  case_() {
    local name="$1" expect="$2" dir="$3" status
    if ( cd "$dir" && validate_repo "$dir" >/dev/null 2>&1 ); then
      status=0
    else
      status=1
    fi
    if { [ "$expect" = pass ] && [ "$status" -ne 0 ]; } ||
      { [ "$expect" = fail ] && [ "$status" -eq 0 ]; }; then
      echo "self-test FAIL: $name (expected $expect)" >&2
      failures=$((failures + 1))
    else
      echo "self-test ok: $name"
    fi
  }

  case_ "well-formed page" pass "$(fixture v2.5.1 "$(note 2.5.1)")"

  case_ "cumulative body (the original defect)" fail "$(fixture v2.5.1 "$(note 2.5.1)

## 2.5.0

Old notes.

## Download

| Linux | \`Lifting-Plan-Calculator-2.5.0-linux-x86_64.AppImage\` |")"

  case_ "another release's body copied" fail \
    "$(fixture v2.5.0 "$(note 2.5.1 | sed 's/^# Lifting Plan Calculator 2.5.1/# Lifting Plan Calculator 2.5.0/')")"

  case_ "wrong version in the title" fail "$(fixture v2.5.1 "$(note 2.5.0)")"

  case_ "version as a substring" fail "$(fixture v2.5.1 "$(note 12.5.1)")"

  case_ "Download table with no asset" fail "$(fixture v2.5.1 "$(note 2.5.1 | sed '/\.AppImage/d')")"

  case_ "Download table naming another version" fail "$(fixture v2.5.1 "$(note 2.5.0)")"

  case_ "no Download section" fail "$(fixture v2.5.1 "$(note 2.5.1 | sed 's/^## Download/## Assets/')")"

  case_ "empty file" fail "$(fixture v2.5.1 "")"

  case_ "tag with no notes file" fail "$(
    d="$(fixture v2.5.1 "$(note 2.5.1)")"
    rm "$d/$NOTES_DIR/v2.5.1.md"
    printf '%s' "$d"
  )"

  case_ "a carried section headed '## Release 2.4.0'" fail "$(fixture v2.5.1 "$(note 2.5.1)

## Release 2.4.0

Old notes.")"

  case_ "a pre-release asset naming the full pre-release version" pass \
    "$(fixture v2.5.2-alpha.1 "$(note 2.5.2-alpha.1)")"

  # The --tag path gates both builds and publish; exercise it directly.
  tag_case() { # name, pass|fail, dir, tag
    local name="$1" expect="$2" dir="$3" tag="$4" status
    if ( validate_tag_at "$dir" "$tag" ) >/dev/null 2>&1; then
      status=0
    else
      status=1
    fi
    if { [ "$expect" = pass ] && [ "$status" -ne 0 ]; } ||
      { [ "$expect" = fail ] && [ "$status" -eq 0 ]; }; then
      echo "self-test FAIL: $name (expected $expect)" >&2
      failures=$((failures + 1))
    else
      echo "self-test ok: $name"
    fi
  }

  tag_ok="$(fixture v2.5.2 "$(note 2.5.2)")"
  tag_case "tag mode: tag, version, notes and accepted record agree" pass "$tag_ok" v2.5.2
  tag_case "tag mode: not a version tag" fail "$tag_ok" v2.5
  tag_case "tag mode: tag disagrees with tauri.conf.json" fail "$tag_ok" v2.5.3

  tag_bad_conf="$(fixture v2.5.2 "$(note 2.5.2)")"
  printf '{\n  "version": "2.5.1"\n}\n' > "$tag_bad_conf/src-tauri/tauri.conf.json"
  tag_case "tag mode: tauri.conf.json names another version" fail "$tag_bad_conf" v2.5.2

  tag_no_record="$(fixture v2.5.2 "$(note 2.5.2)")"
  rm -f "$tag_no_record/$DECISIONS_DIR/"*.md
  tag_case "tag mode: no council record" fail "$tag_no_record" v2.5.2

  tag_proposed="$(fixture v2.5.2 "$(note 2.5.2)")"
  printf '# record\n\n**Release:** v2.5.2\n\n**Status:** proposed\n' \
    > "$tag_proposed/$DECISIONS_DIR/2026-01-01-test.md"
  tag_case "tag mode: record not accepted" fail "$tag_proposed" v2.5.2

  tag_prefix="$(fixture v2.5.2 "$(note 2.5.2)")"
  printf '# record\n\n**Release:** v2.5.20\n\n**Status:** accepted\n' \
    > "$tag_prefix/$DECISIONS_DIR/2026-01-01-test.md"
  tag_case "tag mode: record names a longer tag" fail "$tag_prefix" v2.5.2

  tag_no_notes="$(fixture v2.5.2 "$(note 2.5.2)")"
  rm -f "$tag_no_notes/$NOTES_DIR/v2.5.2.md"
  tag_case "tag mode: no notes file" fail "$tag_no_notes" v2.5.2

  if [ "$failures" -ne 0 ]; then
    echo "self-test: FAILED ($failures case(s))" >&2
    exit 1
  fi
  echo "self-test: all cases held"
}

main() {
  case "${1:-}" in
    --self-test) self_test ;;
    --tag)
      [ $# -ge 2 ] || { echo "usage: $0 --tag <tag>" >&2; exit 2; }
      validate_tag "$2" || true
      ;;
    *)
      validate_repo "${1:-$REPO_ROOT}" || true
      ;;
  esac

  if [ "$FAILED" -ne 0 ]; then
    echo "release notes check: FAILED" >&2
    exit 1
  fi
  if [ "$WARNED" -ne 0 ]; then
    echo "release notes check: passed with warnings"
  else
    echo "release notes check: OK"
  fi
}

# Sourced by the self-test, not when executed.
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  main "$@"
fi
