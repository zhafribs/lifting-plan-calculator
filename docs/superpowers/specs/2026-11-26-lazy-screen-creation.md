# Lazy screen creation

**Date:** 2026-11-26

## Problem

`ui/js/app.js:376-387` constructs all ten screens at startup, including the
report layout/PDF viewer and the Graph canvas. Only the landing screen is shown,
so most of that DOM, observers and captures are built for nothing.

The Graph screen is special: it registers the offscreen renderer that Summary
uses for the working-range diagram (`ui/js/screens/graph.js` calls
`registerGraphCapture`). Removing eager creation without keeping that link would
silently drop the diagram from every report.

## Goal

Create a screen the first time it is shown, keep it for later visits, and let
Summary obtain the Graph renderer without the Graph tab being opened:

- startup builds only the landing screen;
- every tab is built once, on first visit;
- Summary still captures the current diagram;
- start-screen flags, `Ctrl+1..8`, the rail and every "Show report" button behave
  exactly as today;
- a screen created later receives the current state and solved result through the
  existing `sync`/`render`/`show` calls.

## Non-goals

- Changing any screen's internals, layout or copy.
- Vector diagram, report caching, or the design pass.
- Persisting screens between launches.

## Constraints

- No new dependency, npm step or CSP change.
- No visible change to any tab.
- The report's diagram must work when Summary is opened before Graph.
- A screen factory runs at most once; a failure is not cached and can be retried.
- No screen may rely on construction order for state; state is read in `go()`.

## Acceptance

- A default launch logs and constructs only the landing screen.
- Visiting each tab constructs it once; returning to it reuses the instance and
  does not rebuild DOM.
- `--screen=report` constructs Graph first (headless) and then Report; the page
  view contains the diagram.
- "Show report" from Crane, Outrigger, Uniform, Nonuniform and Tandem opens the
  report with the requested sections, including a diagram.
- `Ctrl+1..8`, the rail, the start-screen flag and About/Settings still work.
- `node --test` and `cargo test` stay green; no report content changes.
