# Lifting Plan Calculator: a practical tutorial

Welcome to the Lifting Plan Calculator. This guide walks through every tab in the order you will use them, using the same example lift as the screenshots: **20,000 kg of load, 350 kg of lifting tackle, and 20,350 kg at the hook**.

Think of it as the walkthrough I would give you at a demo table: no jargon, just the fields to fill and the checks that matter. You can read it from start to finish, or jump to the tab you need.

## Contents

1. [Before you start](#before-you-start)
2. [Overall Weight](#1-overall-weight)
3. [Crane](#2-crane)
4. [Graph](#3-graph)
5. [Uniform Load](#4-uniform-load)
6. [Nonuniform Load](#5-nonuniform-load)
7. [Tandem](#6-tandem)
8. [Summary](#7-summary)
9. [About](#8-about)
10. [Common questions](#common-questions)

## Before you start

- The app works in **kilograms and metres**. Convert the site figures before you type them in.
- Have the crane's own load chart ready. The app imports `.xlsx` workbooks and also accepts a single chart point typed by hand.
- The **75% rule** is the safety band used throughout: the required chart capacity is the gross load at the hook divided by 0.75. A 20,350 kg lift therefore needs at least **27,133.33 kg** of charted capacity.
- Nothing is stored between launches. Save the report if you want to keep a plan.
- This is a calculation aid. Check every input and every chart against the source before the lift, and have the finished plan approved by the competent person on site.

## Quick start

1. Add the load and the lifting tackle in **Overall Weight**.
2. Open the crane's Excel chart in **Crane**, or type a manual chart point.
3. Check the capacity band. Green is within 75%, amber is between 75% and 100%, red is over 100%.
4. Choose the sling arrangement in **Uniform Load**, **Nonuniform Load** or **Tandem**.
5. Review the **Summary** and save or print the plan.

---

## 1. Overall Weight

![Overall Weight tab with a load item and a tackle item](screenshots/01-overall.png)

The Overall Weight tab is where every lift starts. It adds the load lines and the rigging into one gross load at the hook.

### Add a load item

1. Click **Add load item**.
2. Give the item a description, for example `steel plate`.
3. Choose **Measured by**:
   - **Fixed** uses a weight for each item. Enter the weight in kilograms and the quantity.
   - **Weight by Metre** uses a weight per metre and a length. This suits pipe, beam and cable.
   - **Weight by Volume** uses a volume and a density. This suits tanks and bulk material.
4. Enter the quantity. The subtotal updates as you type.
5. Add another line for every separate item in the lift.

### Add lifting weight

1. Click **Add tackle item**.
2. Describe the item, for example `4-leg chain sling`, `shackles` or `hook block`.
3. Enter the quantity and the weight of each item in kilograms.
4. Repeat for everything that rides with the load: slings, shackles, spreader beams, the hook block and any attached rigging.

The app calls this **Lifting tackle**. It is the lifting weight that gets added to the load. You do not need a separate field for it.

### Read the Weight summary

The card on the right shows:

- **Total load weight**: the sum of the load lines.
- **Lifting tackle weight**: the sum of the rigging.
- **Overall weight**: the gross load at the hook. This is the figure the Crane tab checks.
- **Min. crane rating (75%)**: the charted capacity the crane must have at the working radius. It is the overall weight divided by 0.75.

When the figures look right, click **Check the crane**.

---

## 2. Crane

![Crane tab with an imported Excel load chart](screenshots/09-crane-excel.png)

The Crane tab does two things: it records which crane is on the job, and it checks the gross load against the crane's load chart at the working radius.

### Crane identity

Enter the **Crane name** and the **Crane rated capacity**. The rated capacity is printed on the plan for the record. The charted capacity at the working radius is the figure used for the 75% check.

### Open an Excel load chart

1. Click **Open Excel load chart...**.
2. Choose an `.xlsx` file. Files must be `.xlsx`, not `.xls` or `.xlsm`, and the workbook must be 24 MB or smaller.
3. The app looks through the sheets and uses the first load chart it recognises.

Two chart layouts are supported:

- **Column chart**: a header row with a radius column and a load column. A boom column is optional. Useful header names include `Radius`, `Working radius`, `R`, `Load`, `Rated load`, `Capacity`, `SWL`, `Boom`, `Boom length` and `Length`.
- **Matrix chart**: boom lengths across the top row, working radius down the first column, and capacities in the cells.

A title line above the chart can name the crane and its rating, for example `Tadano TL200M (20 ton)`. The app reads that line and fills in the crane name and rated capacity for you.

Things worth knowing about the import:

- Put the unit in the load column header, or add a line that says `Unit: ton` or `Unit: kg` above the chart. A header containing `ton` or `kg` also works. The app converts tonnes to kilograms. If no unit is shown, it treats the figures as kilograms.
- In a matrix chart, values of 2,000 or less are assumed to be tonnes, and larger values are assumed to be kilograms, unless the workbook declares a unit.
- A matrix chart is read from the longest main boom by default. You can choose another boom length from the **Boom length** list after the import.
- The **Working radius** list shows the charted radii at the selected boom.
- If the workbook carries a jib sheet, a **Jib configuration** card appears. Choose the main boom, boom with jib, or boom with extended jib, then set the boom angle and jib offset angle.
- The app suggests the farthest radius that still meets the 75% rule. Enter the lift weight first, or the suggestion cannot be calculated.

If the workbook has a workbook table, the **Workbook table** toggle shows the rows the app read. This is a quick way to confirm the import picked up the right columns.

### Use manual data

Manual entry is useful when you only have a single chart point, or a company-specific chart that is not in a workbook.

1. Fill in **Radius**, **Rated load**, **Boom angle** and **Boom length**.
2. The chart source switches to **Manual chart**.
3. Check the capacity card on the right.
4. To start another crane or company, click **Clear** and enter the new figures.

Manual data is treated as the chart for this plan. It is not saved when the app closes.

### Read the capacity check

![Crane tab with a manual chart and the 75% capacity check](screenshots/02-crane.png)

The badge and meter on the right show the result:

- **WITHIN LIMIT** is green. Capacity usage is 75% or less.
- **CAUTION** is amber. Capacity usage is above 75% and up to 100%.
- **WARNING** is red. Capacity usage is above 100%.

The rows underneath show the crane, boom length, boom angle, working radius, chart capacity, gross load at the hook and capacity usage. The plot marks the working radius on the chart curve.

The chart is the rating that matters. The **Crane rated capacity** field is printed for the record and is not used as a second pass or fail test. A working radius outside the chart's rated range is refused, and the app names the range the chart covers.

---

## 3. Graph

![Working range diagram](screenshots/03-graph.png)

The Graph tab draws the working range to scale. It is a visual check on the crane setup, not a separate calculation.

- **Boom configuration** lets you choose the main boom, boom with jib, or boom with extended jib. The jib fields light up when they apply.
- **Boom angle & envelope** sets the boom angle and the envelope from and to angles. **Show dimensions** adds the measurements to the drawing. **Auto-fit view** keeps the diagram in the window. **Fit view** redraws it on demand.
- **Crane location on axis (X, Y)** sets where the crane sits. You can type the position, use one of the quick buttons, or drag the crane marker on the diagram.
- **Readout** shows the working radius, tip height, envelope radius, boom angle, tip location and ground clearance.

When a new-format workbook is loaded in Crane, the diagram follows the crane position, boom length and working radius from that tab. If you edit the diagram by hand, a **Reset to Crane Tab** button appears so you can return to the workbook position.

To put the diagram in the report, tick **Working range diagram** in the Summary tab.

---

## 4. Uniform Load

![Uniform Load tab with a sling result](screenshots/04-uniform.png)

Use Uniform Load when one continuous sling shares the load across its legs. The app lists 19 arrangements, including the common one-leg, two-leg, nested and bridle setups.

1. Choose the **Arrangement**.
2. Enter the **Sling total length**.
3. Fill in the load dimensions the selected arrangement asks for: load diameter, side or height, bottom or length.
4. If the arrangement uses a choke, set the **Choke angle** and the **Angle basis**.
5. For pick-point arrangements, enter the pick-point distance, length and width.
6. For nested arrangements, fill in the **Secondary slings** card.
7. Enter the **Tag WLL** and **Tag WLL angle** for the installed tag.

The **Sling result** card shows the effective legs, sling angle factor, sling angle, headroom, tension per leg, maximum tension and required WLL. The **Installed tag check** compares the tag rating with the required WLL. A green result means the tag meets the requirement. A red result means it does not.

If no tag rating is entered, the app still shows the required WLL and reports that there is no tag rating to compare.

---

## 5. Nonuniform Load

![Nonuniform Load tab with the asymmetric bridle result](screenshots/05-nonuniform.png)

Use Nonuniform Load for a two-leg asymmetric bridle with a shortening grab hook. The centre of gravity is off the midpoint, so the two legs carry different tensions.

1. Enter the **Sling length**.
2. Enter the **Distance between pick points**.
3. Enter the **COG from pick point 1**. The COG can be measured from either pick point.
4. Enter the **Height difference (2 above 1)**. A negative value means pick point 2 sits lower.
5. Fill in the **Installed tags** card for sling 1 and sling 2: tag WLL, sling legs and tag WLL angle.

The solver works out which leg is long and shortens the other with the grab hook. The **Bridle result** shows the geometry, and the **Sling comparison** table puts sling 1 and sling 2 side by side. The **Installed tag check** compares each tag with its required WLL.

---

## 6. Tandem

![Tandem tab with the two-crane check](screenshots/06-tandem.png)

Use Tandem for two independent cranes sharing one load.

1. Choose the **Tandem method**:
   - **Aligned COG Pivot** is for up-ending and load rotation, including full 90 degree vertical rotation.
   - **Repositioned Top Lifting Point** is for level-shifting and drift lifts where the load cannot go vertical.
2. Enter the **Distance, crane 1 hook to COG** and **Distance, crane 2 hook to COG**.
3. Enter the **COG depth below baseline** and the **Cargo tilt angle**.
4. Fill in each crane's tag WLL, sling legs, tag WLL angle and rated capacity.

The right side shows the tilted geometry, Case A (level baseline), Case B (declared tilt), and Case C (full 90 degree vertical rotation) when the method allows it. The **Governing case, per crane** card names the case that sizes each crane. **Crane checks** then compares each crane's required WLL and rated capacity.

The method note under the dropdown explains the second crane's projection rule and the rotation limit. Read it before you choose a method.

---

## 7. Summary

![Summary tab with the printable report preview](screenshots/07-summary.png)

Summary turns the inputs into a printable plan. Every figure is shown with the formula and the values that produced it, so a reviewer can check the arithmetic.

### Choose the report sections

Tick the sections you want:

- **Crane**
- **Uniform Load**
- **Nonuniform Load**
- **Tandem**
- **Working range diagram**

The preview follows the other tabs live.

### Fill in the plan details

Enter the **Plan reference**, **Prepared by**, **Checked by** and **Approved by**. These lines are printed on the plan and are not used by the calculation engine.

You can also edit the **Footer** centre text. It is printed in the centre of every page.

### Preview and export

- **Pages** shows the report as the PDF pages that will be saved, with page breaks. **Live** shows the HTML preview as you type.
- Use **Fit width**, the zoom buttons and **Refresh** to inspect a page.
- **Margins** opens the page margin panel. The default is 25.4 mm, and the presets cover common metric and imperial sizes.
- **Save as HTML** writes a portable copy with its KaTeX assets beside it.
- **Save as PDF** writes A4 pages with the margins you set and the footer on every page.
- **Print** opens the system print dialog.
- **Open in browser** opens the saved report in your default browser.

If the desktop cannot print from the preview, save the HTML and print it from a browser. Keep the browser's margin setting on **Default** so the app's A4 margin is not overridden.

---

## 8. About

![About tab](screenshots/08-about.png)

The About tab shows the app version, what the calculator does, the developer contact details and the update controls.

- **Check for updates** asks GitHub whether a newer release exists.
- **Visit repository** opens the project on GitHub.
- On Windows, when a newer release is available, the update card can download and start the installer. The Linux AppImage is updated from its release page.

The app needs no sign-in, and all calculations run on your machine.

---

## Common questions

### Why does the Crane tab say "No capacity found"?

The gross load at the hook is still zero. Add a load item in **Overall Weight** and the check will run. The message "Enter the lift weight to check capacity" is the app's way of saying it has no load to compare against the chart.

### Why does the app refuse my working radius?

The radius is outside the range the chart covers. Pick a radius inside the chart, or check that the correct boom length is selected. The app will not borrow the nearest charted capacity from outside the rated range.

### Why does the import say it cannot find a load chart?

The workbook needs either a header row with radius and load columns, or a matrix with boom lengths across the top and radius down the first column. Rename the headers to plain names such as `Radius` and `Load`, or move the chart onto its own sheet. Save `.xls` and `.xlsm` files as `.xlsx` first.

### Why is the tag check red?

The installed tag's rating is below the required WLL for the arrangement. Use a higher-rated tag, add more legs where the arrangement allows it, or reduce the sling angle.

### Why does the Summary say "No pages"?

The report has no sections selected, or the preview has not drawn yet. Tick at least one section and give the app a moment to redraw. The page view waits until typing stops.

### Where did my plan go?

The app does not store plans between launches. Use **Save as HTML** or **Save as PDF** in Summary to keep a copy.

---

## Keyboard shortcuts

| Tab | Shortcut |
| --- | --- |
| Overall Weight | Ctrl+1 |
| Crane | Ctrl+2 |
| Graph | Ctrl+3 |
| Uniform Load | Ctrl+4 |
| Nonuniform Load | Ctrl+5 |
| Tandem | Ctrl+6 |
| Summary | Ctrl+7 |
| About | No shortcut |

---

## A safe workflow

1. Enter the load and all lifting tackle in Overall Weight.
2. Import the crane's current load chart, or type the chart point from the company's data.
3. Check the working radius and the 75% band.
4. Choose the rigging arrangement and check the required WLL against the tags.
5. Open Summary, fill in the plan details and include the sections you need.
6. Save the PDF or HTML, print it, and check the finished plan against the source chart before it is approved.

That is the whole app. If you get stuck, the message on screen usually names the field or the limit that needs attention.
