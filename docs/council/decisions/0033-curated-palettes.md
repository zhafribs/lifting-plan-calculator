# 0033 — A small curated palette set

**Status:** accepted
**Date:** 2026-11-19
**Release:** v2.17.0
**Change:** `ui/js/theme.js`, `ui/js/screens/settings.js`, `ui/js/app.js`,
`src-tauri/src/appearance.rs`, `ui/js/windows-theme.js`, tests and docs
**Seats:** Product Manager, UI/UX Designer, Accessibility Reviewer, Frontend
Engineer, QA & Test Engineer, Release Manager. The Release Manager chaired; the
implementer did not score.

## Proposal

The operator asked for more colour than the desktop-derived palette, without the
app turning excessive or ridiculous.

The council considered three shapes:

1. **A free colour picker.** Rejected. Every new hue would need its own contrast
   check, and a saturated accent can make primary buttons, focus rings and
   status bands unreadable. It also invites a different palette per element.
2. **A large grid of named themes.** Rejected. Six or more palettes turn a
   calculation tool into a theme shop, and most of them would be unused.
3. **Three curated palettes beside the desktop.** Accepted. Each is a colour
   family with a light and a dark accent, applied through the existing token
   path; `Match the desktop` remains the default.

The accepted set is **Clay** (warm brown), **Lavender** (cool violet) and
**Pine** (muted green). The scheme (Match the desktop / Light / Dark) still
chooses the lightness; the palette chooses the family. There is no free picker.

## What the council saw

- **Product Manager:** three choices are enough to cover warm, cool and green
  without a list the operator has to study. The default stays the desktop, so
  nothing changes for someone who never opens Settings.
- **UI/UX Designer:** the palettes use the same surface ladder, so the Settings
  page, cards, fields and status bands keep their shape. The palette select sits
  under the scheme, not beside every colour.
- **Accessibility Reviewer:** the filled-button text colour now compares black
  and white directly. Clay's mid-tone accent previously took white at about
  2.9:1; it now takes black at about 7:1. The curated pairs were added to the
  contrast test. Status and capacity colours stay semantic in every palette.
- **Frontend Engineer:** one `applyTheme` path; a named palette replaces the
  desktop accent and forces the tinted ladder. The desktop object is still
  returned for `Match the desktop`, and no new dependency or CSS file is added.
- **QA & Test Engineer:** `node --test` → **23 passed** (18 theme, 5 Windows
  tokens); `cargo test` → **266 passed**, including the new palette
  normalisation and the old-file default; `cargo clippy --all-targets -- -D
  warnings` is clean. Clay was inspected in Settings at 720×540 and 1440×950.
- **Release Manager:** this is a minor release because it adds a setting. The
  report and arithmetic are untouched; the tag carries the accepted record.

## Decision

**Accepted.** Add exactly three curated palettes — Clay, Lavender and Pine —
under a Palette select in Settings, with `Match the desktop` as the default. No
free colour picker and no larger theme grid.

## Scores

| Seat | Score | Why it passed |
| --- | --- | --- |
| Product Manager | 8/10 | three families, one default, no theme shop |
| UI/UX Designer | 8/10 | the shell's shape survives every palette |
| Accessibility Reviewer | 9/10 | the mid-tone contrast bug is fixed and pinned by tests |
| Frontend Engineer | 8/10 | one token path, no new runtime |
| QA & Test Engineer | 8/10 | palette, fallback and contrast are covered |

**Mean 8.2/10** — at the bar.

## Known limits

- A named palette ignores the desktop's palette by design; `Match the desktop`
  is the way back.
- The setting is stored per user, not per plan; a plan never carries a theme.
- The palettes are fixed. A new colour requires a new contrast-checked pair and
  a council pass, not a picker.
