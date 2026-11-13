# Contributing

Thanks for taking a look. This is a small, single-maintainer desktop
application, but bug reports and focused pull requests are welcome.

## Reporting bugs

Open an issue with the [bug report form](.github/ISSUE_TEMPLATE/bug_report.yml)
and include:

- the AppImage version (or commit SHA, if built from source),
- your distribution and desktop environment,
- the exact steps to reproduce, the expected result and what happened,
- the contents of `~/.cache/lifting-plan-calculator/ui.log`, if present.

## Development

There is no Node.js build step and no npm dependency: the frontend in `ui/` is
plain ES modules embedded into the binary at compile time. The calculation
engine is Rust in `src-tauri/`.

```bash
cd src-tauri
cargo run          # development build
cargo test         # the ported test suite
cargo clippy --all-targets
```

For the frontend, edit the files under `ui/` and reload; `cargo run` picks them
up without a rebuild step.

### Building the AppImage

```bash
./build-appimage.sh            # host build, fastest
./build-appimage-portable.sh   # Ubuntu 22.04 userspace build (bwrap), widest compatibility
```

The result lands in `dist/`.

## Pull requests

- Keep changes focused; one topic per pull request.
- Match the surrounding code style. Rust is expected to pass `cargo test` and
  `cargo clippy` (both run in CI); the frontend follows the existing module
  layout.
- Update `CHANGELOG.md` under `[Unreleased]` for user-visible changes.
- Describe how you tested the change (which screen, which inputs, which
  window size).

## Review

A change that touches the engine, the report, the release pipeline or the
public documentation is reviewed by the seats named in the
[engineering council charter](docs/council/CHARTER.md). The bar is **8 out of 10
from every participating seat**; a seat scoring below 8 lists the changes it
needs and re-scores them. A trivial change — a comment, a broken link, or a typo
that changes no meaning — is reviewed by the Maintainer seat alone; the pull
request says so, and no numbered record is written.

A change to a computed figure, a verdict, a limit, or the words describing one
always summons the Domain Safety Engineer, whether or not it touches
`src-tauri/`. The accepted record goes in `docs/council/decisions/` as
`NNNN-<slug>.md`, and a release tag is refused until a record naming that
tag is accepted.

## Releases

Releases are cut from `main` by its maintainer:

1. Land the work with its `CHANGELOG.md` entry under `[Unreleased]`, and an
   accepted record in `docs/council/decisions/` naming the release.
2. Bump `version` in `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`,
   then move the changelog section from `[Unreleased]` to the new version and
   date.
3. Summarise it as `docs/releases/<tag>.md` — the shape is described in
   [`docs/releases/README.md`](docs/releases/README.md). The file is the whole
   release page; nothing is shared with an earlier release.
4. Run `./packaging/check-release-notes.sh --self-test` and
   `./packaging/check-release-notes.sh`. CI runs both on every push.
5. Commit, then tag `v<version>` and push the tag. The
   [release workflow](.github/workflows/release.yml) refuses a tag whose name,
   version, notes or council record disagree, before either platform is built.
6. Watch the run. The release is created as a **draft**, all assets are attached,
   and only then is it made public — so a partial upload never reaches a user.
   If the run fails, delete the hidden draft in the Releases page and re-run the
   failed workflow from the Actions tab; deleting the draft is what makes the
   rerun safe, because published bytes are never overwritten.

The release page carries four files and the checksum list: the Linux `AppImage`
and its `.AppImage.zsync` delta-update sidecar, the Windows `.msi` and
`-setup.exe`, plus `SHA256SUMS.txt`.

### Republishing an old page

Two of the 13 pages published before v2.5.2 used to repeat earlier notes (v2.5.0
and v2.5.1). All 13 were republished from this directory on 13 November 2026 by
one command, which is safe to repeat:

```bash
./packaging/publish-release-notes.sh        # list what would change
./packaging/publish-release-notes.sh --yes  # rewrite the bodies
```


## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`,
`fix:`, `docs:`, `refactor:`, `test:`, `build:`, `ci:`, `chore:`. A short
imperative subject is enough; add a body when the reason is not obvious.

## License

By contributing you agree that your contributions are licensed under the
GNU General Public License, version 3 or later (see [LICENSE](LICENSE)).
