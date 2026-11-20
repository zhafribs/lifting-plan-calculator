# SKILLS.md

Working modes worth reaching for, adapted from
[`zhafribs/Proton-Drive-Linux-Cli`](https://github.com/zhafribs/Proton-Drive-Linux-Cli).
They are **not vendored in this repository** — the files live at
`.opencode/skills/<name>/SKILL.md` over there, and many harnesses expose the same
names as built-in skills. Read the upstream file or invoke the skill by name.

`AGENTS.md` always wins. None of these overrides this repo's rules: the chart is
the only capacity check, the ported test literals are not to be edited, and a
tag needs its notes and council record.

| Skill | Use it when |
| --- | --- |
| `karpathy-coding` | Any change: think first, minimal surgical edit, loop until verified. Pairs with `AGENTS.md`. |
| `pstack` | Deep-first work that needs proof — a fix that must show it can go red, a multi-step refactor. |
| `caveman` | Short replies wanted: dense, no ceremony, no restating the question. |
| `grill-me` | The request is underspecified — clarify edge cases before writing code. |
| `superpowers` | A feature large enough to want a spec before the diff. |
| `superstar-engineer` | Trade-off calls: reliability, simplest defensible architecture. |
| `understand-anything` | First pass in unfamiliar code — map files, calls and dependencies before editing. |
| `graphify` | Architecture questions across the engine, report builders and `ui/`. |
| `thermo-nuclear-audit` | Whole-repo review: debt, structural traps, risky untested paths. |
| `gstack` | Regression hunt on a maintenance change; QA passes. |
| `find-skills` | Looking for a skill that is not listed here. |
| `impeccable` | Report layout, `ui/css`, tokens — polish, critique, audit. |
| `design-taste-frontend` | New screen or a visual pass that must not read as AI slop. |
| `web-design-guidelines` | Reviewing UI code against Web Interface Guidelines. |
| `writing-guidelines` | README, tutorial, release notes, `AGENTS.md` itself. |
| `agent-browser` | Browser-driven UI checks. Needs a DevTools browser; the app's webview has no debug port, so use it for static `ui/` pages only, not the report preview. |

## How these meet this repo

- The verification these skills ask for is the CI list in `AGENTS.md`:
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, the
  `node --test` files and the `node --check` loop.
- `impeccable` and `design-taste-frontend` stop at visual changes; a change to a
  computed figure still needs the Domain Safety Engineer and a council record.
- `agent-browser` cannot see the Tauri window. Test the report and PDF through
  the app's own export path (`--export-pdf-test`), not a browser.
