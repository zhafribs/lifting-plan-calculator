# Remaining work

**Status:** 27 November 2026 · v2.17.23.

Verification log: the Windows 11 (VM) smoke pass passed all five checks on
26 November 2026. The headless-Edge engine check now runs on every CI push
(`windows-check`, "Print the diagram probe with Edge").

## Landed in 2.17.23

| Roadmap item | What landed | Where |
| --- | --- | --- |
| 1.1 Two notices in one frame | Live-region writes are serialised with a gap, so both are written; a burst is de-duplicated and bounded | `ui/js/live-announcer.js`, test |
| 1.2 A fifth error evicts an unread one | A repeated message is counted on its existing notice (`×2`) instead of adding one, so a repeating failure evicts nothing; distinct failures are still capped at four | `ui/js/app.js`, toast count |
| 1.3 A hung PDF parse | The page render has a 15 s deadline and falls back to Live with a message | `ui/js/async.js`, `report.js` |
| 1.4 Chart tick contrast | Closed without a code change: keep the printed page's grey so screen and paper match. The decision was recorded in 2.17.22 | ADR-0009, `0069` |
| 2. PDF text layer | The subset is not the cause: the full Inter face drops `=`, `(`, `)` on both engines. Recorded as a Type 3 text-mapping limit (both emit Type 3 for SVG webfont text); experiment below | this file, `0067` limit |
| 4. Housekeeping | `check-vendored.sh` now fails on an unlisted file; the signer audits against its own empty allowlist; the embedded Inter subset is hash-pinned; the unused `paint` default and event detail are gone | `packaging/`, `tools/sign-manifest/.cargo/audit.toml`, `diagram-font.test.mjs`, `graph.js`, `app.js` |
| 5. Durable Windows check | CI prints a diagram probe through Edge and fails if it is rasterised or the label face is dropped | `packaging/windows-diagram-probe.mjs`, `.github/workflows/ci.yml` |
| 6. Product questions | Written as decision briefs with options and recommendations; nothing implemented | `docs/product-decisions.md` |

### The full-face experiment

The same label (`R = 21.21 m (45°) - test`) drawn in an SVG with the embedded
subset, then with the full `ui/fonts/InterVariable.woff2`, printed by Chromium
158 and by the app's WebKit path, then `pdftotext`:

| Face | Engine | Extracted |
| --- | --- | --- |
| subset | Chromium | `R  21.21 m 45° - test` |
| full | Chromium | `R  21.21 m 45° - test` |
| full | WebKit | `R  21.21 m 45° - test` |

All three lose `=`, `(`, `)`. The full face shows the subset is not the cause;
both engines write the SVG text as Type 3 fonts, and that text layer is where the
punctuation goes missing.

## Still open

1. **A real screen-reader pass.** Orca/AT-SPI are not installed here. The code
   audit and its fixes are in `docs/accessibility.md`; the Windows 11 script is
   `docs/narrator-pass.md` (Narrator is built in, so the VM can run it). This is
   the one engineering item not finished.
2. **Product decisions.** Saving a plan, several lifts per report, language and
   units — each needs an answer before it becomes work; see
   `docs/product-decisions.md`.

## Limits still accepted

- The chart's tick grey is 3.77:1 on the light chart: it is the printed page's
  own value, kept so the screen matches the paper (`0069`).
- PDF text extraction drops a few punctuation glyphs in diagram labels; the
  printed page is correct, and the full-font experiment shows the engine, not
  the subset, is the cause (`0067`).
- Low-alpha overlaps in the printed diagram are the approximation recorded in
  ADR-0007.
- The page-1 preview briefly shows report content that the print may push whole
  onto page 2 (`0068`).

## Picking this up

```bash
cd src-tauri
cargo test --all-targets
cargo clippy --all-targets -- -D warnings

cd ..
./packaging/check-vendored.sh
./packaging/check-release-notes.sh --self-test
# frontend: the list in AGENTS.md, then the ES-module syntax loop
```

A release still needs `docs/releases/<tag>.md` naming the version and an accepted
`docs/council/decisions/NNNN-*.md` naming the tag. The last records are `0067`
(notices and Inter), `0068` (page-1 preview), `0069` (chart palette) and `0070`
(written with this release).
