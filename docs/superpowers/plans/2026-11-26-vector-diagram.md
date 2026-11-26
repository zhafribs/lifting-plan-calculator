# Vector Diagram Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Emit the working-range diagram as SVG for the report, so the PDF carries vectors while keeping the current layout, print colours and PNG fallback.

**Architecture:** A minimal Canvas2D-compatible SVG surface mirrors the primitives `graph.js` already uses; `paint()` writes into it exactly as it writes into a canvas. A shared print-colour function replaces the pixel pass. `renderDiagram` asks for SVG, falls back to the existing PNG on failure, and the capture cache keys the format.

**Tech Stack:** Plain ES modules, SVG, the existing `<canvas>` 2D context for text measurement, Node tests, the existing WebKit/WebView2 print path.

**Spec:** `docs/superpowers/specs/2026-11-26-vector-diagram.md`

**Status (2026-11-26):** Tasks 1–4 implemented; Task 5 verification in progress.
The PDF size and the forced PNG fallback are not yet recorded, so Steps 5.1–5.2
stay open.
**Spike result:** `pdfimages -list` reports no images and the labels extract as
text; the standalone SVG uses the system sans, recorded in ADR-0007 along with
the colour flattening and the PNG fallback.

## Global Constraints

- No new dependency, npm step or CSP change.
- The screen canvas, report layout, arithmetic and page structure do not change.
- Printed colours must match the existing `printify` filter result.
- Text stays text in the SVG; no outlines unless the spike proves a font cannot print.
- The PNG path remains the fallback and must still work.
- The capture cache key includes the format.

## Review Focus

- The spike must prove the diagram is vector in the exported PDF, not merely an SVG that WebKit rasterises.
- `measureText` must use a real 2D context so pill and label widths do not shift.
- Opaque and high-alpha colours match `printify` within ±2 over the diagram
  background; low-alpha overlaps are the documented approximation.
- A failed SVG build must still produce a report with the PNG diagram.
- A text-only report edit must not rebuild either format.

---

### Task 1: Spike — prove WebKit prints SVG as vector

**Files:**
- Temporary only; no production commit. Use a throwaway edit and revert it.

- [x] **Step 1: Make the diagram capture return a minimal SVG**

Temporarily replace the return in `renderDiagram` with a small SVG data URL
containing one line and one text label in the current print colours. Keep the
rest of the report untouched.

- [x] **Step 2: Export a PDF through the app**

Run the app, open Summary, tick the diagram, save a PDF to `/tmp`.

- [x] **Step 3: Inspect the PDF**

Run:

```bash
pdfimages -list /tmp/spike.pdf
pdffonts /tmp/spike.pdf
pdftotext /tmp/spike.pdf - | head
```

Expected for a vector image: no full-page image object for the diagram, the
label text extractable, or at least a vector path with no large raster. If the
diagram appears as a single large image, **stop**: the plan is not viable and
the finding is reported instead of building the surface.

- [x] **Step 4: Revert the temporary edit**

`git checkout -- ui/js/screens/graph.js` and confirm the tree is clean.

---

### Task 2: Print colour mapping

**Files:**
- Modify: `ui/js/graph/print-palette.js`
- Modify: `ui/js/graph/print-palette.test.mjs`

**Interfaces:**
- Produces: `printColour(value: string, backdrop?: string) -> string` — accepts `#rrggbb`, `rgb()`, `rgba()`; flattens alpha at or above 0.5 over the backdrop and returns `rgb()`, keeps lower alpha as `rgba()`.
- `printify(data)` keeps its current behaviour and uses the same channel math.

- [x] **Step 1: Write the failing tests**

Add to `print-palette.test.mjs`:

```js
import { printColour } from "./print-palette.js";

test("printColour matches the pixel filter for hex and rgb", () => {
  assert.equal(printColour("#141922"), "rgb(244, 249, 255)");
  assert.equal(printColour("rgb(255, 183, 3)"), "rgb(150, 72, 0)");
});

test("printColour flattens high alpha and keeps low alpha", () => {
  assert.match(printColour("rgba(92, 179, 255, 0.5)"), /^rgb\(/);
  assert.match(printColour("rgba(92, 179, 255, 0.1)"), /^rgba\(/);
});
```

- [x] **Step 2: Run and confirm failure**

Run: `node --test ui/js/graph/print-palette.test.mjs`  
Expected: FAIL — `printColour` is not exported.

- [x] **Step 3: Implement `printColour`**

Parse the three supported forms, invert each channel, apply the existing
`ROTATION` and `BRIGHTNESS`, and return `rgb(r, g, b)` or `rgba(r, g, b, a)`.
`printify` should call the same channel function so the two cannot drift.

- [x] **Step 4: Run the tests**

