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
  const positionLine = el("div", { class: "faint small", style: { margin: "10px 0 0" } });
  let tableOpen = true;
  const workbookSection = el(
    "div",
    { class: "mt16" },
    el("div", { class: "fields" }, boomWrap, radiusWrap),
    workbookBox,
    positionLine,
  );

  // --- jib configuration (the workbook's jib sheet) ------------------------
  // Shown only for a workbook that carries jib tables. The configuration picks
  // its jib (the plain jib is the workbook's shortest table, the extended jib
  // the longest) and the operator chooses one of the offset angles that jib
  // allows. The selection lives in the shared state and both this card and the
  // Graph tab's configuration radios go through `store.chooseJib`.
  const JIB_CONFIGS = [
    ["boom", "Boom length"],
    ["boom_jib", "Boom length + jib"],
    ["boom_ext_jib", "Boom length + extended jib"],
  ];
  const jibConfigBox = el("div", { class: "checks" });
  const jibLengthLine = el("div", { class: "faint small" });
  const jibBoomAngleSelect = selectInput({
    options: [],
    value: null,
    onChange: (value) => store.setBoomAngle(Number(value)),
  });
  const jibOffsetSelect = selectInput({
    options: [],
    value: null,
    onChange: (value) => store.chooseJib({ offset: Number(value) }),
  });
  const jibFields = el(
    "div",
    { class: "stack-8" },
    jibLengthLine,
    el(
      "div",
      { class: "fields" },
      el("div", { class: "field" }, el("label", { text: "Boom angle" }), jibBoomAngleSelect),
      el("div", { class: "field" }, el("label", { text: "Jib offset angle" }), jibOffsetSelect),
    ),
  );
  const jibCard = card({
    title: "Jib configuration",
    note: "From the workbook's jib sheet",
    body: el("div", { class: "stack-8" }, jibConfigBox, jibFields),
  });

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
        jibCard,
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
                  chart_position: null,
                  chart_jibs: [],
                  jib_config: "boom",
                  jib_length: null,
                  jib_offset: null,
                  chart_jib_points: [],
                  boom_angle_deg: null,
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
      // With a jib selected the radius follows from the boom angle (the jib
      // sheet's own basis), so the radius picker has nothing to drive.
      const jibActive =
        state.crane.jib_config === "boom_jib" || state.crane.jib_config === "boom_ext_jib";
      radiusWrap.style.display = radii.length && !jibActive ? "" : "none";
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
      renderWorkbookTable(table);
      const position = state.crane.chart_position;
      positionLine.textContent = position
        ? `Crane location on axis (X, Y): (${position[0]}, ${position[1]}) — from the workbook`
        : "";
      syncJib(state);
    } else {
      positionLine.textContent = "";
      jibCard.style.display = "none";
      workbookBox.replaceChildren();
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

  // The jib controls, populated from the workbook's jib tables.
  function syncJib(state) {
    const jibs = state.crane.chart_jibs || [];
    jibCard.style.display = jibs.length ? "" : "none";
    if (!jibs.length) return;

    const config = state.crane.jib_config || "boom";
    jibConfigBox.replaceChildren(
      ...JIB_CONFIGS.map(([key, label]) => {
        const radio = el("input", { type: "radio", name: "crane-jib-config" });
        radio.checked = key === config;
        radio.addEventListener("change", () => store.chooseJib({ config: key }));
        return el("label", { class: "check" }, radio, el("span", { class: "t", text: label }));
      }),
    );

    const active = config !== "boom";
    jibFields.style.display = active ? "" : "none";
    if (!active) return;

    // The configuration determines the jib: the plain jib is the workbook's
    // shortest table, the extended jib its longest. Only the offset angles
    // that jib allows remain to choose.
    jibLengthLine.textContent = `Jib ${fmt(state.crane.jib_length ?? 0, 2, "m")} — set by the configuration`;

    // The boom angle, listed from the selected jib chart's own rows: it is the
    // jib chart's input, the way the working radius is the main boom's.
    const angles = (state.crane.chart_jib_points || [])
      .map((point) => point[0])
      .sort((a, b) => a - b);
    jibBoomAngleSelect.replaceChildren(
      ...angles.map((value) =>
        el("option", {
          value: String(value),
          text: Number.isInteger(value) ? fmt(value, 0, "°") : fmt(value, 1, "°"),
        }),
      ),
    );
    const effectiveAngle =
      state.crane.boom_angle_deg ?? store.solved?.crane?.result?.boom_angle_deg ?? null;
    if (effectiveAngle !== null && effectiveAngle !== undefined && angles.length) {
      jibBoomAngleSelect.value = String(nearest(angles, effectiveAngle));
    }

    const offsets = jibs
      .filter((jib) => Math.abs(jib.length - (state.crane.jib_length ?? 0)) < 1e-9)
      .map((jib) => jib.offset)
      .sort((a, b) => a - b);
    jibOffsetSelect.replaceChildren(
      ...offsets.map((value) => el("option", { value: String(value), text: fmt(value, 0, "°") })),
    );
    if (state.crane.jib_offset !== null && state.crane.jib_offset !== undefined) {
      jibOffsetSelect.value = String(nearest(offsets, state.crane.jib_offset));
    }
  }

  // The workbook table under the chart controls, foldable: the toggle header
  // stays put, the table body comes and goes. The open state lasts while the
  // tab lives, like every other form state.
  function renderWorkbookTable(table) {
    if (!table || !table.headers?.length) {
      workbookBox.replaceChildren();
      return;
    }
    const toggle = el(
      "button",
      {
        class: "btn ghost small",
        style: { margin: "10px 0 6px" },
        title: tableOpen ? "Collapse the workbook table" : "Expand the workbook table",
        onClick: () => {
          tableOpen = !tableOpen;
          renderWorkbookTable(table);
        },
      },
      tableOpen ? "\u25be Workbook table" : "\u25b8 Workbook table",
    );
    const nodes = [el("div", {}, toggle)];
    if (tableOpen) {
      nodes.push(
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
    }
    workbookBox.replaceChildren(...nodes);
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
    const craneState = store.state.crane;
    const jibActive =
      craneState.chart_source === "excel" &&
      (craneState.jib_config === "boom_jib" || craneState.jib_config === "boom_ext_jib");

    // The angle produced for the working radius (by the graph, or implied by
    // the assembly geometry) is mirrored into the manual row so the entry, the
    // capacity check and the report all read the same figure. Only the
    // new-format workbook — it carries the crane position — does this. The
    // mirror carries the printed resolution, not the raw float.
    const mirroredAngle = Math.round(result.boom_angle_deg * 100) / 100;
    if (
      craneState.chart_position &&
      mirroredAngle &&
      Math.abs((craneState.rows[0]?.angle ?? 0) - mirroredAngle) > 1e-9
    ) {
      store.update((draft) => {
        if (draft.crane.rows[0]) draft.crane.rows[0].angle = mirroredAngle;
      });
    }

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
        jibActive
          ? ["Jib", `${fmt(craneState.jib_length ?? 0, 2, "m")} at ${fmt(craneState.jib_offset ?? 0, 0, "°")} offset`]
          : null,
        ["Working radius", result.radius_m ? fmt(result.radius_m, 2, "m") : "—"],
        [
          jibActive ? "Jib capacity at boom angle" : "Chart capacity at radius",
          result.chart_capacity_kg ? fmt(result.chart_capacity_kg, 2, "kg") : "—",
        ],
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

    // The capacity-vs-radius picture belongs to the main boom chart; with a
    // jib configured the checked figure comes from the jib table at a boom
    // angle, so the boom chart's marker would name the wrong thing.
    plotBox.replaceChildren(jibActive ? el("div", {}) : plot(solved, result));
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
