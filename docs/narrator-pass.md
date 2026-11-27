# Narrator pass — Windows 11

The code audit in [`accessibility.md`](accessibility.md) has never been heard by
a screen reader. This is the pass that turns it into evidence. It takes about
15 minutes on the Windows 11 VM, and Narrator is already installed.

What this pass covers: a single failure notice and its dismissal, the page-1
preview swap, the chart's name, the report section switches, and the theme
control. What it cannot cover by hand: two messages in the same frame — that is
covered by `ui/js/live-announcer.test.mjs`, because no human action can fire two
at once.

## You need

- The Windows 11 VM.
- Lifting Plan Calculator **2.17.23** — the MSI or setup from the release page.
- About 15 minutes and a quiet room.

## Setup

1. Install 2.17.23. If SmartScreen warns, choose **More info → Run anyway**.
2. Start Narrator: **Win+Ctrl+Enter**. If Narrator Home opens, close it with
   **Alt+F4**.
3. Optional: **Win+Ctrl+N** opens Narrator settings; pick a voice and speed you
   can follow, then close it.
4. Learn the two keys this pass uses: **Tab** moves between controls, and
   **Caps Lock+Space** toggles scan mode (arrow keys then read static text).
   Narrator reads the focused control by itself.
5. Prepare a plan worth printing:
   - **Ctrl+1** (Overall Weight) → **Add load item** twice → type a name and a
     weight in each row.
   - **Ctrl+2** (Crane) → **Load example**.
   - **Ctrl+8** (Summary). The report should be two pages; if it is one, add
     another load item.
6. Leave the app on Summary.

## Scenario 1 — a failure notice and its dismissal

1. **Ctrl+2** (Crane).
2. **Tab** to **Open Excel load chart...** and press **Enter**.
3. In the file picker choose a file that is not a workbook. If you do not have
   one, make `C:\Users\Public\bad.txt` in Notepad first; the picker has an
   **All files** filter.
4. **Listen:** Narrator should announce the failure message once. Write down the
   exact words.
5. With the notice on screen, press **Tab** until it reaches the close button.
   Narrator should say **"Dismiss: <first words of the message>, button"**.
   Press **Enter**. Focus should move to the next notice, or back to the control
   that opened the picker — not silently to nowhere.
6. Repeat steps 2–3 with the same file. The notice should show **×2**.
   **Listen:** does Narrator announce the repeat? Note yes or no.
7. If two notices are on screen, focus one close button and press **Escape**; the
   notice should disappear.

**Good looks like:** one clear message, a named close button, focus does not
vanish, Escape works, and the repeat is either announced or visibly counted —
not both silently.

## Scenario 2 — the page preview

1. **Ctrl+8**. The bar should show **Pages** selected.
2. In scan mode, read the bar while the PDF prints: the status should go from
   **"Rendering…"** to nothing, and the page count to **"Page 1 of 2"**.
3. **Tab** to the pages area. Narrator should say **"Report pages, region"**.
4. Switch to **Live** and back to **Pages** with the keyboard. **Listen:** is any
   message announced twice?
5. Watch the page: the report appears first, then the printed PDF replaces it.

**Good looks like:** the bar is readable, the page region is named, and the swap
is not announced as a second event.

## Scenario 3 — the working-range chart

1. **Ctrl+4** (Graph).
2. **Tab** through the fields (boom, angle, crane position). Narrator should read
   each label and value.
3. In scan mode, arrow to the chart. Narrator should say **"Working range
   diagram, image"**.
4. **Tab** to the **Readout** card and read the rows.

**Good looks like:** every input is named, the chart is named and not a control,
and the readout is reachable.

## Scenario 4 — report section switches

1. **Ctrl+8** (Summary).
2. **Tab** from the top: Pages/Live, zoom, margins, then the section rows.
3. On **Plan details**, Narrator should read the title and the checkbox state
   ("checked"). Toggle it with **Space**; the state should change.
4. Note whether the name includes the description text, and whether it is too
   long to be useful.

**Good looks like:** the title and state are clear; the description does not bury
the title.

## Scenario 5 — the theme

1. Open **Settings** from the rail (no shortcut; Tab to it or click it).
2. **Tab** to **Colour scheme**; change it to **Dark**, then **Light**. Narrator
   should read the selected option.
3. With the chart open (**Ctrl+4**), confirm by eye that it repaints; Narrator has
   nothing to say about the colours themselves.

**Good looks like:** the select is named and its value is read; nothing is
announced twice.

## Results to send back

Fill this in and send it with the app version, your Windows build, and the
Narrator voice you used:

| # | Step | What Narrator said | Expected? | Notes |
| --- | --- | --- | --- | --- |
| 1 | Failure announced | | yes / no / partial | |
| 2 | Close button named | | yes / no / partial | |
| 3 | Focus after Enter | | yes / no / partial | |
| 4 | Repeat (×2) spoken | | yes / no | |
| 5 | Escape dismisses | | yes / no | |
| 6 | Preview bar readable | | yes / no / partial | |
| 7 | "Report pages, region" | | yes / no / partial | |
| 8 | No double announcement | | yes / no | |
| 9 | Inputs read | | yes / no / partial | |
| 10 | "Working range diagram, image" | | yes / no / partial | |
| 11 | Section title + state | | yes / no / partial | |
| 12 | Colour scheme read | | yes / no / partial | |

You can paste the table into the chat, or save it as a file and point me at it.

## What happens with it

Every mismatch is fixed, the fixes are covered by the existing probes where they
can be, and the pass is recorded in the next council record with the Narrator
version and the Windows build. If something cannot be fixed, it is written into
`docs/accessibility.md` as an accepted limit rather than left implied.
