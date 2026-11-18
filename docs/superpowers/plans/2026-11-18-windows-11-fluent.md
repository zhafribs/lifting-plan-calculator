# Windows 11 Fluent Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the Windows build a Windows 11 Fluent shell, Fluent NavigationView, Mica window and system light/dark/accent, while leaving Linux, the calculation engine and the printed report unchanged.

**Architecture:** Keep the existing plain ES-module frontend. Add a Windows-only platform marker and a small pure theme-helper module, load a separate `ui/css/windows.css` layer after the unchanged base CSS, and add a NavigationView collapse state in the shell. Mica is configured through a Windows-only Tauri config file; the system accent is read from Windows and exposed through `app_info`.

**Tech Stack:** Tauri 2, Rust 1.85+, plain ES modules, CSS custom properties, Node's built-in test runner.

**Spec:** `docs/superpowers/specs/2026-11-18-windows-11-fluent-design.md`

## Global Constraints

- Windows-only visual scope. Linux and macOS keep the current theme.
- The printed report and PDF builders do not change.
- No npm build step, no Fluent UI component runtime, no new npm dependency.
- CSP stays as it is: no inline scripts or inline event handlers.
- Follow Windows system light/dark and system accent; fall back to the Windows 11 accent if the system value cannot be read.
- Native Windows title bar; Mica through Tauri window effects; solid Fluent fallback on Windows 10.
- Keep `Ctrl+1` through `Ctrl+8` exactly as they are.
- The app must remain usable at the 720×540 minimum window size.
- Rail state is session-only; no new persistence.

## Review Focus

- Windows 10 without Mica: the same Fluent layout must render on a solid surface.
- Light theme: body, muted, placeholder, status and accent-on-surface contrast.
- Low-contrast system accents (yellow/lime): primary-button and focus text must stay legible.
- Compact NavigationView at 720px: labels disappear, tools remain reachable, no horizontal scroll.
- Live system theme change and `forced-colors` mode: the shell must not strand a light surface on a dark window, and focus must stay visible.

---

### Task 1: Theme helpers, platform preview and accent plumbing

**Files:**
- Create: `ui/js/windows-theme.js`
- Create: `ui/js/windows-theme.test.mjs`
- Modify: `ui/js/app.js`
- Modify: `src-tauri/src/commands.rs`
- Create: `src-tauri/src/theme_windows.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`

**Interfaces:**
- `platformScope(info: { platform: string, platform_preview?: string }) -> "windows" | "default"`
- `initialNavMode(width: number) -> "expanded" | "compact"`
- `nextNavMode(mode: "expanded" | "compact") -> "expanded" | "compact"`
- `accentVars(hex: string, dark: boolean) -> { accent: string, accentSoft: string, accentHover: string, accentContrast: string }`
- `navToggleLabel(mode) -> "Collapse navigation" | "Expand navigation"`
- `AppInfo` gains `platform_preview: Option<String>` and `accent: Option<String>`.
- `theme_windows::system_accent() -> Option<String>` (Windows only).

- [ ] **Step 1: Write the failing helper tests**

Create `ui/js/windows-theme.test.mjs`:

```js
import test from "node:test";
import assert from "node:assert/strict";
import {
  platformScope,
  initialNavMode,
  nextNavMode,
  accentVars,
  navToggleLabel,
} from "./windows-theme.js";

test("windows platform is scoped; linux is default", () => {
  assert.equal(platformScope({ platform: "windows" }), "windows");
  assert.equal(platformScope({ platform: "linux" }), "default");
  assert.equal(platformScope({ platform: "linux", platform_preview: "windows" }), "windows");
});

test("the rail starts expanded on a wide window and compact on a narrow one", () => {
  assert.equal(initialNavMode(1440), "expanded");
  assert.equal(initialNavMode(900), "compact");
});

test("the rail toggles both ways and labels its action", () => {
  assert.equal(nextNavMode("expanded"), "compact");
  assert.equal(nextNavMode("compact"), "expanded");
  assert.equal(navToggleLabel("expanded"), "Collapse navigation");
  assert.equal(navToggleLabel("compact"), "Expand navigation");
});

test("accent contrast follows the accent luminance", () => {
  assert.equal(accentVars("#0078D4", false).accentContrast, "#ffffff");
  assert.equal(accentVars("#FCE100", false).accentContrast, "#000000");
  assert.match(accentVars("#0078D4", true).accentSoft, /^rgba?\(/);
});
```

- [ ] **Step 2: Run the test and confirm it fails**

Run: `node --test ui/js/windows-theme.test.mjs`  
Expected: FAIL because `./windows-theme.js` does not exist.

- [ ] **Step 3: Implement the pure helpers in `ui/js/windows-theme.js`**

