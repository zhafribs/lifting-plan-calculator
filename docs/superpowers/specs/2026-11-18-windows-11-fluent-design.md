# Windows 11 Fluent design

- **Status:** proposed, awaiting approval
- **Date:** 2026-11-18
- **Scope:** the Windows build only
- **Report/PDF:** unchanged

## Intent

The Windows build currently shares the Linux dark theme. It looks like a
cross-platform tool, not a Windows 11 application. This change gives the
Windows build a Fluent shell and controls while leaving Linux, macOS, the
calculation engine and the printed plan alone.

## Decisions from the meeting

- Windows-only Fluent redesign, not a separate fork of the app.
- Full Fluent NavigationView, with a native Windows 11 title bar and Mica.
- NavigationView starts expanded and can collapse to an icon rail.
- Follow the Windows system light/dark theme and system accent colour.
- Keep the printable report and PDF unchanged.
- Build it with a custom token layer and authored shell; do not vendor the
  Fluent UI component library.

## Goals

- A Windows 11 user recognises the shell as a native Windows application:
  NavigationView, Fluent cards, Fluent controls, Segoe UI Variable, Mica.
- Light and dark follow the system, including a live theme change.
- The system accent colour drives primary actions, selection and focus.
- The compact rail gives forms more width without losing the tab names on
  demand.
- Windows 10 degrades to a solid Fluent surface when Mica is unavailable.
- The report preview and PDF remain the neutral plan document.

## Non-goals

- No Linux or macOS restyle.
- No calculation, report, PDF or chart-format change.
- No custom title bar, custom window buttons or in-app window dragging.
- No Fluent UI Web Components, npm build step or component runtime.
- No Acrylic. Mica is the material; Acrylic's resize cost is not accepted.
- No persistence of the rail state between launches. The app keeps no session
  state by design.

## Constraints

- The frontend is plain ES modules embedded into the Tauri binary. There is no
  npm build step.
- The window CSP forbids inline scripts and inline handlers. Theme values are
  set through CSS custom properties, not inline event handlers.
- Tauri v2 window effects require a transparent window. Mica is applied through
  the window configuration, not a CSS blur.
- The Windows build must still run on Windows 10; Mica is Windows 11 only.
- The app currently starts at 1440×950 and must remain usable at its 720×540
  minimum.
- The report is a safety document. It keeps its current typography and colours.

## Approaches considered

### A. Custom Windows token layer plus authored Fluent shell (chosen)

Keep one HTML/JS tree. Add a platform marker, a Windows-only token layer, a
NavigationView state and Fluent control styles. No dependency, no build step,
and the Linux rule set stays byte-for-byte the fallback.

Trade-off: more CSS and interaction code to write, and the Fluent behaviour is
ours to test.

### B. Vendor Fluent UI Web Components

More native control behaviour out of the box, but it adds a component runtime,
a large vendored surface, more CSP work and a second styling system. Rejected:
the project deliberately has no package build and the existing controls are
small and already understood.

### C. Visual skin only

Rejected by the product decision: it would keep the Linux layout with Windows
colours, which is the outcome the request explicitly did not want.

## Architecture

### Platform marker

`app_info` already returns `platform`. At boot, after `api.appInfo()`:

- set `document.documentElement.dataset.platform = info.platform`;
- on Windows, also set `data-theme` from `prefers-color-scheme` only for
  components that need a JS branch, not for the main tokens;
- add `data-nav="expanded"` or `data-nav="compact"` for the NavigationView.

The default CSS remains the Linux theme. Windows rules are scoped under
`:root[data-platform="windows"]`. macOS is not a shipped target and receives the
default.

Add a QA hook `--platform-preview=windows` (read by `app_info`) so the Fluent
layout can be previewed and screenshotted on Linux. It sets the platform marker
only; Mica is not available outside Windows.

### Tokens

The Windows token layer is split into light and dark. The light set applies by
default inside the Windows scope; a `prefers-color-scheme: dark` block overrides
it. The values are Fluent-like, not a copy of a specific Microsoft app.

**Light**

- window/surface: `#f3f3f3`, `#ffffff`, `#f9f9f9`, `#f0f0f0`
- border: `#e5e5e5`, strong `#d1d1d1`
- text: `#1b1b1b`, muted `#5c5c5c`, faint `#8a8a8a`
- accent: system accent, fallback `#0067c0`
- status text: `#0f7b0f` (ok), `#9d5d00` (caution), `#c42b1c` (over)
- shadow: `0 2px 8px rgba(0,0,0,0.08)`

**Dark**

- window/surface: `#202020`, `#2b2b2b`, `#323232`, `#3a3a3a`
- border: `#3a3a3a`, strong `#4a4a4a`
- text: `#ffffff`, muted `#c7c7c7`, faint `#9a9a9a`
- accent: system accent, fallback `#60cdff`
- status text: `#6ccb5f` (ok), `#fce100` (caution), `#ff99a4` (over)
- shadow: `0 2px 8px rgba(0,0,0,0.35)`

Shape and type:

- cards: 8px radius, 1px border, subtle shadow
- controls: 4px radius, 32px height
- NavigationView items: 40px height, 4px radius
- body: Segoe UI Variable Text, 14px/1.45
- page title: Segoe UI Variable Display, 28px/1.2, semibold
- card heading: 20px/1.3, semibold
- labels: 12px, medium
- spacing: 4/8/12/16/24px
- focus: 2px system accent outline with 2px offset

The report keeps Inter and its existing colours.

### Accent colour

