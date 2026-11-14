# 0005 — Print the plan with the app's own web view

**Status:** accepted
**Date:** 2026-11-14
**Release:** v2.6.0
**Supersedes:** [0004](0004-bundled-pdf-renderer.md)
**Change:** the PDF export path, the footer, the AppImage payload and the release
pipeline around them

## Problem

2.5.4 bundled Chrome for Testing's headless shell so *Save as PDF* worked on a
machine with no browser installed. That took the AppImage from about 82 MB to
182 MB — over half the download for a feature used once per plan — and the owner
wants one AppImage variant, not a full/slim pair.

Two proposals were rejected before this one:

- **Bundle less**: the payload is 278 MB unpacked (202 MB binary, 50 MB locales,
  11 MB ICU). Trimming locales and SwiftShader saves about 20 MB. Not a fix.
- **Download the renderer on first use**: rejected by Security (5/10) and QA
  (3/10). A downloaded binary would be re-executed forever from `~/.cache` with
  no re-verification, a zip parser would become runtime attack surface, and —
  the decisive point — a downloaded headless shell still needs host libraries
  (`libnss3`, `libgbm`, `libasound`) that browserless machines usually lack, so
  it failed exactly where it was meant to help.

## Proposal

Print the report with the WebKitGTK view the app already runs on, straight to a
PDF file, and draw the footer onto the finished pages. No bundled browser, no
download, no network, no consent step. Windows is unchanged: it prints with the
Edge that ships with the system.

WebKitGTK does not render the report's CSS `@page` margin boxes, so the footer
(plan reference, credit line, `n / total`) is written onto the pages afterwards
with `lopdf`, in a new module `src-tauri/src/pdf.rs`. That also removes a defect
inherited from 2.5.4: Chromium builds older than 131 drop margin boxes silently,
which took the reference and page counter off a signed plan.

The footer's typeface is embedded (Liberation Serif, the face the Chromium build
used) as a Type0/Identity-H font with a ToUnicode map, so an operator's reference
or credit line in Latin with diacritics, Greek, Cyrillic or Vietnamese prints as
typed. Scripts outside that face fall back to a placeholder box, as they do in
the report's own font stack.

## What was reviewed

- `cargo test --all-targets` → 198 passed, including stamping a committed
  WebKitGTK-printed fixture (compressed streams, indirect resources, xref
  stream), footer placement, `/Rotate` refusal, `/Contents` as an array
  reference, CSS escaping, staging and the font's coverage.
- `cargo clippy --all-targets -- -D warnings` → clean.
- The real export: 7 A4 pages, footer on every page at x = 72 pt with the
  counter's right edge at 523 pt and the baseline at 33 pt (the Chromium build
  put it at 32.9 pt), KaTeX typeset, page 1 visually identical to the Chromium
  output. A reference of `Nguyễn Văn Ánh — LP/2026 (A)` extracts and renders
  correctly.
- The built AppImage: **79.1 MB**, carries GTK's four print backends including
  `libprintbackend-file.so`, and no browser payload.
- CI gained a `pdf-export` job (xvfb + dbus, the real export, page count, A4
  size, footer text and position on every page, and ink in page 1's footer
  strip) and an `audit` job. The release workflow now inspects the built
  AppImage and fails if the print backend is missing, if a browser payload is
  present, or if the file exceeds 95 MB.

## Scores

| Seat | Score | Position |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | Gathers to 8 once the work is committed: CI proof, payload assert, embedded font and the leak fix close the first pass's 6 |
| Application Security Engineer | 8.5/10 | All seven required changes from the first pass landed; no boundary crossing |
| QA & Test Engineer | 8/10 | Staging, a real-Skia fixture and a display-backed CI job close the first pass's 6; the tree was taggable once committed |
| Product Manager | 9/10 | One variant, 100 MB smaller, no first-use failure; the notes now match what each platform actually does |

Mean 8.4, every seat at or above the 8/10 bar, no veto.

The first pass of this proposal scored Build 6, Security 8, QA 3 and Product 8
against a *different* design — slim AppImage plus a first-use renderer download —
which was withdrawn. The scores above are for what shipped.

## Required changes

All landed in 2.6.0:

- Print and stamp into a hidden sibling file and rename on success, so a failed
  footer cannot leave a plan without its reference and page numbers.
- Escape `<` in the report's CSS strings: the reference and credit line are
  written into a `<style>` block, and the preview is an unsandboxed `srcdoc`
  frame, so a closing tag there would have been script execution.
- Name the network access honestly in `SECURITY.md`, and state the in-process,
  downloads-nothing invariant as policy.
- Own print jobs in a registry instead of their own signal handlers, so an
  export does not leak a web view.
- One margin constant shared by the report's CSS, the GTK page setup and the
  footer's position; refuse a rotated page rather than mis-stamp it.
- Private 0700 workspace for the report's HTML and KaTeX assets.
- `cargo audit` in CI.

## Decision

**Accepted.** One AppImage, about 79 MB, with the PDF export working on a
machine with nothing installed and no network.

## Consequences

- Linux PDFs are produced by WebKitGTK rather than Chromium, so pagination can
  differ from 2.5.3 and earlier on a long plan, and the two platforms' PDFs are
  no longer produced by the same engine. The footer is drawn by the same code on
  both, so it cannot drift.
- The export's fidelity is now bounded by WebKitGTK's CSS support. The report's
  styles are conservative (tables, flex, no grid), and the display-backed CI job
  prints the worked example on every push, but a future CSS feature should be
  checked against WebKit before it is relied on in print.
- A desktop with no GTK print backend fails after 20 seconds with a message
  naming *Save as HTML*. The AppImage always carries the backend, and the
  release job asserts it.
- A font that does not cover a script still prints a placeholder box for it;
  this matches the report's own font stack.
- The export was proved from the build tree and through CI, not by clicking
  *Save as PDF* inside the packaged AppImage. The release job asserts that the
  packaged file carries GTK's file print backend and no browser payload, which
  covers the packaging failure mode; a click-through of the published AppImage
  remains a manual release step.
