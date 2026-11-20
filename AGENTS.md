# AGENTS.md

Guidance for coding agents. Human-facing docs: [README.md](README.md),
[CONTRIBUTING.md](CONTRIBUTING.md), and the review process in
[docs/council/CHARTER.md](docs/council/CHARTER.md).

## What this is

A Tauri 2 desktop app for planning a crane lift. The calculation engine is Rust
in `src-tauri/src/`; the frontend is plain ES modules in `ui/`, embedded into the
binary at compile time. There is **no Node build step and no npm dependency** —
never add one. Editing `ui/` and running `cargo run` is the whole loop.

## Commands

```bash
cd src-tauri
cargo run          # dev build, picks up ui/ changes without a rebuild step
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Frontend modules that work without a webview:

```bash
node --test ui/js/pdf-math.test.mjs ui/js/jib.test.mjs \
  ui/js/graph/print-palette.test.mjs ui/js/theme.test.mjs \
  ui/js/windows-theme.test.mjs
# CI syntax-checks every module as an ES module (so the copy is renamed .mjs)
mkdir -p /tmp/jschk && printf '{"type":"module"}\n' > /tmp/jschk/package.json
while IFS= read -r -d '' f; do
  cp "$f" /tmp/jschk/check.mjs && node --check /tmp/jschk/check.mjs || exit 1
done < <(find ui/js -name '*.js' -print0)
```

Repo-level gates:

```bash
./packaging/check-vendored.sh                 # vendored PDF parser is hash-pinned
./packaging/check-release-notes.sh --self-test
./packaging/check-release-notes.sh            # every tag has notes
cargo audit --file src-tauri/Cargo.lock --deny warnings   # allowlist: .cargo/audit.toml
```

Builds: `./build-appimage.sh` (host) or `./build-appimage-portable.sh`
(Ubuntu 22.04 userspace, glibc 2.35 floor). Output goes to `dist/`.

## Layout

- `src-tauri/src/calc.rs` — the 75% rule, chart lookup, capacity bands.
- `calc.rs`-adjacent engines: `model.rs`, `crane.rs`, `outrigger.rs`,
  `sling.rs`, `nonuniform.rs`, `excel.rs`, `report*.rs`.
- `commands.rs` is the Tauri command surface; `dto.rs` holds the wire shapes.
- `ui/js/screens/` — one module per tab; shared helpers in `ui/js/`.
- `ui/vendor/katex/` — bundled KaTeX used by the report.
- `docs/adr/` — decisions, one file each. `docs/council/decisions/NNNN-<slug>.md`
  — accepted review records. `docs/releases/<tag>.md` — the release page body.

## Rules the build enforces

- **Do not edit expected test values to make a test pass.** The suite carries the
  original Kotlin tests' own literals; a drift in the arithmetic must fail.
- The version lives in **two** files — `src-tauri/Cargo.toml` and
  `src-tauri/tauri.conf.json`. Bump both together.
- A tag is refused without `docs/releases/<tag>.md` naming that version and an
  accepted council record naming the tag. The notes are the release page; never
  reuse an earlier release's notes.
- Every `ui/js/*.js` module must pass `node --check`, and clippy must stay
  warning-free.
- `packaging/check-vendored.sh` pins vendored files by hash. Changing one is a
  deliberate change, not an update.

## Domain rules

- The chart is the only capacity check; the crane's rated capacity is printed
  for the record, not tested. A radius outside the chart's rated range is
  refused, never approximated.
- Bands: ≤ 75% **WITHIN LIMIT**, 75–100% **CAUTION**, > 100% **WARNING**.
- Every figure in the printable report is written as the formula that produced
  it. Keep that property when touching the report builders.
- A change to any figure, verdict, limit, or the words describing one summons
  the Domain Safety Engineer; engine/report changes need a council record under
  `docs/council/decisions/` before landing.
- Privacy contract: nothing is stored between launches, one GitHub update check
  at launch, and only `~/.cache/lifting-plan-calculator/ui.log` is written.
  Adding a network call, a file write, or a permission needs the security seat.

## Style

- Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:` …), imperative subject.
- Match the surrounding code; comments explain *why*, not *what*.
- User-visible changes get a `CHANGELOG.md` entry under `[Unreleased]`, in the
  same words the release notes will use.
- Keep changes focused: one topic per PR, no drive-by refactors.
