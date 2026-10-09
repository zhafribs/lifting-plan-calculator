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
  be `rustfmt`-formatted; the frontend follows the existing module layout.
- Update `CHANGELOG.md` under `[Unreleased]` for user-visible changes.
- Describe how you tested the change (which screen, which inputs, which
  window size).

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`,
`fix:`, `docs:`, `refactor:`, `test:`, `build:`, `ci:`, `chore:`. A short
imperative subject is enough; add a body when the reason is not obvious.

## License

By contributing you agree that your contributions are licensed under the
GNU General Public License, version 3 or later (see [LICENSE](LICENSE)).
