# Remaining work

**Status:** 26 November 2026 · latest release v2.17.22 · working tree clean.

Verification log: the Windows 11 (VM) smoke pass passed all five checks on
26 November 2026 — the page-1 preview and its swap, zoom panning to the left
edge, the theme switch repainting the chart, Save as PDF (vector diagram, Inter
labels, stamped footer), and the persistent error toast dismissed by keyboard.
The engine-level check (headless Edge, the Windows export path) passed the same
day: no image objects, Inter embedded, KaTeX typeset.

Each item below names the record where the limit was accepted, the next concrete
step, and what "done" means. Work lands through the three-seat gate (Safety,
Security, Design — 8/10 each) and a release with per-tag notes and an accepted
council record.

## 1. Close the small documented gaps

About a day in total. Every item is already written down as a known limit.

| # | Item | Where | Next step | Done when |
| --- | --- | --- | --- | --- |
| 1.1 | Two messages to one live region in the same frame collapse to the last | `0067` known limits | Queue announcements with a short gap instead of two RAFs in one frame | Two errors fired in one frame are both announced (runtime probe) |
| 1.2 | A fifth error can still push out the oldest error | `0067`; cap in `ui/js/app.js` | Decide the policy (raise the error cap, or drop only seen notices) and implement it | A repeating failure cannot silently remove an unread error, or the bound is stated in the UI copy |
| 1.3 | A hung pdf.js parse leaves the page-1 preview on screen | `0068` known limits | Time out `viewer.show`, toast, and fall back to Live | A stalled parse recovers with a message instead of waiting forever |
| 1.4 | Chart tick grey is 3.77:1, below the text floor | `0069` known limits | Design call: darken `axisText` in the light set only, or keep print parity | One decision, recorded; the palette test matches it |

## 2. PDF text layer

About a day, uncertain. The Inter subset drops `=` (and sometimes `(` and `-`)
from *extracted* diagram labels: `pdftotext` returns `R 21.21 m`. The printed
page is correct (`0067` known limits).

First experiment: print the same diagram with the full Inter face instead of the
subset. If the full font also loses the punctuation, the cause is Skia's Type 3
`ToUnicode` mapping, not the subset, and the limit is confirmed rather than
fixable. Otherwise try `pyftsubset` flags (`--no-subset-tables`,
`--glyph-names`) and re-check with `pdftotext`.

Done when: `R = 21.21 m` extracts with the `=`, or the limit is recorded as
engine-bound with the experiment that showed it.

## 3. Accessibility pass with a screen reader

Half a day. No assistive technology has ever run against the app; the toast
focus handoff, live regions and the page preview are verified by code and runtime
probes only (`0067` known limits).

Run Orca on Linux over: an error toast and its dismissal, the page-1 preview
swap, the Graph tab's keyboard dragging, the report's section switches, and the
Settings theme controls. Fix what it finds, then write the pass (steps, results,
issues) into the next council record.

Done when: a written AT pass exists and its findings are either fixed or
recorded.

## 4. Housekeeping

Half a day, no user-visible change.

- `packaging/check-vendored.sh` pins the files it knows but does not flag an
  unlisted file in a vendored directory.
- The signer's `cargo audit` uses the app's allowlist; give it its own, so an
  exception that is only justified for the app cannot hide a signer advisory.
- Pin the embedded Inter subset to its `pyftsubset` output by hash, so the base64
  cannot drift from the OFL source.
- Two seat notes: an unused default on `paint(g, palette = …)` and an unused
  event `detail` on `lpc:theme`.

## 5. Durable Windows check (optional)

About a day. The Windows export path was verified by hand; nothing repeats it.

Add a `windows-latest` CI job that runs the headless Edge flags from
`render_with_browser` on a report fixture and asserts the PDF has no
`/Subtype /Image` object and does embed `InterVariable`. The open question is the
fixture: the repo has no report fixture that carries the diagram, so one must be
generated in the job or committed.

Done when: CI fails if the Windows document engine stops printing the diagram as
vectors or stops embedding the label face.

## 6. Product questions (no estimate)

Not committed work; these need a decision before a plan.

- **Saving a plan.** Nothing is stored between launches by design (the privacy
  contract in `AGENTS.md`). Saving or reopening a plan would change that contract
  and needs an explicit decision.
- **Several lifts in one report.** The report is one lift; tandem is the only
  multi-crane case.
- **Language and units.** The UI and report are English with metric units; the
  chart is the only capacity check and its radii are metric.

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
(notices and Inter), `0068` (page-1 preview) and `0069` (chart palette).
