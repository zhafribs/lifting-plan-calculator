# 0070 — Roadmap batch: notices, page timeout, checks and names

**Status:** accepted
**Date:** 2026-11-27
**Release:** v2.17.23
**Change:** `ui/js/live-announcer.js`, `ui/js/async.js`, `ui/js/app.js`,
`ui/js/screens/report.js`, `ui/js/screens/overall.js`, `ui/js/screens/graph.js`,
`ui/js/pdf-viewer.js`, `ui/css/app.css`, `packaging/check-vendored.sh`,
`packaging/windows-diagram-probe.mjs`, `.github/workflows/ci.yml`,
`tools/sign-manifest/.cargo/audit.toml`, tests, `docs/roadmap.md`,
`docs/accessibility.md`, `docs/product-decisions.md`
**Seats:** Safety & QA, Security & Build, Design & Steward. Bar 8/10 per seat.
The implementer did not score.

## Proposal

- Serialise live-region writes so two notices in one frame are both written;
  de-duplicate a repeat and bound a burst.
- Count a repeated notice on its existing toast (`×2`) instead of adding one;
  distinct notices stay capped at four, a transient giving way before an error.
- Give the page render a 15 s deadline and abandon it on timeout.
- Name the glyph-only controls, the chart and the page region; keep visible text
  as the name where there is any.
- Make the vendored check fail on an unlisted file; give the signer its own
  audit allowlist; pin the embedded Inter subset by hash; remove two dead
  defaults.
- Add a Windows CI probe that prints an SVG diagram with the embedded subset
  through Edge and fails if it is rasterised or the face is dropped.
- Record the screen-reader checklist and the product questions.

## What the council saw

- **Safety & QA (8.5):** the printed diagram is untouched (both print paths pass
  `PRINT_PALETTE`, `printify` once, no `src-tauri` source change). The repeat
  count cannot merge a different failure; the timeout abandons the in-flight
  show; the vendored exemption is scoped and proved with planted files.
- **Security & Build (8.5):** no new app dependency, network call, write,
  permission or CSP change; the Windows steps take no user input; the PowerShell
  assertions are two literal patterns; the signer's empty allowlist is read from
  its own directory; the new tests are hermetic.
- **Design & Steward (8.0):** the glyph buttons are named and the word buttons
  keep their visible names; the page region has `role="region"`; the copy says
  "written", not "announced"; the roadmap, a11y audit and product brief state
  what was and was not done.

## Evidence

- **Notices:** a runtime probe fired two identical errors and one different one:
  `Could not save the plan. x2 | Could not read the chart. x1`.
- **Announcer:** 6 tests — order, gap, repeat not queued, 20-message burst
  bounded to 9 spoken, missing setter rejected.
- **Timeout:** `withTimeout` tests cover resolve, pass-through rejection and
  deadline; `viewer.abandon()` stops a late show.
- **Vendored check:** an unlisted `ui/fonts/extra.woff2` fails; a planted
  `ui/vendor/pdfjs/lpc-autorender.js` fails; the real tree passes; the PDF.js
  `LICENSE` hash is pinned.
- **Signer audit:** `(cd tools/sign-manifest && cargo audit --deny warnings)`
  passes with `ignore = []`; invalid TOML in that file fails the run, so the
  config is read.
- **Subset pin:** 72,292 B, `sha256 4a5713d1cbf18b7acdfbe07279a589766976e446fac1ee6faa58fac055440f9f`.
- **Windows probe:** reproduced locally with Chromium 158 — 0 × `/Subtype /Image`,
  `InterVariable` present; a rasterised control PDF does contain
  `/Subtype /Image` in clear text, so the check is not a false pass.
- **PDF text layer:** both Chromium 158 and WebKitGTK 2.54.3 emit Type 3 fonts
  named `InterVariable` for the SVG text; subset and full-face prints all
  extracted `R  21.21 m 45° - test`, losing `=`, `(` and `)`. The subset is not
  the cause; the table is in `docs/roadmap.md`.
- **Gates:** 89 frontend tests, 283 Rust tests, clippy `-D warnings`, the ES
  syntax loop, `check-vendored.sh` and both audits pass.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Safety & QA | 8.5/10 | print untouched; repeat/queue/timeout verified; vendored exemption scoped |
| Security & Build | 8.5/10 | no new surface; CI input-free; signer allowlist isolated |
| Design & Steward | 8.0/10 | Label in Name respected; copy and records honest |

**Mean 8.3/10** — at the bar.

## Decision

**Accepted.** Notices are counted and queued honestly, the page render cannot
hang, the checks catch unlisted and unpinned files, and the Windows engine gets
a vector/font gate on every push.

## Known limits

- No screen reader has run. `docs/accessibility.md` carries the audit and the
  five-step Orca checklist; the AT pass is the one open engineering item.
- A repeat inside the 350 ms announcement gap is counted on screen but not
  spoken again.
- Five *distinct* errors still cap at four; only an identical repeat is free.
- The Windows probe hand-builds its SVG, so it checks the engine, not a
  regression in `svg-surface.js`; the PDF export test covers the latter.
- The Windows gate is a plaintext scan of the PDF; if Edge ever moves object
  dictionaries into compressed object streams, the check must be revisited.
- The product questions (saving, several lifts, language and units) are written
  up in `docs/product-decisions.md`; none is decided.
