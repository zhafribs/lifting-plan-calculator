// Outrigger Pad Size: the bearing area under one outrigger, sized from the
// BS 8004 presumed allowable bearing values in the ground table.

import { el } from "../dom.js";
import {
  card,
  clearButton,
  exampleButton,
  field,
  kvRows,
  meter,
  note,
  numberInput,
  reportButton,
  selectInput,
  statusBadge,
} from "../components.js";
import { fmt } from "../format.js";

export function createOutriggerScreen(ctx) {
  const { store, catalog } = ctx;

  const set = (patch) => store.update((s) => Object.assign(s.outrigger, patch));
  const groundFor = (state) =>
    catalog.ground_types.find((ground) => ground.key === state.outrigger.ground_key) || null;

  const categorySelect = selectInput({
    options: [
      { key: "", label: "Select a ground category" },
      ...catalog.ground_categories.map((category) => ({ key: category, label: category })),
    ],
    value: "",
    onChange: (category) => {
      const first = catalog.ground_types.find((ground) => ground.category === category);
      set({
        ground_key: first ? first.key : "",
        bearing_kpa: first && first.v_default ? first.v_default : 0,
      });
    },
  });

  // The description list is rebuilt when the category changes. A fixed-size
  // select would show every row at once and lose the category grouping.
  const groundSelect = el("select");
  groundSelect.addEventListener("change", () => {
    const ground = catalog.ground_types.find((item) => item.key === groundSelect.value);
    if (!ground) return;
    set({
      ground_key: ground.key,
      bearing_kpa: ground.v_default ? ground.v_default : 0,
    });
  });

  const groundMeta = el("div", { class: "ground-meta" });
  const vInput = numberInput({
    value: 0,
    onInput: (value) => set({ bearing_kpa: value ?? 0 }),
  });

  const vField = field({
    label: "Max permissible SBC (V)",
    unit: "kPa",
    input: vInput,
  });
  vField.classList.add("v-field");

  const cmInput = numberInput({
    value: 0,
    onInput: (value) => set({ crane_mass_kn: value ?? 0, crane_mass_linked: false }),
  });
  const liftInput = numberInput({
    value: 0,
    onInput: (value) => set({ lift_load_kn: value ?? 0, lift_load_linked: false }),
  });
  const overallHint = el("span", { class: "faint small" });
  const useCrane = el(
    "button",
    {
      class: "btn ghost small",
      onClick: () => {
        const capacity = store.state.crane?.capacity_t ?? 0;
        if (capacity <= 0) {
          ctx.toast("Enter the crane rated capacity in the Crane tab first.", "error");
          return;
        }
        set({
          crane_mass_kn: Math.round(capacity * 9.80665 * 100) / 100,
          crane_mass_linked: true,
        });
      },
    },
    "Use crane tab",
  );
  const useOverall = el(
    "button",
    {
      class: "btn ghost small",
      onClick: () => {
        const gross = store.solved?.totals?.gross ?? 0;
        if (gross <= 0) {
          ctx.toast("Add a load in Overall Weight first.", "error");
          return;
        }
        set({
          lift_load_kn: Math.round(gross * catalog.kg_to_kn * 100) / 100,
          lift_load_linked: true,
        });
      },
    },
    "Use overall weight",
  );

  const shapeSelect = selectInput({
    options: [
      { key: "square", label: "Square" },
      { key: "rectangular", label: "Rectangular" },
    ],
    value: "square",
    onChange: (value) => {
      if (value === "square") {
        const side = store.state.outrigger.pad_length_m || store.state.outrigger.pad_width_m || 0;
        set({ pad_shape: "square", pad_length_m: side, pad_width_m: side });
      } else {
        set({ pad_shape: "rectangular" });
      }
    },
  });

  const sideInput = numberInput({
    value: 0,
    onInput: (value) => set({ pad_length_m: value ?? 0, pad_width_m: value ?? 0 }),
  });
  const lengthInput = numberInput({
    value: 0,
    onInput: (value) => set({ pad_length_m: value ?? 0 }),
  });
  const widthInput = numberInput({
    value: 0,
    onInput: (value) => set({ pad_width_m: value ?? 0 }),
  });
  const squareField = field({ label: "Pad side", unit: "m", input: sideInput });
  const rectFields = el(
    "div",
    { class: "fields" },
    field({ label: "Pad length", unit: "m", input: lengthInput }),
    field({ label: "Pad width", unit: "m", input: widthInput }),
  );
  const useRecommendation = el(
    "button",
    {
      class: "btn ghost small",
      onClick: () => {
        const result = store.solved?.outrigger?.result;
        if (!result || !(result.recommended_square_rounded_m > 0)) {
          ctx.toast("Enter the ground and the loads first.", "error");
          return;
        }
        const side = result.recommended_square_rounded_m;
        set({ pad_shape: "square", pad_length_m: side, pad_width_m: side });
      },
    },
    "Use recommended square",
  );

  const badge = el("div", {});
  const resultRows = el("div", {});
  const formulaBox = el("div", { class: "formula-block" });
  const padDiagram = el("div", { class: "pad-diagram" });
  const groundReference = el("div", {});

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
          title: "Ground",
          note: "BS 8004 Table 1, Malaysian practice",
          body: el(
            "div",
            { class: "stack-8" },
            el(
              "div",
              { class: "fields" },
              field({ label: "Ground category", input: categorySelect }),
              field({ label: "Soil / rock description", input: groundSelect }),
            ),
            el(
              "div",
              { class: "fields mt12" },
              vField,
            ),
            groundMeta,
            note(
              "V is the presumed allowable bearing value. For a range, the lower bound is loaded for preliminary sizing; confirm a site-specific value before the lift.",
            ),
          ),
        }),
        card({
          title: "Crane and lift",
          note: "The two vertical loads in the formula",
          body: el(
            "div",
            { class: "stack-8" },
            el(
              "div",
              { class: "fields" },
              field({
                label: "Crane mass (CM)",
                unit: "kN",
                input: cmInput,
                hint: "Defaults to the Crane tab's rated capacity until edited.",
              }),
              field({
                label: "Lift load (L)",
                unit: "kN",
                input: liftInput,
                hint: "Defaults to the Overall Weight gross load until edited.",
              }),
            ),
            el(
              "div",
              { class: "row wrap" },
              useCrane,
              useOverall,
              overallHint,
            ),
          ),
        }),
        card({
          title: "Pad",
          note: "Bearing area only; thickness is a separate check",
          body: el(
            "div",
            { class: "stack-8" },
            el("div", { class: "fields" }, field({ label: "Pad shape", input: shapeSelect })),
            squareField,
            rectFields,
            el(
              "div",
              { class: "row wrap" },
              useRecommendation,
              el("span", {
                class: "faint small",
                text: "Sizes are rounded up to 50 mm increments.",
              }),
            ),
            note(
              "This sizes the pad's bearing area. Pad thickness, steel plate design, timber grade and the crane manufacturer's outrigger reaction are separate checks.",
            ),
          ),
        }),
      ),
      el(
        "div",
        { class: "stack sticky pane" },
        card({ title: "Pad size result", body: el("div", {}, badge, resultRows, formulaBox, padDiagram) }),
        card({ title: "Ground reference", body: groundReference }),
        card({
          title: "Actions",
          body: el(
            "div",
            { class: "row wrap" },
            exampleButton(() =>
              store.update((s) => {
                s.outrigger = {
                  ...catalog.example_outrigger,
                  crane_mass_linked: false,
                  lift_load_linked: false,
                };
              }),
            ),
            reportButton("Show report", () => ctx.showReport(["crane", "outrigger"])),
            clearButton(() =>
              store.update((s) => {
                s.outrigger = {
                  ground_key: "",
                  bearing_kpa: 0,
                  crane_mass_kn: 0,
                  lift_load_kn: 0,
                  crane_mass_linked: true,
                  lift_load_linked: true,
                  pad_shape: "square",
                  pad_length_m: 0,
                  pad_width_m: 0,
                };
              }),
            ),
          ),
          foot: el("span", {
            class: "faint",
            text: "Clear returns the tab to a blank ground and pad.",
          }),
        }),
      ),
    ),
  );

  function fillGroundOptions(state) {
    const ground = groundFor(state);
    const category = ground ? ground.category : "";
    if (document.activeElement !== categorySelect) categorySelect.value = category;

    const options = category
      ? catalog.ground_types.filter((item) => item.category === category)
      : [];
    const signature = `${category}|${options.map((item) => item.key).join(",")}`;
    if (groundSelect.dataset.signature !== signature) {
      groundSelect.dataset.signature = signature;
      groundSelect.replaceChildren(
        el("option", {
          value: "",
          text: category ? "Select a ground description" : "Choose a category first",
        }),
        ...options.map((item) => el("option", { value: item.key, text: item.description })),
      );
    }
    groundSelect.disabled = !category;
    if (document.activeElement !== groundSelect) groundSelect.value = ground ? ground.key : "";
  }

  function renderGroundMeta(state) {
    const ground = groundFor(state);
    if (!ground) {
      groundMeta.replaceChildren(
        el("p", {
          class: "faint small",
          style: { margin: 0 },
          text: "Choose a category and description to load the published value and its field note.",
        }),
      );
      return;
    }
    const items = [
      el("div", {
        class: "ground-value",
        text: `Table value: ${ground.value_label}${ground.suitable ? " kPa" : ""}`,
      }),
      el("p", { class: "ground-note", text: ground.notes }),
    ];
    if (!ground.suitable) {
      items.push(
        el("p", {
          class: "ground-warning",
          text: "Not suitable for shallow foundations. Use piling or ground improvement, or a site-specific assessment.",
        }),
      );
    } else if (ground.v_default === null) {
      items.push(
        el("p", {
          class: "ground-warning",
          text: `Enter a site-specific value below ${fmt(ground.v_max, 0, "kPa")}. The table publishes a less-than value.`,
        }),
      );
    }
    groundMeta.replaceChildren(...items);
  }

  function updateGroundReference(state, solved) {
    const ground = groundFor(state);
    if (!ground) {
      groundReference.replaceChildren(
        el("p", { class: "faint small", text: "No ground reference selected." }),
      );
      return;
    }
    const rows = [
      ["Category", ground.category],
      ["Description", ground.description],
      ["Table value", ground.suitable ? `${ground.value_label} kPa` : "Not applicable"],
      ["Field note", ground.notes],
    ];
    const result = solved?.outrigger?.result;
    if (result) {
      const min = ground.v_min;
      const max = ground.v_max;
      const outside =
        (min !== null && result.bearing_kpa < min) ||
        (max !== null && result.bearing_kpa > max);
      if (outside) {
        rows.push([
          "Design V",
          fmt(result.bearing_kpa, 2, "kPa"),
          "caution",
          "one-line",
        ]);
      }
    }
    groundReference.replaceChildren(kvRows(rows));
    if (!ground.suitable) {
      groundReference.append(
        el("p", {
          class: "error-text",
          style: { margin: "10px 2px 0" },
          text: "This ground is not suitable for a shallow pad. The calculation is intentionally refused.",
        }),
      );
    }
  }

  function sync(state) {
    fillGroundOptions(state);
    renderGroundMeta(state);
    if (document.activeElement !== vInput) vInput.value = String(state.outrigger.bearing_kpa ?? 0);
    if (document.activeElement !== cmInput) cmInput.value = String(state.outrigger.crane_mass_kn ?? 0);
    if (document.activeElement !== liftInput) liftInput.value = String(state.outrigger.lift_load_kn ?? 0);
    if (document.activeElement !== shapeSelect) shapeSelect.value = state.outrigger.pad_shape || "square";
    if (document.activeElement !== sideInput) sideInput.value = String(state.outrigger.pad_length_m ?? 0);
    if (document.activeElement !== lengthInput) lengthInput.value = String(state.outrigger.pad_length_m ?? 0);
    if (document.activeElement !== widthInput) widthInput.value = String(state.outrigger.pad_width_m ?? 0);

    const square = (state.outrigger.pad_shape || "square") === "square";
    squareField.style.display = square ? "" : "none";
    rectFields.style.display = square ? "none" : "";

    const gross = store.solved?.totals?.gross ?? 0;
    overallHint.textContent = gross > 0
      ? `Overall weight: ${fmt(gross, 2, "kg")} (${fmt(gross * catalog.kg_to_kn, 2, "kN")}).`
      : "No overall weight entered yet.";
    useOverall.disabled = gross <= 0;
  }

  function bandClass(band) {
    return band === "ok" ? "ok" : band === "caution" ? "caution" : "over";
  }

  function padPlan(result) {
    if (!result.pad_length_m || !result.pad_width_m) {
      return el("p", {
        class: "faint small",
        style: { margin: "10px 2px 0" },
        text: "Enter a pad size to draw the plan.",
      });
    }
    const width = 340;
    const height = 214;
    const maxW = 220;
    const maxH = 96;
    const ratio = result.pad_length_m / result.pad_width_m;
    let padW;
    let padH;
    if (ratio >= 1) {
      padW = Math.min(maxW, maxH * ratio);
      padH = padW / ratio;
    } else {
      padH = Math.min(maxH, maxW / ratio);
      padW = padH * ratio;
    }
    const x = (width - padW) / 2;
    const y = 58;
    const dimY = y + padH + 20;
    const dimX = x + padW + 20;
    const svg = `<svg viewBox="0 0 ${width} ${height}" width="100%" height="${height}" role="img" aria-label="Outrigger pad plan">
      <rect x="${x}" y="${y}" width="${padW}" height="${padH}" rx="6" fill="#1c2838" stroke="#5cb3ff" stroke-width="1.5"/>
      <line x1="${x}" y1="${dimY}" x2="${x + padW}" y2="${dimY}" stroke="#e2a558" stroke-width="1" stroke-dasharray="4 3"/>
      <text x="${x + padW / 2}" y="${dimY + 14}" fill="#e2a558" font-size="10.5" text-anchor="middle">${fmt(result.pad_length_m, 2, "m")}</text>
      <line x1="${dimX}" y1="${y}" x2="${dimX}" y2="${y + padH}" stroke="#e2a558" stroke-width="1" stroke-dasharray="4 3"/>
      <text x="${dimX + 6}" y="${y + padH / 2}" fill="#e2a558" font-size="10.5" dominant-baseline="middle">${fmt(result.pad_width_m, 2, "m")}</text>
      <circle cx="${x + padW / 2}" cy="${y + padH / 2}" r="4" fill="#e2a558"/>
      <text x="${x + padW / 2}" y="${y + padH / 2 - 10}" fill="#e2a558" font-size="9.5" text-anchor="middle">outrigger</text>
      <text x="${width / 2}" y="${height - 6}" fill="#64748d" font-size="9.5" text-anchor="middle">Plan view, not to scale</text>
    </svg>`;
    return el("div", { html: svg });
  }

  function render(solved) {
    if (!solved) return;
    updateGroundReference(store.state, solved);
    const section = solved.outrigger;
    formulaBox.replaceChildren();
    resultRows.replaceChildren();
    padDiagram.replaceChildren();

    if (!section.ok) {
      badge.replaceChildren(statusBadge("over", "Could not size the pad", section.error || ""));
      resultRows.style.display = "none";
      formulaBox.style.display = "none";
      padDiagram.style.display = "none";
      return;
    }
    resultRows.style.display = "";
    formulaBox.style.display = "";
    padDiagram.style.display = "";
    const result = section.result;
    const band = result.status || "";
    const statusText =
      band === "ok"
        ? "Pad size is adequate"
        : band === "caution"
          ? "Pad is close to the limit"
          : band === "over"
            ? "Pad is too small"
            : "Enter a pad size";
    badge.replaceChildren(statusBadge(band === "size_required" ? "" : band, statusText, result.message));

    const rows = [
      ["Ground category", result.ground_category],
      ["Soil / rock", result.ground_description],
      [
        "Table value",
        result.ground_value_label === "Not applicable"
          ? "Not applicable"
          : `${result.ground_value_label} kPa`,
      ],
      ["Design V", fmt(result.bearing_kpa, 2, "kPa")],
      ["Crane mass CM", fmt(result.crane_mass_kn, 2, "kN")],
      ["Lift load L", fmt(result.lift_load_kn, 2, "kN")],
      ["Design load 0.65 × (CM + L)", fmt(result.design_load_kn, 2, "kN")],
      ["Required pad area A", fmt(result.required_area_m2, 3, "m²"), "", "big one-line"],
      ["Minimum square pad", fmt(result.recommended_square_rounded_m, 2, "m"), "caution", "big one-line"],
    ];
    if (result.pad_area_m2 !== null) {
      rows.push(
        ["Selected pad", `${fmt(result.pad_length_m, 2, "m")} × ${fmt(result.pad_width_m, 2, "m")}`],
        ["Pad area", fmt(result.pad_area_m2, 3, "m²")],
        ["Applied pressure", fmt(result.applied_pressure_kpa, 2, "kPa")],
        ["Utilisation", fmt(result.utilisation_percent, 2, "%"), bandClass(band), "big one-line"],
      );
    }
    const resultNodes = [kvRows(rows)];
    if (result.utilisation_percent !== null) {
      resultNodes.push(meter(result.utilisation_percent, band === "size_required" ? "" : band));
    }
    resultRows.replaceChildren(...resultNodes);

    formulaBox.append(
      el("div", { class: "formula-line", text: "A = 0.65 × (CM + L) / V" }),
      el("div", {
        class: "formula-line",
        text: `A = 0.65 × (${fmt(result.crane_mass_kn, 2, "")} + ${fmt(result.lift_load_kn, 2, "")}) / ${fmt(result.bearing_kpa, 2, "")} = ${fmt(result.required_area_m2, 3, "m²")}`,
      }),
      el("div", {
        class: "formula-line",
        text: `Minimum square side = √A = ${fmt(result.recommended_square_m, 3, "m")} → ${fmt(result.recommended_square_rounded_m, 2, "m")}`,
      }),
    );
    padDiagram.append(padPlan(result));
  }

  return { root, sync, render };
}