`platformScope` returns `"windows"` when `platform_preview` or `platform` is `"windows"`, otherwise `"default"`. `initialNavMode` returns `"compact"` below 1000px. `accentVars` must treat the hex as sRGB, return `#ffffff` contrast for dark accents and `#000000` for light accents, and produce an alpha-mixed `accentSoft` string.

- [ ] **Step 4: Run the helper test**

Run: `node --test ui/js/windows-theme.test.mjs`  
Expected: PASS.

- [ ] **Step 5: Add the Tauri theme plumbing**

In `src-tauri/src/theme_windows.rs`, implement `system_accent()` using `windows::Win32::Graphics::Dwm::DwmGetColorizationColor`. Return `#rrggbb` from the RGB bits; return `None` on any failure. Add the `Win32_Graphics_Dwm` feature to the existing Windows `windows` dependency in `src-tauri/Cargo.toml`.

In `src-tauri/src/commands.rs`:

- add `platform_preview: Option<String>` and `accent: Option<String>` to `AppInfo`;
- parse `--platform-preview=windows` into `platform_preview`;
- set `accent` from `theme_windows::system_accent()` on Windows and `None` elsewhere.

In `src-tauri/src/lib.rs`, add the `theme_windows` module behind `#[cfg(target_os = "windows")]`.

- [ ] **Step 6: Apply the marker in `ui/js/app.js`**

After `api.appInfo()` resolves:

- keep `ctx.platform` as the real platform for updater behaviour;
- set `document.documentElement.dataset.platform = platformScope(info)`;
- set `document.documentElement.dataset.nav = initialNavMode(window.innerWidth)`;
- apply `accentVars(info.accent || fallbackAccent, dark)` to the root CSS variables;
- log `theme: platform=... nav=... accent=...` through `window.uilog` for diagnostics.

The fallback accent is `#0067c0` in light and `#60cdff` in dark. Re-apply the accent variables when `matchMedia("(prefers-color-scheme: dark)")` changes.

- [ ] **Step 7: Verify compile and tests**

Run: `node --check ui/js/app.js`  
Run: `node --test ui/js/windows-theme.test.mjs`  
Run: `cd src-tauri && cargo check && cargo test`  
Expected: all pass; on Linux `accent` is `null` and the Windows helper is not compiled.

- [ ] **Step 8: Commit**

```bash
git add ui/js/windows-theme.js ui/js/windows-theme.test.mjs ui/js/app.js \
  src-tauri/src/commands.rs src-tauri/src/theme_windows.rs src-tauri/src/lib.rs \
  src-tauri/Cargo.toml
git commit -m "feat(theme): add the windows platform marker and accent helpers"
```

---

### Task 2: Windows token layer and light/dark themes

**Files:**
- Create: `ui/css/windows.css`
- Modify: `ui/index.html`

**Interfaces:**
- CSS scope: `:root[data-platform="windows"]`
- Dark scope: `@media (prefers-color-scheme: dark) { :root[data-platform="windows"] { ... } }`
- Tokens consumed by later tasks: `--bg`, `--bg-rail`, `--surface`, `--surface-2`, `--surface-3`, `--border`, `--border-soft`, `--border-strong`, `--text`, `--muted`, `--faint`, `--accent`, `--accent-soft`, `--accent-hover`, `--accent-contrast`, `--ok-text`, `--caution-text`, `--over-text`, `--radius`, `--radius-sm`, `--shadow`, `--font-ui`, `--font-display`.

- [ ] **Step 1: Add the Windows stylesheet link**

In `ui/index.html`, add after the base stylesheet:

```html
<link rel="stylesheet" href="css/windows.css" />
```

The base `ui/css/app.css` remains unchanged.

- [ ] **Step 2: Create the token layer**

In `ui/css/windows.css`, define the Windows light tokens and the `prefers-color-scheme: dark` overrides from the design spec. Scope every rule under `[data-platform="windows"]`. Set `--font-ui` to Segoe UI Variable Text/Segoe UI and `--font-display` to Segoe UI Variable Display/Segoe UI. Set the light/dark surface, border, text, status and shadow values from the design spec. Do not restyle the report.

- [ ] **Step 3: Add a CSS guard test**

Extend `ui/js/windows-theme.test.mjs` with a read of `ui/css/windows.css`:

```js
import { readFileSync } from "node:fs";
const windowsCss = readFileSync(new URL("../css/windows.css", import.meta.url), "utf8");
test("the windows layer is scoped and has a dark variant", () => {
  assert.match(windowsCss, /\[data-platform="windows"\]/);
  assert.match(windowsCss, /prefers-color-scheme:\s*dark/);
  assert.match(windowsCss, /--font-ui/);
});
```

- [ ] **Step 4: Run the tests and preview both themes**

