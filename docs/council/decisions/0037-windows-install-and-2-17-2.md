# 0037 — The Windows install layout, and 2.17.2

**Status:** accepted
**Date:** 2026-11-20
**Release:** v2.17.2
**Change:** `src-tauri/tauri.conf.json`, `.github/workflows/release.yml`,
`README.md`, `CHANGELOG.md`, `docs/releases/v2.17.2.md`, the UI fixes recorded
in 0036
**Seats:** three seats, each covering charter roles: Safety & QA (Domain Safety
Engineer, QA & Test Engineer), Security & Build (Application Security Engineer,
Build & Platform Engineer), Design & Steward (Technical Writer, Product Manager,
Maintainer). Bar 8/10 per seat. The implementer did not score.

## Proposal

A Windows user installed 2.17.1 and found two files in `C:\Program Files`: the
executable and the uninstaller. Make the install look and behave like a normal
per-machine Windows app, and prove the layout before the tag.

## Why two files was the whole app

- The frontend (`ui/`, KaTeX, fonts, the PDF viewer) is compiled into the
  executable: `bundle.frontendDist` points at `../ui`, and the assets are read
  through Tauri's asset resolver.
- Tauri 2 defaults `build.windows.staticVCRuntime` to true, so the C runtime is
  linked into the executable. There is no `vcruntime140.dll` to ship.
- WebView2 is an operating-system component, not an app file.

Adding placeholder DLLs would make the folder look busier and the install
worse. The fix is real files a distributed Windows app has, not invented ones.

## What changed

1. **Bundle metadata.** `publisher` (so Add/Remove Programs names a person),
   `copyright`, `homepage`, and the SPDX `license`.
2. **Resources.** `LICENSE`, `COPYRIGHT`, `THIRD_PARTY_NOTICES.md` and
   `licenses/` are installed beside the executable. On Windows the resource
   directory is the executable's directory.
3. **WebView2 bootstrapper.** `webviewInstallMode` is `embedBootstrapper`
   (+1.8 MB): the installer can install the runtime on a machine that does not
   have it, with no separate download of the bootstrapper. `offlineInstaller`
   remains the option for a machine with no internet (+127 MB).
4. **A release check.** The Windows job now performs an MSI administrative
   install and a silent NSIS install, lists the payload, asserts the executable,
   licence, copyright, notices and `licenses/` are present, then uninstalls. A
   release cannot publish with an incomplete layout.

## Evidence

- `cargo tauri info` reads the new configuration; `cargo metadata --no-deps`
  resolves the 2.17.2 manifest and lock.
- Manual release run **44703698700** built both installers with the new config.
- Manual release run **44704662388**, job *Windows installers*, step *Check the
  installed layout*: **success**. The step printed:

```
PFiles\Lifting Plan Calculator\COPYRIGHT
PFiles\Lifting Plan Calculator\LICENSE
PFiles\Lifting Plan Calculator\licenses\Inter-OFL-1.1.txt
PFiles\Lifting Plan Calculator\licenses\KaTeX-MIT.txt
PFiles\Lifting Plan Calculator\licenses\pdfjs-Apache-2.0.txt
PFiles\Lifting Plan Calculator\lifting-plan-calculator.exe
PFiles\Lifting Plan Calculator\THIRD_PARTY_NOTICES.md
```

and, from the NSIS silent install in `C:\Program Files\Lifting Plan
Calculator`:

```
COPYRIGHT
LICENSE
licenses\Inter-OFL-1.1.txt
licenses\KaTeX-MIT.txt
licenses\pdfjs-Apache-2.0.txt
lifting-plan-calculator.exe
THIRD_PARTY_NOTICES.md
uninstall.exe
```

- The UI fixes in 0036 were checked on the built binary with
  `--platform-preview=windows` at 720×540, 800×600, 980×700, 1100×800,
  1280×720, 1440×950 and 1600×1000.
- `cargo test --all-targets` → 268 passed; `cargo clippy --all-targets --
  -D warnings` clean; `node --test` → 38 passed.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8/10 | 0036 changes no figure, limit or verdict; the margins still apply and the report prints as before |
| Security & Build | 8/10 | both installers build and lay out the payload in CI; the signature and digest path is unchanged; no new dependency |
| Design & Steward | 8/10 | the folder now shows the licence and notices; the README says why it stays small; no placeholder files |

**Mean 8.0/10** — at the bar.

## Known limits

- The installers are unsigned, so SmartScreen still shows "Unknown publisher"
  until an Authenticode certificate is bought and wired into the bundle.
- The Windows UI was checked through the layout preview on Linux, not on Windows
  hardware; the native frame, Mica and the accent lookup are Windows-only code
  paths that CI builds but does not render.
- WebView2 still comes from the machine or the bootstrapper. A machine with no
  internet and no runtime needs the `offlineInstaller` variant, which is a
  one-line change, not a code change.
- Rust was not replaced: the calculator is already native code and the UI's cost
  is WebView2, not the language.
