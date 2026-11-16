# 0014 — The window runs under a policy, and a jib is checked at the chart's basis

**Status:** accepted
**Date:** 2026-11-17
**Release:** v2.12.0
**Change:** Content-Security-Policy for the window, the report's auto-render pass
as a file, and the crane and jib chart-basis fixes

## Problem

Four things were wrong or missing:

- the window had no content policy (`"csp": null`), so any script that reached the
  document ran, inline handlers included;
- switching the jib configuration back to "Boom length" kept the jib's working
  radius and zeroed the row's angle instead of returning to the main boom's
  radius basis;
- "Boom length + extended jib" was offered even when the workbook carried a
  single jib length;
- the Graph tab's boom angle only reached the Crane tab with a workbook and a
  live radius, and a jib angle between charted rows was read at the nearest row
  (or interpolated), which can claim capacity the lift does not have.

## Proposal

- **Set a policy.** `default-src 'self'`, `script-src 'self'`, images and fonts
  from self or `data:`, workers from self, the IPC endpoints as the only
  connection target, and `object-src 'none'`, `base-uri 'self'`,
  `form-action 'none'`. Inline scripts and inline event handlers stop running.
- **Move the report's typesetting out of the document.** KaTeX starts from
  `vendor/katex/lpc-autorender.js`; the preview no longer reads the frame's
  document, it receives a message. A sandboxed frame was tried and rejected —
  an opaque origin makes `script-src 'self'` match nothing, so the equations do
  not typeset.
- **Restore the main boom through the import's own rule.** `jib_changed` with
  `"boom"` goes through `applied`: the farthest charted radius within 75%, the
  row mirrored, the jib table and angle cleared.
- **Offer only tables the workbook has.** The shared `jib.js` decides what the
  jib lengths allow; both tabs disable and grey the extended jib when there is a
  single length, and `store.chooseJib` refuses it if a stale value arrives.
- **Publish the graph's angle, and read a jib at the lower row.** The angle is
  published with or without a workbook. With a workbook an angle edit also
  publishes the reach it produces, so the radius and the chart check follow.
  The jib lookup floors the angle to the chart's own rows
  (`charted_angle_below`), and the Crane tab's select shows that row.

## What was reviewed

- `cargo test --all-targets` → 226 passed. New tests: the jib round-trip
  restores the radius (with and without a lift weight to suggest from); a jib
  angle between rows uses the lower row; an angle below the first row is refused
  as outside the chart; the lower-row helper never rounds up.
- `node --test ui/js/*.test.mjs` → 13 passed, covering the shared jib rules
  (availability, length choice and the lower-row angle); the suite now runs in
  CI beside the page-view arithmetic.
- `cargo clippy --all-targets -- -D warnings` → clean. All 19 frontend modules
  syntax-check, and the vendored KaTeX and fonts still pass their check.
- The policy was exercised under its exact string in headless Chromium: the
  report typeset 76 equations, while an inline `<script>` and an `onerror=`
  handler were both blocked. The sandboxed frame produced 0 equations, which is
  why it is not shipped.
- The tree was built into an AppImage; the window, report preview and PDF export
  were confirmed in the app, and the export probe still prints the worked
  example.

## Decision

**Accepted.** The window runs under the policy, the report still typesets, the
main-boom basis returns when the jib is removed, an absent extended jib is not
offered, and a jib angle is checked at the chart's lower row.

## Consequences

- No inline script or inline handler may be reintroduced in the UI or the report;
  the KaTeX pass stays a file and reports by message.
- The report preview is same-origin under the policy; adding a `sandbox`
  attribute would stop the equations from typesetting.
- A jib plan between chart rows is conservative: the boom's real angle is still
  reported and drawn, and only the chart lookup uses the lower row.
- A workbook with one jib length now shows two configurations, not three.
- Still open, unchanged: Windows Authenticode (SignPath application submitted),
  the zsync sidecar's zeroed signature sections, Dependabot grouping, and
  reviewers on the GitHub `release` environment.
