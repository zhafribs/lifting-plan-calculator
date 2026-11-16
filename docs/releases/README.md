# Release notes

One file per tag, named after the tag: `v2.5.1.md` is the body of the `v2.5.1`
release.

The `verify-notes` job in `.github/workflows/release.yml` runs
`packaging/check-release-notes.sh --tag "$GITHUB_REF_NAME"` **before either
platform build**. It fails the release if the tag is not a plain semantic
version, if `src-tauri/tauri.conf.json` names a different version, if
`docs/releases/<tag>.md` is missing or malformed, if that file names another
version, or if no accepted council record names the tag. The publish job then
uses exactly that file as the body, so a page cut from v2.5.2 onward cannot carry
an earlier release's notes.

The 13 pages already published (v2.0.0 … v2.5.1) were written before this
process. Two of them used to repeat earlier notes — v2.5.0 carried 2.4.0, and
v2.5.1 carried 2.5.0 and 2.4.0; the other eleven are single-release bodies.
`packaging/publish-release-notes.sh` republished all 13 from this directory on
13 November 2026; it only rewrites the body, so it is safe to run again.

## Writing one

Write the file before pushing the tag. Keep the shape of the existing files:

1. **Title** — `# Lifting Plan Calculator <version>`.
2. **Line of facts** — release date, platforms, then the release's headline in
   one sentence. If a change corrects reference data (a chart, an example
   workbook), that leads — it outranks a layout change.
3. **Highlights** — the two or three changes worth reading first, each with a
   bold lead-in.
4. **Added / Changed / Fixed** — everything else, grouped. Keep the words from
   `CHANGELOG.md` so the two never disagree, and do not restate a highlight.
5. **Compatibility and upgrade notes** — only where behaviour changed, a file
   name changed or a platform requires something.
6. **Download** — a table of the assets the workflow attaches for that tag,
   named with that tag's version. `SHA256SUMS.txt` covers all of them.
7. **Scope** — the one-line reminder that this is a calculation aid, not a
   certified lift plan, linking to *Scope of use* in the readme.
8. **Footer** — the five links, unchanged.

Do not paste the previous release's notes. Earlier versions of this project
did, and every release page repeated the two before it. The checker rejects a
file that names another version anywhere it lists assets, and one that carries a
version heading below its title.

## What a release page deliberately does not do

- **No screenshots.** The readme carries them; a release page is the change, not
  the product tour.
- **No changelog inside the app.** The update reminder's *Release notes* button
  opens the GitHub release page.
- **No Authenticode.** The installers carry no Windows code-signing certificate,
  so SmartScreen shows "Unknown publisher". The update path does not rely on it:
  the release is signed with the project's own Ed25519 key instead.

## Signed releases

`SHA256SUMS.txt` is signed into `SHA256SUMS.txt.sig` with an Ed25519 key. The
public half is `src-tauri/update-public-key.txt` and is compiled into every
build; the private half is the `UPDATE_SIGNING_KEY` repository secret and a file
on the maintainer's machine. The app verifies the signature before it trusts any
digest, so an unsigned or wrongly signed manifest is refused rather than run.

**Creating the key (once).** Run `./scripts/update-signing-key.sh`. It writes
`~/.config/lifting-plan-calculator/update-signing.key` (mode 0600), writes the
public key into the repository, and prints the `gh secret set` command. Commit
the public key; from the next release the workflow signs and the app enforces.

**Back the private key up somewhere safe.** Without it no release can be signed,
and every installed copy refuses an unsigned manifest. If it is lost, recovery
means publishing a build with a new key that users install manually, because the
old builds will not accept it.

**Rotation** is the same ceremony with the new key committed in the release that
introduces it; the previous key only signs earlier releases.

## Checking and publishing

```bash
./packaging/check-release-notes.sh --self-test   # the checker still catches the defect
./packaging/check-release-notes.sh               # every tag against this directory
./packaging/check-release-notes.sh --tag v2.5.2  # one tag, as the release job does
```

CI runs the first two on every push and pull request. A notes file may land
before its tag; the checker warns until the tag exists.