Run: `node --test ui/js/graph/print-palette.test.mjs`  
Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add ui/js/graph/print-palette.js ui/js/graph/print-palette.test.mjs
git commit -m "feat(graph): expose the print colour mapping"
```

---

### Task 3: SVG surface

**Files:**
- Create: `ui/js/graph/svg-surface.js`
- Create: `ui/js/graph/svg-surface.test.mjs`

**Interfaces:**
- Produces: `createSvgSurface({ width, height, measure }) -> surface`
  - properties: `fillStyle`, `strokeStyle`, `lineWidth`, `lineCap`, `font`, `textAlign`, `textBaseline`
  - methods used by `graph.js`: `beginPath`, `moveTo`, `lineTo`, `arc`, `arcTo`, `closePath`, `fill`, `stroke`, `fillRect`, `fillText`, `setLineDash`, `setTransform`, `measureText`
  - `surface.toString() -> string` returns a complete `<svg>` element.
- `measure` is a real 2D context from a throwaway canvas; `measureText` delegates to it.

- [x] **Step 1: Write the failing surface tests**

`svg-surface.test.mjs` covers:

- a filled and stroked path emits `<path>` with `d`, `fill` and `stroke`;
- `fillRect` emits a `<rect>` in print colours;
- `fillText` emits escaped text with the right `text-anchor` and baseline;
- `measureText` returns the delegated width;
- `setLineDash` adds `stroke-dasharray`;
- the output contains no unescaped `<`, `&` or `"` from label text.

- [x] **Step 2: Run and confirm failure**

Run: `node --test ui/js/graph/svg-surface.test.mjs`  
Expected: FAIL — module missing.

- [x] **Step 3: Implement the surface**

Emit a 24×24-independent SVG with `viewBox="0 0 width height"` and
`width={width*scale}` / `height={height*scale}`. Build path data from the
primitives; emit one `<path>` per `fill()`/`stroke()` call with the current
style, converted by `printColour`. Map `textAlign` to `text-anchor` and
`textBaseline` to `dominant-baseline`. Ignore `setTransform` scale for geometry
(the viewBox already carries the size). Escape text with the existing
`escapeHtml` helper or a local XML escape.

- [x] **Step 4: Run the tests**

Run: `node --test ui/js/graph/svg-surface.test.mjs`  
Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add ui/js/graph/svg-surface.js ui/js/graph/svg-surface.test.mjs
git commit -m "feat(graph): add an SVG surface for the printed diagram"
```

---

### Task 4: Wire SVG with PNG fallback and cache format

**Files:**
- Modify: `ui/js/screens/graph.js`
- Modify: `ui/js/graph/capture-cache.js` (key format)
- Modify: `ui/js/graph/capture-cache.test.mjs`
- Modify: `ui/js/screens/report.js` (request `format: "svg"`)

**Interfaces:**
- `captureWorkingDiagram({ scale, format })`; `format` is `"svg" | "png"`.
- `renderDiagram(width, height, scale, format)` returns a data URL.
- `captureKey(state, width, height, scale, format)` includes the format.

- [x] **Step 1: Extend the cache-key test**

Add a case proving `captureKey(..., "svg") !== captureKey(..., "png")` and that
the existing state/size/scale assertions still hold.

- [x] **Step 2: Implement `renderDiagramSvg` and fallback**

`renderDiagram` asks for SVG when requested. It saves/restores `box`, `view`,
`showDims` and `labelScale` as today, calls `paint(surface)`, returns a UTF-8
`data:image/svg+xml,` URL. On any error, log and return the PNG path. The cache
stores by the key including format.

- [x] **Step 3: Request SVG from the report**

In `report.js`, `captureWorkingDiagram({ scale, format: "svg" })`. The existing
diagram checkbox and scale control stay unchanged.

- [x] **Step 4: Run the pure tests**

Run: `node --test ui/js/graph/capture-cache.test.mjs ui/js/graph/svg-surface.test.mjs ui/js/graph/print-palette.test.mjs`  
Expected: PASS.

- [x] **Step 5: Commit**

```bash
git add ui/js/screens/graph.js ui/js/screens/report.js ui/js/graph/capture-cache.js ui/js/graph/capture-cache.test.mjs
git commit -m "feat(report): print the diagram as svg with a png fallback"
```

---

### Task 5: Verify the PDF and review

**Files:**
- No production changes unless verification fails.

- [ ] **Step 1: Visual and PDF checks**

Export a plan. Verify:

- the diagram matches the 2.17.18 export at 100% and zoomed in;
- `pdfimages -list` shows no full-page diagram raster;
- the PDF is no larger than 2.17.18 (normally smaller);
- the live frame and Pages view both show the diagram.

- [x] **Step 2: Force the fallback**

Temporarily make `renderDiagramSvg` throw; export again. Expected: a report
with the PNG diagram, no error banner.

**Recorded:** `diagram svg failed: forced fallback` in `ui.log`, then a 205 ms
PNG capture; the page view still carried the diagram.

- [x] **Step 3: Run the gates**

Run the CI frontend suite, `cargo test --all-targets`, clippy and the syntax loop.

- [x] **Step 4: Three-seat review**

Safety & QA (diagram content, fallback, PDF), Security & Build (no new surface,
data URL, cache), Design & Steward (surface boundary, tests, honesty).

**Result:** Safety & QA 8.5, Security & Build 8.5, Design & Steward 8.5. The
grid/colour, placeholder, font and record blockers were fixed before landing.
The PDF-size comparison in Step 1 is the only open item; no size claim ships.

---

## Self-Review

- **Spec coverage:** spike (Task 1), colours (Task 2), surface (Task 3), wiring
  and fallback (Task 4), PDF/review (Task 5).
- **Step scan:** the spike has a stop condition; each pure module has tests;
  the fallback is tested by forcing it.
- **Type consistency:** `printColour`, `createSvgSurface`, `format`, `captureKey`.
- **Review Focus:** vector proof, text metrics, colour tolerance, fallback,
  cache format.
- **Proportion:** the plan stops before the surface if the spike fails.