Windows exposes the user's colourisation colour through
`DwmGetColorizationColor`. Add a Windows-only helper behind the existing
`windows` crate, with the `Win32_Graphics_Dwm` feature. Return a hex string in
`app_info` or a separate `system_accent` command.

The UI sets:

- `--accent`
- `--accent-soft` (accent at 10–14% alpha)
- `--accent-hover` (slightly darkened/lightened)
- `--accent-contrast` (white or black, chosen by luminance)

If the API fails, use the Windows 11 fallback accent for the current theme. No
new crate is required.

### Mica

Add `src-tauri/tauri.windows.conf.json` with:

- `transparent: true`
- `windowEffects: { "effects": ["mica"] }`
- native decorations retained
- a solid Fluent fallback token set when the effect is unavailable

Mica is visible through the window chrome and narrow gaps. Dense content sits on
opaque or near-opaque surfaces so text contrast does not depend on the wallpaper.
No `backdrop-filter` is used for the app surfaces.

Windows 10 receives the same Fluent layout with the solid surface tokens; the
effect simply does not apply.

### NavigationView

- Expanded width: 240px; compact width: 56px.
- The rail keeps the existing groups and order.
- A Fluent hamburger in the rail header toggles expanded/compact.
- Compact mode hides labels and group headings, shows 20px icons with accessible
  names and tooltips, and keeps the active indicator.
- Active item: a subtle Fluent selection fill and a 3px accent bar.
- The rail footer keeps the app name/version/contact in expanded mode and shows
  only the app icon in compact mode.
- `aria-expanded` on the toggle; `aria-current="page"` on the active nav item.
- Keyboard: the toggle is focusable; existing Ctrl+1…8 shortcuts are unchanged.
- Under 1000px the rail starts compact; the user can still expand it.

The screen content and form order do not change. Only the shell and control
surfaces change.

### Components

- **Topbar:** Fluent title block, 20px display title, 12px subtitle, right-side
  status chips as Fluent badges.
- **Update bar:** Fluent InfoBar with accent icon and the existing actions.
- **Cards:** Fluent card surface, 8px radius, subtle border and shadow. The
  existing spark icon in the card head becomes a 16px Fluent line icon.
- **Buttons:** 32px high, 4px radius, 13px semibold. Primary uses accent;
  secondary uses surface with border; ghost uses transparent with hover fill.
  No `translateY` press effect.
- **Fields:** 32px high, 4px radius, 1px border, accent focus underline. Labels
  stay above the input. Selects use the Fluent chevron.
- **Rows/tables:** 40px Fluent list rows; existing column semantics unchanged.
- **Badges/meters:** Fluent status colours and shapes; the 75% meter keeps its
  meaning and labels.
- **Dialogs/popovers:** 8px radius, Fluent shadow, no new modal for existing
  inline panels.

### Icons

Use the existing authored SVG system with a Fluent pass: 16px grid, 1.5px
stroke, rounded caps and joins. Add only the icons the shell needs: hamburger,
chevron, settings/about, and a compact app mark. No emoji or font-based icon
substitution.

## What changes in the repository

- `ui/css/windows.css` (new): Windows token scope, Fluent shell, controls,
  cards, nav states, light/dark, forced-colors, compact rail. Loaded after the
  unchanged base `ui/css/app.css`.
- `ui/index.html`: one extra stylesheet link for the Windows layer.
- `ui/js/app.js`: set the platform marker; rail collapse state and toggle;
  update-bar/card class hooks; accent property.
- `ui/js/dom.js`: Fluent shell icons.
- `src-tauri/src/commands.rs`: `platform_preview` and, on Windows, accent.
- `src-tauri/src/theme_windows.rs` (new, Windows only): read the system accent.
- `src-tauri/Cargo.toml`: add the `Win32_Graphics_Dwm` feature for the accent colour.
- `src-tauri/tauri.windows.conf.json` (new): Mica/transparent window config.
- `ui/js/windows-theme.test.mjs` (new): token/selector and compact-nav state
  tests that can run without Windows.
- `docs/tutorial.md` / README: one section on the Windows 11 shell and the
  Windows 10 fallback.

No calculation, report or PDF builder changes.

## Testing

- Linux remains the default token set: existing UI tests and screenshots must
  not change.
- `--platform-preview=windows` screenshots at 1440×950 and 720×540, light and
  dark, expanded and compact.
- Windows 11: 22H2/23H2/24H2, light/dark, system accent change, Mica on/off,
  125%/150% scaling, high contrast, narrow window, keyboard-only navigation.
- Windows 10: solid Fluent fallback, no Mica, same layout.
- Contrast: body, muted, placeholder, status text and accent-on-surface meet
  4.5:1 where body size; large text 3:1.
- Focus visible on every interactive control; forced-colors mode keeps the
  focus indicator.
- No horizontal scroll at 720px; compact rail preserves form width.
- Report preview and PDF output unchanged.

## Risks and fallbacks

- **Mica does not render with native decorations on a target build.** Fall back
  to the solid Fluent surface; the layout remains.
- **System accent API fails.** Use the Windows 11 fallback accent; keep the
  same contrast derivation.
- **Light theme exposes contrast problems.** Ship both themes behind the same
  token contract and test the light theme first on Windows.
- **Collapsible nav adds state.** Keep the state session-only and make the
  toggle keyboard accessible; no plan data is stored.
- **Windows CI cannot render WebView2 screenshots on Linux.** Use the preview
  hook for Linux screenshots, a Windows manual checklist, and a DOM/token test
  for the platform scope.

## Approval

This document is the meeting's proposed design. No product code is written until
it is approved. Once approved, the implementation plan follows in a separate
step and the build proceeds in small milestones.