Run: `node --test ui/js/windows-theme.test.mjs`  
Run: `cd src-tauri && cargo run -- --platform-preview=windows`  
Expected: the Windows preview starts with the Linux layout but Windows tokens. Switch the host light/dark preference and confirm the surfaces change. Mica is not present on Linux; that is expected.

- [ ] **Step 5: Commit**

```bash
git add ui/css/windows.css ui/index.html ui/js/windows-theme.test.mjs
git commit -m "feat(theme): add the windows fluent token layer"
```

---

### Task 3: Mica window configuration

**Files:**
- Create: `src-tauri/tauri.windows.conf.json`

**Interfaces:**
- Windows-only Tauri config merged with `tauri.conf.json` at Windows build time.

- [ ] **Step 1: Add the Windows window config**

Create `src-tauri/tauri.windows.conf.json` with a `app.windows[0]` object that keeps the existing title/size/min size, sets `transparent: true`, and sets:

```json
"windowEffects": { "effects": ["mica"] }
```

Keep native decorations. Do not add a custom title bar.

- [ ] **Step 2: Validate the config**

Run: `python3 -m json.tool src-tauri/tauri.windows.conf.json >/dev/null`  
Expected: valid JSON, no output.

- [ ] **Step 3: Verify the Windows build path**

Run: `cd src-tauri && cargo check` on Linux.  
Expected: passes, because the Windows config is read only by the Windows build. The actual Mica check is part of the Windows QA task.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tauri.windows.conf.json
git commit -m "feat(theme): apply mica to the windows window"
```

---

### Task 4: Fluent NavigationView shell

**Files:**
- Modify: `ui/js/app.js`
- Modify: `ui/css/windows.css`
- Modify: `ui/js/windows-theme.test.mjs`

**Interfaces:**
- `data-nav="expanded" | "compact"` on `<html>`.
- `#rail-toggle` button with `aria-expanded` and `aria-label` from `navToggleLabel`.
- Existing nav buttons keep `aria-current="page"` for the active tab.

- [ ] **Step 1: Extend the helper tests**

Add assertions for the initial/toggle labels and the 1000px breakpoint. The existing `initialNavMode` and `nextNavMode` tests already cover the state; add a test that the nav toggle label changes with the state.

- [ ] **Step 2: Add the rail header and toggle**

In `ui/js/app.js`, add a Fluent hamburger button to the rail header. Its click handler flips `document.documentElement.dataset.nav` with `nextNavMode`, updates `aria-expanded` and `aria-label`, and keeps the focus on the toggle. Add `aria-current="page"` to the active nav button on every navigation.

- [ ] **Step 3: Add the expanded/compact CSS**

In `ui/css/windows.css`, under the Windows scope:

- `.app` uses `grid-template-columns: var(--rail-w) 1fr`; `--rail-w` is 240px expanded and 56px compact.
- Compact mode hides `.nav-group`, `.nav-item span:not(.key)`, `.rail-logo .name`, `.rail-logo .kind`, and the footer text.
- Compact nav items are 40×40px, centered, with their existing `title` as the accessible tooltip.
- Active item gets a subtle selection fill and a 3px accent bar.
- The rail keeps its scroll behavior and footer at the bottom.
- Under 1000px, the initial JS state is compact; the toggle still expands it.

- [ ] **Step 4: Verify the shell**

Run: `node --test ui/js/windows-theme.test.mjs`  
Run: `cd src-tauri && cargo run -- --platform-preview=windows`  
Check expanded and compact at 1440×950 and 720×540. Keyboard: Tab to the toggle, Enter to collapse, focus stays visible. Ctrl+1…8 still navigate.

- [ ] **Step 5: Commit**

```bash
git add ui/js/app.js ui/css/windows.css ui/js/windows-theme.test.mjs
git commit -m "feat(theme): add the fluent navigation view shell"
```

---

### Task 5: Fluent shell and control surfaces

**Files:**
- Modify: `ui/css/windows.css`
- Modify: `ui/js/dom.js` (only if a shell icon is missing)

**Interfaces:**
- Existing markup classes are the contract: `.topbar`, `.update-bar`, `.card`, `.btn`, `.field`, `.row-card`, `.kv`, `.badge`, `.meter`, `.table-wrap`.
- Add no new component runtime.

- [ ] **Step 1: Style the topbar and update bar**

In `ui/css/windows.css`, add Fluent rules for the topbar title, subtitle, status chips, and the update InfoBar. Use the Windows font tokens, 20px display title, 12px subtitle, 4px control radius and accent focus.

- [ ] **Step 2: Style cards, buttons, fields and selects**

Add Windows scoped rules for:

- cards: 8px radius, 1px border, subtle shadow, 12/16px padding;
- buttons: 32px height, 4px radius, primary/secondary/ghost surfaces; no translate press;
- text/number inputs and selects: 32px height, 4px radius, accent focus underline;
- labels and hints: 12px, medium;
- rows and tables: 40px list rows;
- badges/meters: Fluent status colours and shapes;
- focus: 2px accent outline with 2px offset;
- `@media (forced-colors: active)` coverage.

