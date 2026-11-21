// Outrigger Pad Size: the minimum bearing pad under one outrigger, sized from
// the BS 8004 presumed allowable bearing values in the ground table.

import { el } from "../dom.js";
import {
  card,
  clearButton,
  exampleButton,
  field,
  kvRows,
  note,
  numberInput,
  reportButton,
  selectInput,
  attachSelectMenu,
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
  attachSelectMenu(groundSelect);
  groundSelect.addEventListener("change", () => {
    const ground = catalog.ground_types.find((item) => item.key === groundSelect.value);
    if (!ground) return;
    set({
      ground_key: ground.key,
      bearing_kpa: ground.v_default ? ground.v_default : 0,
    });
  });

  const groundMeta = el("div", { class: "ground-meta" });
  const tableValueOut = el("div", { class: "readout-value", text: "—" });
  const tableValueField = el(
    "div",
    { class: "field readout-field" },
    el("span", { text: "Published table value" }),
    tableValueOut,
    el("span", { class: "hint", text: "BS 8004 Table 1" }),
  );

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
  const overallHint = el("p", {
    class: "faint small",
    style: { margin: "2px 2px 0" },
  });
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
          crane_capacity_t: Math.round(capacity * 100) / 100,
          crane_capacity_linked: true,
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

  const cmField = field({ label: "Crane mass (CM)", unit: "kN", input: cmInput });
  cmField.classList.add("linked-field");
  cmField.append(
    useCrane,
    el("span", {
      class: "hint",
      text: "Defaults to the Crane tab's rated capacity until edited.",
    }),
  );
  const liftField = field({ label: "Lift load (L)", unit: "kN", input: liftInput });
  liftField.classList.add("linked-field");
  liftField.append(
    useOverall,
    overallHint,
    el("span", {
      class: "hint",
      text: "Defaults to the Overall Weight gross load until edited.",
    }),
  );

  const shapeSelect = selectInput({
    options: [
      { key: "square", label: "Square" },
      { key: "rectangular", label: "Rectangular" },
    ],
    value: "square",
    onChange: (value) => set({ pad_shape: value }),
  });
  const widthInput = numberInput({
    value: 0,
    onInput: (value) => set({ pad_width_m: value ?? 0 }),
  });
  const padWidthField = field({
    label: "Available pad width",
    unit: "m",
    input: widthInput,
    hint: "The minimum length is calculated for this width.",
  });
  padWidthField.classList.add("pad-width-field");
  const padWidthBlock = el(
    "div",
    { class: "pad-extra" },
    el("span", { class: "pad-extra-title", text: "Rectangular pad" }),
    padWidthField,
  );

  const materialSelect = selectInput({
    options: [
      { key: "steel", label: "Steel plate" },
      { key: "timber", label: "Hardwood timber" },
    ],
    value: "steel",
    onChange: (value) => set({ thickness_material: value }),
  });
  const footLengthInput = numberInput({
    value: 0,
    onInput: (value) => set({ foot_length_m: value ?? 0 }),
  });
  const footWidthInput = numberInput({
    value: 0,
    onInput: (value) => set({ foot_width_m: value ?? 0 }),
  });
  const footFields = el(
    "div",
    { class: "fields" },
    field({
      label: "Outrigger foot length",
      unit: "m",
      input: footLengthInput,
    }),
    field({
      label: "Outrigger foot width",
      unit: "m",
      input: footWidthInput,
    }),
  );
  const footBlock = el(
    "div",
    { class: "pad-extra" },
    el("span", { class: "pad-extra-title", text: "Timber 45° rule" }),
    footFields,
    el("p", {
      class: "faint small",
      style: { margin: "6px 2px 0" },
      text: "Foot length and width are the steel outrigger float that sits on the pad. The timber thickness should roughly equal the overhang from the foot edge to the pad edge.",
    }),
  );

  const badge = el("div", {});
  const resultRows = el("div", {});
  const formulaBox = el("div", { class: "formula-block" });
  const padDiagram = el("div", { class: "pad-diagram" });
  const thicknessRows = el("div", {});
  const thicknessNotes = el("p", {
    class: "faint small",
    style: { margin: "10px 2px 0" },
  });
  const thicknessCard = card({
    title: "Pad thickness",
    note: "Steel plate or hardwood timber",
    body: el("div", {}, thicknessRows, thicknessNotes),
  });
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
            el("div", { class: "fields mt12" }, vField, tableValueField),
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
            el("div", { class: "fields" }, cmField, liftField),
          ),
        }),
        card({
          title: "Pad",
          note: "The formula gives the minimum bearing size",
          body: el(
            "div",
            { class: "stack-8" },
            el(
              "div",
              { class: "fields" },
              field({ label: "Pad shape", input: shapeSelect }),
              field({ label: "Pad material", input: materialSelect }),
            ),
            padWidthBlock,
            footBlock,
            note(
              "The app calculates the minimum pad for the chosen shape. For a rectangular pad, enter the width you have and the minimum length is calculated.",
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
        card({
          title: "Minimum pad size",
          body: el("div", {}, badge, resultRows, formulaBox, padDiagram),
        }),
        thicknessCard,
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
                  crane_capacity_linked: false,
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
                  crane_capacity_linked: true,
                  thickness_material: "steel",
                  crane_capacity_t: 0,
                  foot_length_m: 0,
                  foot_width_m: 0,
                  pad_shape: "square",
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
      tableValueOut.textContent = "—";
      groundMeta.replaceChildren(
        el("p", {
          class: "faint small",
          style: { margin: 0 },
          text: "Choose a category and description to load the published value and its field note.",
        }),
      );
      return;
    }
    tableValueOut.textContent = ground.suitable
      ? `${ground.value_label} kPa`
      : "Not applicable";
    const items = [el("p", { class: "ground-note", text: ground.notes })];
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
          "Design (V)",
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
    if (document.activeElement !== widthInput) widthInput.value = String(state.outrigger.pad_width_m ?? 0);
    if (document.activeElement !== materialSelect) materialSelect.value = state.outrigger.thickness_material || "steel";
    if (document.activeElement !== footLengthInput) footLengthInput.value = String(state.outrigger.foot_length_m ?? 0);
    if (document.activeElement !== footWidthInput) footWidthInput.value = String(state.outrigger.foot_width_m ?? 0);

    const rectangular = (state.outrigger.pad_shape || "square") === "rectangular";
    const timber = (state.outrigger.thickness_material || "steel") === "timber";
    padWidthBlock.style.display = rectangular ? "" : "none";
    footBlock.style.display = timber ? "" : "none";

    const gross = store.solved?.totals?.gross ?? 0;
    overallHint.textContent = gross > 0
      ? `Overall weight: ${fmt(gross, 2, "kg")} (${fmt(gross * catalog.kg_to_kn, 2, "kN")}).`
      : "No overall weight entered yet.";
    useOverall.disabled = gross <= 0;
    useCrane.disabled = (store.state.crane?.capacity_t ?? 0) <= 0;
  }

  function padPlan(result) {
    const rectangular = result.pad_shape === "rectangular";
    const padLength = rectangular
      ? result.minimum_length_m
      : result.recommended_square_rounded_m;
    const padWidth = rectangular
      ? result.pad_width_m
      : result.recommended_square_rounded_m;
    if (!padLength || !padWidth) {
      return el("p", {
        class: "faint small",
        style: { margin: "10px 2px 0" },
        text: "Enter a pad width to draw the rectangular pad.",
      });
    }

    const width = 340;
    const height = 214;
    const maxW = 220;
    const maxH = 96;
    const ratio = padLength / padWidth;
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
    const svg = `<svg viewBox="0 0 ${width} ${height}" width="100%" height="${height}" role="img" aria-label="Minimum outrigger pad plan">
      <rect x="${x}" y="${y}" width="${padW}" height="${padH}" rx="6" fill="#1c2838" stroke="#5cb3ff" stroke-width="1.5"/>
      <line x1="${x}" y1="${dimY}" x2="${x + padW}" y2="${dimY}" stroke="#e2a558" stroke-width="1" stroke-dasharray="4 3"/>
      <text x="${x + padW / 2}" y="${dimY + 14}" fill="#e2a558" font-size="10.5" text-anchor="middle">${fmt(padLength, 2, "m")}</text>
      <line x1="${dimX}" y1="${y}" x2="${dimX}" y2="${y + padH}" stroke="#e2a558" stroke-width="1" stroke-dasharray="4 3"/>
      <text x="${dimX + 6}" y="${y + padH / 2}" fill="#e2a558" font-size="10.5" dominant-baseline="middle">${fmt(padWidth, 2, "m")}</text>
      <circle cx="${x + padW / 2}" cy="${y + padH / 2}" r="4" fill="#e2a558"/>
      <text x="${x + padW / 2}" y="${y + padH / 2 - 10}" fill="#e2a558" font-size="9.5" text-anchor="middle">outrigger</text>
      <text x="${width / 2}" y="${height - 6}" fill="#64748d" font-size="9.5" text-anchor="middle">Minimum pad, plan view, not to scale</text>
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
      thicknessCard.style.display = "none";
      return;
    }
    badge.replaceChildren();
    resultRows.style.display = "";
    formulaBox.style.display = "";
    padDiagram.style.display = "";
    thicknessCard.style.display = "";
    const result = section.result;
    const rectangular = result.pad_shape === "rectangular";

    const rows = [
      ["Ground category", result.ground_category],
      ["Soil / rock", result.ground_description],
      [
        "Table value",
        result.ground_value_label === "Not applicable"
          ? "Not applicable"
          : `${result.ground_value_label} kPa`,
      ],
      ["Design (V)", fmt(result.bearing_kpa, 2, "kPa")],
      ["Crane mass (CM)", fmt(result.crane_mass_kn, 2, "kN")],
      ["Lift load (L)", fmt(result.lift_load_kn, 2, "kN")],
      ["Design load (P)", fmt(result.design_load_kn, 2, "kN")],
      ["Required pad area (A)", fmt(result.required_area_m2, 3, "m²"), "", "big one-line"],
    ];
    if (rectangular && result.minimum_length_m !== null) {
      rows.push(
        ["Available pad width", fmt(result.pad_width_m, 2, "m")],
        [
          "Minimum pad",
          `${fmt(result.minimum_length_m, 2, "")} × ${fmt(result.pad_width_m, 2, "")} m`,
          "",
          "big one-line",
        ],
      );
    } else {
      rows.push([
        "Minimum pad",
        `${fmt(result.recommended_square_rounded_m, 2, "")} × ${fmt(result.recommended_square_rounded_m, 2, "")} m`,
        "",
        "big one-line",
      ]);
    }
    resultRows.replaceChildren(kvRows(rows));

    const formulaLines = [
      el("div", { class: "formula-line", text: "A = 0.65 × (CM + L) / V" }),
      el("div", {
        class: "formula-line",
        text: `A = 0.65 × (${fmt(result.crane_mass_kn, 2, "")} + ${fmt(result.lift_load_kn, 2, "")}) / ${fmt(result.bearing_kpa, 2, "")} = ${fmt(result.required_area_m2, 3, "m²")}`,
      }),
    ];
    if (rectangular && result.minimum_length_m !== null) {
      formulaLines.push(
        el("div", {
          class: "formula-line",
          text: `Minimum length = ceil(A / ${fmt(result.pad_width_m, 2, "")} / 0.05) × 0.05 = ${fmt(result.minimum_length_m, 2, "m")}`,
        }),
      );
    } else {
      formulaLines.push(
        el("div", {
          class: "formula-line",
          text: `Minimum square side = √A = ${fmt(result.recommended_square_m, 3, "m")} → ${fmt(result.recommended_square_rounded_m, 2, "m")}`,
        }),
      );
    }
    formulaBox.append(...formulaLines);
    padDiagram.append(padPlan(result));

    const thickness = result.thickness;
    const recommended = thickness.recommended_max_mm > thickness.recommended_min_mm
      ? `${fmt(thickness.recommended_min_mm, 0, "mm")} – ${fmt(thickness.recommended_max_mm, 0, "mm")}`
      : fmt(thickness.recommended_min_mm, 0, "mm");
    const thicknessRowsList = [
      ["Material", thickness.material === "timber" ? "Hardwood timber" : "Steel plate"],
      ["Crane class", thickness.crane_class],
      ["Steel plate range", thickness.steel_range_mm],
      ["Timber range", thickness.timber_range_mm],
      ["Recommended thickness", recommended, "", "big one-line"],
      ["Governing check", thickness.governing],
    ];
    if (thickness.cross_layer) {
      thicknessRowsList.push(["Cross-layering", "Required"]);
    }
    if (thickness.stiffeners) {
      thicknessRowsList.push(["Stiffeners", "Required or 40–50 mm plate"]);
    }
    thicknessRows.replaceChildren(kvRows(thicknessRowsList));
    thicknessNotes.textContent = thickness.notes;
  }

  return { root, sync, render };
}
