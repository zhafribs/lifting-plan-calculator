# 0006 — The report preview is the PDF

**Status:** accepted
**Date:** 2026-11-14
**Release:** v2.7.0
**Change:** the Summary tab's report preview, the preview command and the vendored
PDF viewer

## Problem

The report preview was a live HTML frame. It follows every keystroke, which is
right for typing, but it is a continuous document: it shows no A4 page breaks,
it cannot answer "where does page 2 start", and it cannot be zoomed into a
figure. The operator signs the PDF, not the frame.

## Proposal

Add a **Pages** view that draws the report as the PDF that will be saved, with
zoom, fit-to-width and a `Page N of M` counter, and keep the live HTML view as
**Live** for typing.

- The preview is rendered by `preview_pdf`, which uses the same print and footer
  path as *Save as PDF* and returns the bytes over IPC (`tauri::ipc::Response`,
  no base64).
- Those bytes are kept in memory, keyed by an FNV-1a hash of the report HTML and
  its footer. *Save as PDF* writes those exact bytes when the plan has not
  changed since, so the file cannot differ from the pages on screen.
- The page view renders on demand: when the tab opens, when a section is
  toggled, when a field is left, or on the refresh button. Never per keystroke.
- Requests are single-flight and latest-wins: a generation counter is checked
  before and after the print, and a superseded request neither prints nor
  stores. The UI drops superseded results and re-schedules once.
- Zoom re-renders each page at `zoom × min(devicePixelRatio, 2)`; pages outside
  the viewport keep a sized placeholder and release their canvas.
- The viewer is pdf.js 3.11.174 (Apache-2.0), vendored unmodified, pinned by
  SHA-256 in `ui/vendor/pdfjs/CHECKSUMS.txt` and verified by
  `packaging/check-vendored.sh` in CI and in the release's notes job.
  `isEvalSupported: false` is set for CVE-2024-4367. On a renderer that cannot
  start a Worker, the viewer loads the worker script into the page and pdf.js
  takes its main-thread path.

## What was reviewed

- `cargo test --all-targets` → 201 passed, including the report key, the atomic
  write from memory, and the generation gate that makes a superseded preview
  neither print nor store (`a_newer_preview_supersedes_an_older_one`).
- `cargo clippy --all-targets -- -D warnings` → clean.
- `node --test` over `ui/js/pdf-math.test.mjs` → 7 passed: fit width, fit page,
  the backing-scale cap, zoom steps, the eviction window, page labels.
- The viewer module itself was driven in WebKitGTK over HTTP with a real
  exported plan: page 1 at 148% with `Page 1 of 7`, zoom to 200% re-rendered
  crisply, and the same page rendered again with `window.Worker` replaced by a
  throwing stub, which exercises the main-thread fallback.
- `ui/vendor/pdfjs` matches its checksums.
- The built 2.7.0 AppImage is 79.5 MB, and its binary carries the viewer; the
  release job now fails if that marker or GTK's file print backend is missing.

## Scores

| Seat | Score | Position |
| --- | --- | --- |
| Build & Platform Engineer | 8/10 | Raw bytes, no new cache dir, real cancellation and the vendored hashes close the first pass's 6 |
| Application Security Engineer | 8/10 | Single-flight, the CVE mitigation, a genuine fallback and pinned vendor files close the first pass's 6 |
| QA & Test Engineer | 8/10 | One render shared with the export, eviction, re-rendered zoom and the arithmetic tests close the first pass's 6 |
| Product Manager | 8/10 | On-demand page view with Live kept, literal what-you-see-is-what-saves, and notes that name the platform difference |

Mean 8.0, every seat at the bar, no veto. The design pass scored 6/6/6/6.

## Required changes

All landed in 2.7.0: shared bytes between preview and export; single-flight with
a generation counter; on-demand rendering; zoom that re-renders rather than
stretches; canvas eviction; checksum-pinned vendored viewer with licence and
notices; `THIRD_PARTY_NOTICES.md` row; release notes and changelog.

## Decision

**Accepted.** Summary opens on the real pages, with the live view one click away
and the saved file identical to what was on screen.

## Consequences

- The page view needs the print path, which is WebKitGTK on Linux and Edge on
  Windows. Where it fails, the pane says why and returns to Live; it does not
  show an empty page.
- A preview costs one print of the report — about half a second for a seven-page
  plan on Linux, longer on Windows where Edge is spawned. That is why it is on
  demand rather than live.
- The app grows by about 1.5 MB for the viewer. It is vendored, so the air-gap
  promise and the offline export are unchanged.
- The preview and the export are the same engine, so the platform note from
  2.6.0 still holds: Linux and Windows PDFs can break pages differently.
- Zoom above the renderer's cap is sharp to 200% of the page's own size on a 2x
  screen; beyond that the canvas grows with the zoom, bounded by memory.