- [ ] **Step 3: Add the shell icons if needed**

In `ui/js/dom.js`, add a `menu`/`hamburger` icon and a `chevron` if the Fluent shell needs them. Keep the existing 16px line-icon style, 1.5px stroke and rounded caps. Do not use emoji or a font icon.

- [ ] **Step 4: Verify surfaces**

Run: `node --check ui/js/app.js`  
Run: `cd src-tauri && cargo run -- --platform-preview=windows`  
Inspect all tabs in light and dark, expanded and compact. Check that no field, button or status text overflows and that the report preview stays neutral.

- [ ] **Step 5: Commit**

```bash
git add ui/css/windows.css ui/js/dom.js
git commit -m "feat(theme): style the windows fluent shell and controls"
```

---

### Task 6: Screen QA and responsive fixes

**Files:**
- Modify: `ui/css/windows.css`
- Modify: screen files only if a class hook is missing.

**Interfaces:**
- No new API; this task fixes what the rendered screens expose.

- [ ] **Step 1: Run the Windows preview at 1440×950**

Walk every tab: Overall Weight, Crane, Outrigger Pad Size, Graph, Uniform Load, Nonuniform Load, Tandem, Summary, About. Check expanded and compact, light and dark.

- [ ] **Step 2: Run the Windows preview at 720×540**

Check the same tabs. Fix any overflow, clipped label, stacked-pane collision or unreachable control in `ui/css/windows.css`. Do not change form order or hide required inputs.

- [ ] **Step 3: Check the hard cases**

- Long crane/sling description selects.
- The Graph readout with long figures.
- The Summary report list and preview controls.
- The update bar with four actions.
- The Outrigger thickness card with long notes.
- `forced-colors` mode and keyboard focus.

- [ ] **Step 4: Re-run the tests**

Run: `node --test ui/js/windows-theme.test.mjs`  
Run: `cd src-tauri && cargo test`  
Expected: all pass; no report or calculation changes.

- [ ] **Step 5: Commit**

```bash
git add ui/css/windows.css ui/js/screens
git commit -m "fix(theme): polish the windows fluent screens at both widths"
```

---

### Task 7: Windows QA, docs and screenshots

**Files:**
- Modify: `README.md`
- Modify: `docs/tutorial.md`
- Create: `docs/screenshots/windows-11-light.png`
- Create: `docs/screenshots/windows-11-dark.png`
- Create: `docs/screenshots/windows-11-compact.png`

**Interfaces:**
- No code interface; this is the user-facing proof and the Windows fallback record.

- [ ] **Step 1: Capture the Linux preview screenshots**

Use `--platform-preview=windows` to capture light, dark and compact. These show the shell and tokens but not Mica; label them as layout previews. Capture on the Windows build for the final Mica screenshots if a Windows machine is available.

- [ ] **Step 2: Write the Windows 11 section**

In `README.md` and `docs/tutorial.md`, add a short section:

- the Windows build uses Fluent and Mica;
- it follows system light/dark and accent;
- the rail collapses to icons;
- Windows 10 gets the solid Fluent fallback;
- the report and PDF are unchanged.

- [ ] **Step 3: Run the Windows manual checklist**

On Windows 11 22H2/23H2/24H2 and Windows 10:

- install the MSI/NSIS build;
- light/dark, accent change, Mica on/off;
- 125%/150% scaling;
- compact/expanded rail;
- keyboard-only navigation;
- high contrast;
- report preview and PDF unchanged.

- [ ] **Step 4: Verify and commit**

Run: `node --test ui/js/windows-theme.test.mjs`  
Run: `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings`  
Run: `git diff --check`  
Commit the docs and screenshots.

```bash
git add README.md docs/tutorial.md docs/screenshots
git commit -m "docs: document the windows 11 fluent shell"
```

---

## Self-Review

- **Spec coverage:** platform scope, Windows tokens, system accent, Mica, NavigationView, Fluent controls, light/dark, compact rail, Windows 10 fallback, report unchanged and Windows QA each have a task.
- **Step scan:** each step has a command or a concrete render check; CSS steps name the selector contract and the values come from the spec.
- **Type consistency:** `platformScope`, `initialNavMode`, `nextNavMode`, `accentVars`, `navToggleLabel`, `AppInfo.platform_preview`, `AppInfo.accent` and `system_accent()` are used consistently.
- **Review Focus:** the five high-risk cases are assigned to Task 6 and Task 7.
- **Proportion:** the plan leaves CSS bodies and screen fixes to the implementer; it locks the scope, file boundaries, tokens, interfaces and tests.
