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
- **No signing.** `SHA256SUMS.txt` travels over the same connection as the
  assets, so it catches a corrupted download, not a compromised release. Windows
  shows its "Unknown publisher" warning for the installers.

## Checking and publishing

```bash
./packaging/check-release-notes.sh --self-test   # the checker still catches the defect
./packaging/check-release-notes.sh               # every tag against this directory
./packaging/check-release-notes.sh --tag v2.5.2  # one tag, as the release job does
```

CI runs the first two on every push and pull request. A notes file may land
before its tag; the checker warns until the tag exists.
