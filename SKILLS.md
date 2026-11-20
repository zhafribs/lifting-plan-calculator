# SKILLS.md

Working modes for this repository. Each one is **self-contained**: the
instructions below are the whole skill, with no external checkout to read. Use a
mode when its "when" line matches; [AGENTS.md](AGENTS.md) wins wherever they
disagree.

## karpathy-coding

*When:* any change. Think before writing; make the smallest surgical edit.

- Read the file and its callers before editing; state the one change you will make.
- Change the minimum; do not reformat or rename around the edit.
- Loop: edit → run the narrowest check that covers it → fix → repeat.
- Stop when the check passes; do not polish further unasked.

## pstack

*When:* a fix or refactor that has to prove itself.

- Open by restating the goal, the problem it solves, and the smallest change.
- Name the data shape before writing code, then choose the structure deliberately
  (typed model, table, state machine) rather than adding branches.
- Fix the root cause; migrate every caller.
- Prove it with a check that can go red on this change; re-run the original
  scenario, not only the new test.

## caveman

*When:* the user wants speed, not prose.

- Answer in the fewest lines that still carry the fact.
- No preamble, no restating the question, no "I will now…".
- Keep file paths, numbers and command output exact.

## grill-me

*When:* the request is underspecified and a wrong guess is expensive.

- Ask one question at a time, about the case that decides the design.
- This repo's usual unknowns: units (kg vs kN), whether a radius is inside the
  chart's rated range, the 75% and 100% band boundaries, rounding in the report,
  and which of the 19 uniform arrangements applies.
- Stop asking once the answer no longer changes the diff.

## superpowers

*When:* a new screen, a new check, or anything spanning engine + report + UI.

- Write the design down first: the inputs, the formula, the verdict, the report
  paragraph. Put product decisions in `docs/adr/`, not in a scratch file.
- List the files the change reaches before writing any of them.
- Implement in the order: engine → DTO → command → screen → report.
- Delete the spec once the code and its test carry the decision.

## superstar-engineer

*When:* a trade-off call, or pressure to add a dependency.

- Prefer the design a stranger can verify: fewer moving parts, explicit data.
- Keep the engine pure Rust and the frontend dependency-free; the app has no
  npm step and no runtime service.
- Say what you rejected and why, in one line each.

## understand-anything

*When:* first contact with unfamiliar code.

- Grep the symbol before editing it; list its callers, its test and its DTO.
- Read the test to learn the expected numbers, not the prose.
- Only then decide the diff.

## graphify

*When:* an architecture question across engine, report builders and `ui/`.

- `graphify-out/` in this repo is the recorded graph; query it before grepping
  the whole tree.
- Answer with the path: caller → module → report section.
- A regeneration is a deliberate change; mention it if you touch it.

## thermo-nuclear-audit

*When:* asked for a whole-repo review, not a fix.

- Report only — change nothing.
- Rank findings: wrong arithmetic > unsafe input > untested risk > slow path >
  duplication.
- Each finding: file, what breaks, how it would be noticed.
- Say what you did not check.

## gstack

*When:* a maintenance change needs a regression hunt.

- Run the CI list from `AGENTS.md` before and after.
- Check the report output, not just the unit tests: a figure can pass its test
  and still vanish from the page.
- For PDF work, run the export probe (`--export-pdf-test`) and read the pages.

## find-skills

*When:* no mode here fits.

- Look in `.opencode/skills/` first, then the harness registry.
- If nothing fits, say so and proceed with `AGENTS.md`; never invent a skill
  name or cite a file that does not exist.

## impeccable

*When:* report layout, `ui/css`, tokens, palettes.

- Change tokens before rules and rules before one-off values.
- Check both schemes and every curated palette for contrast, not just the default.
- The report is printed on white paper; the app's dark surfaces do not apply there.

## design-taste-frontend

*When:* a new screen or a visual pass that must not read as generated.

- One system, one section per screen, real labels from the lifting domain.
- No decorative gradients, no invented metrics, no placeholder lorem.
- Every visual choice needs a reason the product already has (a band, a check, a
  unit).

## web-design-guidelines

*When:* reviewing UI code for accessibility.

- Keyboard path for every control; visible focus.
- Labels tied to inputs; errors announced, not just coloured.
- Contrast checked in both schemes and all palettes.

## writing-guidelines

*When:* README, tutorial, release notes, `AGENTS.md` itself.

- State the fact, then the reason; cut the ceremony.
- Release notes follow the shape in `docs/releases/README.md` — one file per tag,
  nothing carried in.
- Do not claim behaviour the diff does not contain.

## agent-browser

*When:* checking static `ui/` pages in a real browser.

- It cannot see the Tauri window or the report preview — there is no debug port.
- Use it for `ui/index.html` and CSS behaviour only.
- For the report and PDF, use the app's own export path: `--export-pdf-test`.

## humanizer

*When:* any prose a person will read — README, tutorial, release notes, commit
bodies, replies — and whenever text smells machine-written.

Adapted from [`blader/humanizer`](https://github.com/blader/humanizer) (MIT),
which builds on Wikipedia's *Signs of AI writing*.

- State the point. **"It's not just X, it's Y"** becomes Y.
- Cut one-line closers ("That is the real win."), staged run-ups ("Here's the
  thing.", "The hard part is…") and sayings that sound deep ("At its core…").
- Drop the objection nobody raised ("A tempting approach would be…") and the
  borrowed authority ("Experts believe…"). Name a real source or say nothing.
- Use the number of items the meaning needs. No forced triads, no list padded to
  three.
- Do not open a sentence by repeating its heading, and never repeat what the
  reader already knows.
- Remove chat residue: "Great question!", "I hope this helps", knowledge-limit
  disclaimers.
- Keep every claim the original made. Add a specific fact only when you know it
  is true; never invent one to sound concrete.
- Technical prose stays plain and neutral. Code, numbers, paths, link targets
  and the report's own wording are not touched by this pass.

This mode changes how a thing is said, never what it claims. A figure, a limit
or a verdict still belongs to the Safety seat.

## Local guardrails

These outrank every mode above:

- A change to a figure, a verdict, a limit or the words describing one summons
  the Domain Safety Engineer and needs a record in `docs/council/decisions/`.
- Never edit expected test values to make a test pass.
- No npm, no new dependency for a few lines, no network call without the
  security seat.
