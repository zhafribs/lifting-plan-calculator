# ADR-0010 — A plan is saved as an explicit, versioned JSON file

- **Status:** accepted, 2026-11-28
- **Owners:** Maintainer, Security & Build, Safety & QA, Design & Steward
- **Applies to:** the app state, the Summary screen, the filesystem

## Context

The app stores no plan between launches (the appearance setting is kept in
`appearance.json`). That is a deliberate privacy property:
the only automatic write is `~/.cache/…/ui.log` (the update check only reads).
The cost
is that closing the app loses the plan, and a plan cannot be handed to a
colleague or reopened later.

The state already has a wire shape (`AppState`) that the UI sends to the engine
on every edit, with serde defaults and catalog keys. The report settings live on
the Summary screen: sections, page breaks, sign-off, footer, margins, diagram
scale and the issue date.

## Decision

- **Explicit save and open only.** Two buttons on the Summary screen, a system
  dialog for the path, and no auto-save, no recent-files list, no cloud.
- **One JSON file**, `format: "lifting-plan-calculator/plan"`, `version: 1`,
  carrying `state` (the wire shape) and `report` (the settings). Results are
  recomputed on open and never stored.
- **The file is untrusted input.** Rust reads, parses and validates it — size,
  format, version, required keys, numeric bounds, catalog keys, lengths, array
  caps and unique line ids — before any of it is applied. A refusal returns an
  error and leaves the current plan untouched.
- **The save path validates the same shape**, so the app cannot write a file it
  would refuse to open.
- **The Graph tab's drawing state is saved with the report.** The configuration,
  boom and jib lengths, angle, envelope sweep and crane position go into the
  file, because the diagram's sweep labels are printed figures. Without them, a
  reopened plan would draw the default diagram.
- **The issue date belongs to the plan.** A saved plan keeps its date; a new plan
  gets today's.
- **The privacy contract changes in one sentence:** no plan is stored unless the
  operator saves one; the appearance setting and `ui.log` are
  the only other writes; the update check only reads. `AGENTS.md` and the README
  say it in those words.
- **Three UI-only outrigger flags ride in `state`.** `crane_mass_linked`,
  `lift_load_linked` and `crane_capacity_linked` decide whether the form keeps
  the outrigger in step with the crane and the load. The solver ignores them;
  without them, reopening a plan would change how the form behaves.

## Consequences

- A plan can be shared, archived and reopened; the printed result is the saved
  result.
- The app has one more user-chosen write. It writes only where the dialog points,
  once, and never reads any file the operator did not choose.
- A file from a future version is refused with a message rather than half-read;
  version 1 is the only format today.
- There is still no dirty-state tracking: closing without saving loses the plan,
  as before.
- The format is the wire shape, so a future change to `AppState` is a format
  change and needs a version bump and a migration.

## Alternatives rejected

- **Auto-save the last plan.** Reopens where you left off, but breaks the
  “nothing stored” promise and needs retention and deletion controls.
- **A recent-plans list.** Storing paths is storage, and a recent list leaks
  file names the operator may not expect.
- **A database or binary format.** More surface to validate for no gain; JSON is
  inspectable, diffable and needs no new dependency.
- **Saving solved results.** Two sources of truth; a stale result must never be
  printed after an edit.
- **Saving on close.** It writes without consent and turns a crash into data
  loss questions.
