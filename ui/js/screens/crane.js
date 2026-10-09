// Crane: identity, load chart (workbook or one hand-typed point), and the
// capacity check at the working radius.

import { el, svgIcon } from "../dom.js";
import {
  card,
  clearButton,
  exampleButton,
  field,
  kvRows,
  meter,
  numberInput,
  reportButton,
  selectInput,
  statusBadge,
  textInput,
  withUnit,
} from "../components.js";
import { fmt } from "../format.js";

export function createCraneScreen(ctx) {
  const { store, catalog } = ctx;

  // --- crane identity ------------------------------------------------------
  const nameInput = textInput({
    value: "",
    placeholder: "e.g. Kobelco CKE90G-2",
    onInput: (value) => store.update((s) => (s.crane.name = value)),
  });
  const capacityInput = numberInput({
    value: 0,
    onInput: (value) => store.update((s) => (s.crane.capacity_t = value ?? 0)),
  });

  // --- workbook ------------------------------------------------------------
  const chartLabel = el("span", { class: "muted small" });
  const openButton = el(
    "button",
    {
      class: "btn primary",
      onClick: async () => {
        const label = await store.importChartFromFile();
        if (label) ctx.toast(`Chart loaded — ${label}`, "ok");
      },
    },
    svgIcon("file", 15),
    "Open Excel load chart...",
  );
  const boomWrap = el("div", { class: "field" });
  const radiusWrap = el("div", { class: "field" });
  const workbookBox = el("div", {});
  const workbookSection = el(
    "div",
    { class: "mt16" },
    el("div", { class: "fields" }, boomWrap, radiusWrap),
    workbookBox,
  );

  const boomSelect = selectInput({ options: [], value: null, onChange: (value) => store.chartAction({ action: "boom", boom: Number(value), gross: currentGross() }) });
  const radiusSelect = selectInput({ options: [], value: null, onChange: (value) => store.chartAction({ action: "radius", radius: Number(value) }) });
  boomWrap.append(el("label", { text: "Boom length" }), boomSelect);
  radiusWrap.append(el("label", { text: "Working radius" }), radiusSelect);

  // --- manual entry --------------------------------------------------------
  const manual = {};
  const manualField = (key, label, unit) =>
    field({
      label,
      unit,
      input: numberInput({
        value: 0,
        onInput: (value) => store.manualChartEdit({ [key]: value ?? 0 }),
      }),
    });
  manual.radius = manualField("radius", "Radius", "m");
  manual.capacity = manualField("capacity", "Rated load", "kg");
  manual.angle = manualField("angle", "Boom angle", "deg");
  manual.boom = manualField("boom", "Boom length", "m");

  // --- capacity check ------------------------------------------------------
  const checkBadge = el("div", {});
  const checkMeter = el("div", {});
  const checkRows = el("div", {});
  const plotBox = el("div", { class: "mt12" });

  const root = el(
    "div",
    { class: "page" },
    el(
      "div",
      { class: "grid-main" },
      el(
        "div",
        { class: "stack pane" },
        card({
          title: "Crane",
          note: "As booked for this lift",
          body: el("div", { class: "fields" }, field({ label: "Crane", input: nameInput }), field({ label: "Crane rated capacity", unit: "ton", input: capacityInput })),
        }),
        card({
          title: "Load chart — Excel workbook",
          note: "Reads .xlsx files",
          body: el(
            "div",
            {},
            el("div", { class: "row wrap" }, openButton, chartLabel),
            workbookSection,
            el("p", {
              class: "faint small",
              style: { margin: "12px 0 0" },
              text:
                "Matrix charts (boom lengths across the top) are read at the selected boom; " +
                "the working radius is suggested as the farthest charted radius that still " +
                "meets the 75% rule.",
            }),
          ),
        }),
        card({
          title: "Load chart — manual entry",
          note: "Typing here switches the chart back to this line",
          body: el("div", { class: "fields fields-4" }, manual.radius, manual.capacity, manual.angle, manual.boom),
        }),
      ),
      el(
        "div",
        { class: "stack sticky pane" },
        card({ title: "Capacity check", body: el("div", {}, checkBadge, checkMeter, checkRows, plotBox) }),
        card({
          title: "Actions",
          body: el(
            "div",
            { class: "row wrap" },
            exampleButton(() =>
              store.update((s) => {
                s.crane = { ...catalog.example_crane, rows: catalog.example_crane.rows.map((row) => ({ ...row })), chart_points: [] };
                s.chart_table = null;
              }),
            ),
            reportButton("Show report", () => ctx.showReport(["crane"])),
            clearButton(() =>
              store.update((s) => {
                s.crane = {
                  name: "",
                  capacity_t: 0,
                  rows: [{ radius: 0, capacity: 0, angle: 0, boom: 0 }],
                  chart_points: [],
                  working_radius: 0,
                  chart_source: "",
                  chart_name: "",
                  chart_label: "",
                  chart_booms: [],
                  chart_boom: null,
                  chart_radii: [],
                };
                s.chart_table = null;
              }),
            ),
          ),
          foot: el("span", { class: "faint", text: "Clear puts the crane, the radius and the chart back to blank." }),
        }),
      ),
    ),
  );

  function currentGross() {
    return store.solved?.totals?.gross ?? 0;
  }

  function sync(state) {
    if (document.activeElement !== nameInput) nameInput.value = state.crane.name;
    if (document.activeElement !== capacityInput) capacityInput.value = String(state.crane.capacity_t ?? 0);
    chartLabel.textContent = state.crane.chart_label || "No workbook loaded.";

    const isExcel = state.crane.chart_source === "excel";
    workbookSection.style.display = isExcel ? "" : "none";
    if (isExcel) {
      const booms = state.crane.chart_booms || [];
      const radii = state.crane.chart_radii || [];
      boomWrap.style.display = booms.length > 1 ? "" : "none";
      radiusWrap.style.display = radii.length ? "" : "none";
      boomSelect.replaceChildren(
        ...booms.map((boom) => el("option", { value: String(boom), text: fmt(boom, 2, "m") })),
      );
      const selectedBoom = state.crane.chart_boom ?? booms[booms.length - 1] ?? null;
      if (selectedBoom !== null) boomSelect.value = String(nearest(booms, selectedBoom));
      radiusSelect.replaceChildren(
        ...radii.map((radius) => el("option", { value: String(radius), text: fmt(radius, 2, "m") })),
      );
      if (radii.length) radiusSelect.value = String(nearest(radii, state.crane.working_radius));

      const table = state.chart_table;
      if (table && table.headers?.length) {
        workbookBox.replaceChildren(
          el("div", { class: "faint small", style: { margin: "10px 0 6px" }, text: "Workbook table" }),
          el(
            "div",
            { class: "table-wrap" },
            el(
              "table",
              { class: "grid" },
              el("thead", {}, el("tr", {}, ...table.headers.map((header) => el("th", { text: header })))),
              el(
                "tbody",
                {},
                ...table.rows.map((row) =>
                  el("tr", {}, ...row.map((cell, index) => el("td", { class: index ? "num" : "", text: cell }))),
                ),
              ),
            ),
          ),
        );
      } else {
        workbookBox.replaceChildren();
      }
    }

    const row = state.crane.rows[0] || { radius: 0, capacity: 0, angle: 0, boom: 0 };
    const setValue = (input, value) => {
      if (document.activeElement !== input) input.value = String(value ?? 0);
    };
    setValue(manual.radius.querySelector("input"), row.radius);
    setValue(manual.capacity.querySelector("input"), row.capacity);
    setValue(manual.angle.querySelector("input"), row.angle);
    setValue(manual.boom.querySelector("input"), row.boom);
  }

  function nearest(values, wanted) {
    let best = values[0];
    let bestDiff = Infinity;
    for (const value of values) {
      const diff = Math.abs(value - wanted);
      if (diff < bestDiff) {
        best = value;
        bestDiff = diff;
      }
    }
    return best;
  }

  function render(solved) {
    if (!solved) return;
    const section = solved.crane;
    const gross = solved.totals.gross;

    if (!section.ok) {
      checkBadge.replaceChildren(statusBadge("over", "Could not check", section.error || ""));
      checkMeter.replaceChildren();
      checkRows.replaceChildren();
      plotBox.replaceChildren();
      return;
    }

    const result = section.result;
    const band = result.capacity_band || "";
    const usage = result.capacity_usage_percent;

    checkBadge.replaceChildren(
      statusBadge(
        band,
        band ? bandWord(band) : "No capacity found",
        result.capacity_status,
      ),
    );
    checkMeter.replaceChildren(usage === null || usage === undefined ? el("div", {}) : meter(usage, band));

    checkRows.replaceChildren(
      kvRows([
        ["Crane", result.crane_name || "—"],
        ["Crane rated capacity", result.crane_capacity_kg ? fmt(result.crane_capacity_kg, 2, "kg") : "—"],
        ["Boom length", result.boom_m ? fmt(result.boom_m, 2, "m") : "—"],
        ["Boom angle", result.boom_angle_deg ? fmt(result.boom_angle_deg, 2, "°") : "—"],
        ["Working radius", result.radius_m ? fmt(result.radius_m, 2, "m") : "—"],
        ["Chart capacity at radius", result.chart_capacity_kg ? fmt(result.chart_capacity_kg, 2, "kg") : "—"],
        ["Gross load at hook", fmt(gross, 2, "kg")],
        [
          "Capacity usage",
          usage === null || usage === undefined ? "—" : fmt(usage, 2, "%"),
          band ? bandClass(band) : "",
          "big",
        ],
        ["Chart source", result.capacity_source || "—"],
      ]),
    );

    plotBox.replaceChildren(plot(solved, result));
  }

  // The capacity-vs-radius picture: the solved chart, with the working radius
  // marked. Presentation only — every figure on it is solved in Rust.
  function plot(solved, result) {
    const points = [];
    const seen = new Set();
    for (const point of store.state.crane.chart_points || []) {
      if (point[0] > 0 && point[1] > 0 && !seen.has(point[0])) {
        seen.add(point[0]);
        points.push(point);
      }
    }
    const row = store.state.crane.rows[0];
    if (row && row.radius > 0 && row.capacity > 0 && !seen.has(row.radius)) {
      points.push([row.radius, row.capacity]);
    }
    points.sort((a, b) => a[0] - b[0]);
    if (points.length < 2) return el("div", {});

    const width = 480;
    const height = 220;
    const m = { l: 46, r: 16, t: 14, b: 30 };
    const markerRadius = result.radius_m;
    const markerCapacity = result.chart_capacity_kg;

    const xs = points.map((point) => point[0]);
    const ys = points.map((point) => point[1]);
    if (markerRadius > 0 && markerCapacity) {
      xs.push(markerRadius);
      ys.push(markerCapacity);
    }
    const xMax = niceMax(Math.max(...xs));
    const yMax = niceMax(Math.max(...ys));
    const X = (radius) => m.l + (radius / xMax) * (width - m.l - m.r);
    const Y = (capacity) => height - m.b - (capacity / yMax) * (height - m.t - m.b);

    let grid = "";
    for (let i = 1; i <= 4; i += 1) {
      const gx = m.l + (i / 4) * (width - m.l - m.r);
      const gy = height - m.b - (i / 4) * (height - m.t - m.b);
      grid += `<line x1="${gx}" y1="${m.t}" x2="${gx}" y2="${height - m.b}" stroke="#243040" stroke-width="1"/>`;
      grid += `<line x1="${m.l}" y1="${gy}" x2="${width - m.r}" y2="${gy}" stroke="#243040" stroke-width="1"/>`;
      grid += `<text x="${gx}" y="${height - m.b + 14}" fill="#64748d" font-size="10" text-anchor="middle">${((i / 4) * xMax).toFixed(0)}</text>`;
      grid += `<text x="${m.l - 8}" y="${gy + 3}" fill="#64748d" font-size="10" text-anchor="end">${(((i / 4) * yMax) / 1000).toFixed(0)}</text>`;
    }

    const line = points.map((point) => `${X(point[0])},${Y(point[1])}`).join(" ");
    const dots = points
      .map((point) => `<circle cx="${X(point[0])}" cy="${Y(point[1])}" r="2.6" fill="#5cb3ff"/>`)
      .join("");
    let marker = "";
    if (markerRadius > 0 && markerCapacity) {
      const mx = X(markerRadius);
      const my = Y(markerCapacity);
      marker =
        `<line x1="${mx}" y1="${m.t}" x2="${mx}" y2="${height - m.b}" stroke="#e2a558" stroke-width="1" stroke-dasharray="4 3"/>` +
        `<circle cx="${mx}" cy="${my}" r="5" fill="none" stroke="#e2a558" stroke-width="2"/>` +
        `<text x="${mx + 8}" y="${my - 8}" fill="#e2a558" font-size="10.5">${fmt(markerCapacity, 0, "kg")} at ${fmt(markerRadius, 2, "m")}</text>`;
    }

    const svg = `<svg viewBox="0 0 ${width} ${height}" width="100%" height="${height}" role="img" aria-label="Chart capacity against working radius">
      ${grid}
      <polyline points="${line}" fill="none" stroke="#5cb3ff" stroke-width="2"/>
      ${dots}${marker}
      <text x="${width - m.r}" y="${height - 6}" fill="#64748d" font-size="10" text-anchor="end">radius (m)</text>
      <text x="6" y="${m.t}" fill="#64748d" font-size="10">rated load (t)</text>
    </svg>`;
    return el("div", { html: svg });
  }

  function niceMax(value) {
    if (value <= 0) return 1;
    const pow = Math.pow(10, Math.floor(Math.log10(value)));
    const scaled = value / pow;
    const step = scaled <= 2 ? 2 : scaled <= 5 ? 5 : 10;
    return step * pow;
  }

  function bandWord(band) {
    return band === "within" ? "Within limit" : band === "caution" ? "Caution" : "Warning";
  }

  function bandClass(band) {
    return band === "within" ? "ok" : band === "caution" ? "caution" : "over";
  }

  return { root, sync, render };
}
