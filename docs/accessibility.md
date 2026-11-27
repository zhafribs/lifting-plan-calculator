# Accessibility

**Status:** 27 November 2026 · review at v2.17.23. A real screen-reader pass has
**not** been run: Orca and AT-SPI tooling are not installed in the development
environment. This page is a code and keyboard audit, and the checklist for the
pass that is still owed.

## What the audit found, and what was fixed

| Area | Finding | State |
| --- | --- | --- |
| Toasts | Two notices in one frame raced for the live region; only the last was written | Fixed: `ui/js/live-announcer.js` serialises writes with a 350 ms gap, de-duplicates a repeat and bounds a burst |
| Toasts | Errors carried `role="alert"` on the box holding the close button | Fixed earlier: a pre-existing `#live-alert` region announces, the box is plain content |
| Toasts | A fifth error could push out an unread error | Fixed for identical repeats: they are counted on the existing notice (`×2`), so a repeating failure evicts nothing. A repeat inside the 350 ms gap is counted but not re-announced. Five *distinct* errors are still capped at four |
| Toasts | Every close button was named "Dismiss" | Fixed earlier: `Dismiss: <first words of the message>` |
| Zoom buttons | "−" and "+" were the accessible names | Fixed for the glyph buttons ("Zoom out", "Zoom in"); "Fit width" and "Refresh" keep their visible names |
| Load and tackle rows | The remove buttons relied on `title` alone | Fixed: explicit `aria-label` ("Remove load line", "Remove tackle line") |
| Working-range canvas | No accessible name | Fixed: `role="img"`, `aria-label="Working range diagram"`. It is pointer-only; the values are typed in the inputs and listed in the readout |
| Report pages | The focusable page region had no name | Fixed: `role="region"`, `aria-label="Report pages"` |
| Page-first preview | The overlay is visual; the bar carries "Rendering…" and "Page 1" | The placeholder is `aria-hidden`; the preview adds no announcement |
| Chart ticks | 3.77:1 on the light chart | Accepted: it is the printed page's own grey, kept so screen and paper match |
| Focus | The app relies on the engine's default ring except where a component overrides it | No change; the overrides are visible |

## What still needs a real screen reader

The Windows script is [`narrator-pass.md`](narrator-pass.md): exact steps, what
to listen for, and a results table. On Linux, run Orca (`orca` with
speech-dispatcher running) with the default output. Note the AT version and the
app build in the record. The scenarios are:

1. **Error and success toasts.** Fire two in quick succession; both should be
   announced. Tab to a notice: the name is "Dismiss: …". Enter removes it and
   focus moves to the next notice or back to the control that raised it. Escape
   dismisses.
2. **Page-1 preview.** Open Pages on a two-page plan. The bar should read
   "Rendering…", then "Page 1 of 2"; nothing should be announced twice when the
   overlay is replaced by the PDF.
3. **Graph tab.** Reach the boom, angle and crane fields by keyboard; the canvas
   is announced as "Working range diagram" but is not a control. Confirm the
   readout is reachable and read in a sensible order.
4. **Report sections.** Toggle each section with the keyboard; the checkbox and
   its card title should read as one label.
5. **Theme controls.** Change the scheme from Settings; the change should be
   announced once, and the chart should repaint without a page reload.

## Trade-offs recorded

- The chart is the one place where information is primarily visual. Every value
  it draws is also in a number field or the readout, so the canvas is named but
  not focusable.
- The light chart's tick grey is below the 4.5:1 text floor by design
  (`0069`); darkening it would break screen/print parity.
