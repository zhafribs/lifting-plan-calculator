// Uniform Load: one continuous sling, checked against the gross load at hook.

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
  statusBadge,
} from "../components.js";
import { fmt } from "../format.js";

export function createUniformScreen(ctx) {
  const { store, catalog } = ctx;

  const set = (patch) => store.update((s) => Object.assign(s.sling, patch));

  const hitchInput = selectInput({
    options: hitchOptions(catalog),
    value: "direct_hook",
    onChange: (value) => store.update((s) => (s.sling.hitch = value)),
  });
  const lengthInput = numberInput({ value: 0, onInput: (value) => set({ sling_length: value ?? 0 }) });

  const diameterInput = numberInput({ value: 0, onInput: (value) => set({ diameter: value ?? 0 }) });
  const sideInput = numberInput({ value: 0, onInput: (value) => set({ side: value ?? 0 }) });
  const bottomInput = numberInput({ value: 0, onInput: (value) => set({ bottom: value ?? 0 }) });
  const chokeInput = numberInput({ value: 60, onInput: (value) => set({ choke_angle: value ?? 60 }) });
  const basisInput = selectInput({
    options: [
      { key: "internal", label: "Internal — at choke" },
      { key: "external", label: "External — hook to load" },
    ],
    value: "internal",
    onChange: (value) => set({ angle_basis: value }),
  });
  const pickDistanceInput = numberInput({ value: 0, onInput: (value) => set({ pick_distance: value ?? 0 }) });
  const pickLengthInput = numberInput({ value: 0, onInput: (value) => set({ pick_length: value ?? 0 }) });
  const pickWidthInput = numberInput({ value: 0, onInput: (value) => set({ pick_width: value ?? 0 }) });
  const secLengthInput = numberInput({ value: 0, onInput: (value) => set({ secondary_length: value ?? 0 }) });
  const secSpreadInput = numberInput({ value: 0, onInput: (value) => set({ secondary_spread: value ?? 0 }) });
  const secTagInput = numberInput({ value: 0, onInput: (value) => set({ secondary_tag_wll: value ?? 0 }) });
  const secTagAngleInput = numberInput({ value: 45, onInput: (value) => set({ secondary_tag_wll_angle: value ?? 45 }) });
  const tagWllInput = numberInput({ value: 0, onInput: (value) => set({ tag_wll: value ?? 0 }) });
  const tagAngleInput = numberInput({ value: 45, onInput: (value) => set({ tag_wll_angle: value ?? 45 }) });

  const loadCard = card({
    title: "Load",
    body: el("div", { class: "fields" }, field({ label: "Load diameter", unit: "m", input: diameterInput }), field({ label: "Load side / height", unit: "m", input: sideInput }), field({ label: "Load bottom / length", unit: "m", input: bottomInput })),
  });
  const chokeCard = card({
    title: "Choke",
    note: " ",
    body: el(
      "div",
      {},
      el("div", { class: "fields" }, field({ label: "Choke angle", unit: "deg", input: chokeInput }), field({ label: "Angle basis", input: basisInput })),
      note("The choke angle derates the tag's rating through the documented reduction table."),
    ),
  });
  const pickCard = card({
    title: "Pick points",
    body: el("div", { class: "fields" }, field({ label: "Pick-point distance", unit: "m", input: pickDistanceInput }), field({ label: "Pick-point length", unit: "m", input: pickLengthInput }), field({ label: "Pick-point width", unit: "m", input: pickWidthInput })),
  });
  const secondaryCard = card({
    title: "Secondary slings",
    note: "Each master leg carries its own sling",
    body: el(
      "div",
      {},
      el("div", { class: "fields" }, field({ label: "Secondary sling length", unit: "m", input: secLengthInput }), field({ label: "Secondary pick spacing", unit: "m", input: secSpreadInput })),
      el("div", { class: "fields mt12" }, field({ label: "Secondary tag WLL", unit: "kg", input: secTagInput }), field({ label: "Secondary tag WLL angle", unit: "deg", input: secTagAngleInput })),
    ),
  });
  const tagCard = card({
    title: "Installed tag",
    body: el("div", { class: "fields" }, field({ label: "Tag WLL", unit: "kg", input: tagWllInput }), el("div", { class: "tag-angle" }, field({ label: "Tag WLL angle", unit: "deg", input: tagAngleInput }))),
    foot: el("span", { class: "tag-note faint" }),
  });

  const resultBadge = el("div", {});
  const resultRows = el("div", {});
  const shapeNote = el("div", {});
  const tagRows = el("div", {});
  const secRows = el("div", {});
  const groupRows = el("div", {});

  const resultCard = card({ title: "Sling result", body: el("div", {}, resultBadge, resultRows, shapeNote) });
  const tagResultCard = card({ title: "Installed tag check", body: tagRows });
  const secondaryResultCard = card({ title: "Secondary slings", body: el("div", {}, secRows, groupRows) });

  const errorText = el("div", { class: "error-text", style: { display: "none" } });

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
          title: "Sling",
          body: el("div", { class: "fields" }, field({ label: "Arrangement", input: hitchInput }), field({ label: "Sling total length", unit: "m", input: lengthInput })),
        }),
        loadCard,
        chokeCard,
        pickCard,
        secondaryCard,
        tagCard,
      ),
      el(
        "div",
        { class: "stack sticky pane" },
        errorText,
        resultCard,
        tagResultCard,
        secondaryResultCard,
        card({
          title: "Actions",
          body: el(
            "div",
            { class: "row wrap" },
            exampleButton(() => {
              const example = catalog.example_sling.find((entry) => entry[0] === store.state.sling.hitch);
              if (example) store.update((s) => (s.sling = { ...example[1] }));
            }),
            reportButton("Show report", () => ctx.showReport(["crane", "uniform"])),
            clearButton(() => store.update((s) => (s.sling = blankSling()))),
          ),
          foot: el("span", { class: "faint", text: "Clear returns the arrangement to Direct hook and blanks the measurements." }),
        }),
      ),
    ),
  );

  function blankSling() {
    return {
      hitch: "direct_hook",
      sling_length: 0,
      diameter: 0,
      side: 0,
      bottom: 0,
      choke_angle: 60,
      angle_basis: "internal",
      tag_wll: 0,
      tag_wll_angle: 45,
      pick_distance: 0,
      pick_length: 0,
      pick_width: 0,
      secondary_length: 0,
      secondary_spread: 0,
      secondary_tag_wll: 0,
      secondary_tag_wll_angle: 45,
    };
  }

  function specFor(state) {
    return catalog.hitches.find((hitch) => hitch.key === state.sling.hitch) || catalog.hitches[0];
  }

  function sync(state) {
    const spec = specFor(state);
    if (document.activeElement !== hitchInput) hitchInput.value = state.sling.hitch;
    setIfIdle(lengthInput, state.sling.sling_length);
    setIfIdle(diameterInput, state.sling.diameter);
    setIfIdle(sideInput, state.sling.side);
    setIfIdle(bottomInput, state.sling.bottom);
    setIfIdle(chokeInput, state.sling.choke_angle);
    if (document.activeElement !== basisInput) basisInput.value = state.sling.angle_basis;
    setIfIdle(pickDistanceInput, state.sling.pick_distance);
    setIfIdle(pickLengthInput, state.sling.pick_length);
    setIfIdle(pickWidthInput, state.sling.pick_width);
    setIfIdle(secLengthInput, state.sling.secondary_length);
    setIfIdle(secSpreadInput, state.sling.secondary_spread);
    setIfIdle(secTagInput, state.sling.secondary_tag_wll);
    setIfIdle(secTagAngleInput, state.sling.secondary_tag_wll_angle);
    setIfIdle(tagWllInput, state.sling.tag_wll);
    setIfIdle(tagAngleInput, state.sling.tag_wll_angle);

    loadCard.style.display = spec.round_load || spec.rectangular_load ? "" : "none";
    fieldOf(diameterInput).style.display = spec.round_load ? "" : "none";
    fieldOf(sideInput).style.display = spec.rectangular_load ? "" : "none";
    fieldOf(bottomInput).style.display = spec.rectangular_load ? "" : "none";

    chokeCard.style.display = spec.needs_angle ? "" : "none";
    chokeCard.querySelector(".card-head h2");
    const chokeNote = chokeCard.querySelector(".card-head .note");
    if (chokeNote) chokeNote.textContent = spec.angle_description || "";

    const pickVisible = spec.needs_pick_distance || spec.shape === "bridle4";
    pickCard.style.display = pickVisible ? "" : "none";
    fieldOf(pickDistanceInput).style.display = spec.needs_pick_distance && spec.shape !== "bridle4" ? "" : "none";
    fieldOf(pickLengthInput).style.display = spec.shape === "bridle4" ? "" : "none";
    fieldOf(pickWidthInput).style.display = spec.shape === "bridle4" ? "" : "none";

    secondaryCard.style.display = spec.nested ? "" : "none";
    const tagAngleField = tagCard.querySelector(".tag-angle");
    if (tagAngleField) tagAngleField.style.display = spec.needs_tag_angle ? "" : "none";
    const tagNote = tagCard.querySelector(".tag-note");
    if (tagNote) {
      tagNote.textContent = spec.needs_tag_angle
        ? "The tag's rating is derated by cos(Tag WLL Angle) across the legs."
        : "A single leg hangs vertical, so it takes the tag's full rating and there is no angle to enter.";
    }
  }

  function setIfIdle(input, value) {
    if (document.activeElement !== input) input.value = String(value ?? 0);
  }

  function fieldOf(input) {
    return input.closest(".field");
  }

  function render(solved) {
    if (!solved) return;
    const section = solved.sling;
    const untouched = (store.state.sling.sling_length ?? 0) <= 0;

    if (!section.ok) {
      errorText.style.display = untouched ? "none" : "";
      errorText.textContent = section.error || "The sling calculation could not be completed.";
      resultBadge.replaceChildren();
      resultRows.replaceChildren(
        untouched
          ? el("p", { class: "muted small", style: { margin: "4px 2px" }, text: "Enter the sling total length to solve this arrangement." })
          : [],
      );
      shapeNote.replaceChildren();
      tagRows.replaceChildren();
      secRows.replaceChildren();
      groupRows.replaceChildren();
      tagResultCard.style.display = "none";
      secondaryResultCard.style.display = "none";
      return;
    }
    tagResultCard.style.display = "";

    errorText.style.display = "none";
    const result = section.result;
    const ok = result.equipment_ok;
    resultBadge.replaceChildren(
      statusBadge(
        ok === true ? "ok" : ok === false ? "over" : "",
        ok === true ? "Tag meets the requirement" : ok === false ? "Tag below the requirement" : "No tag rating entered",
        result.shape_note,
      ),
    );

    const rows = [
      ["Arrangement", result.hitch_label],
      ["Effective legs", String(result.effective_legs)],
      ["Sling angle factor", fmt(result.angle_factor, 3)],
    ];
    if (result.angle > 0) rows.push(["Sling angle", fmt(result.angle, 2, "°")]);
    if (result.angle_long !== null && result.angle_trans !== null) {
      rows.push(["Longitudinal angle", fmt(result.angle_long, 2, "°")]);
      rows.push(["Transverse angle", fmt(result.angle_trans, 2, "°")]);
      rows.push(["Combined factor", fmt(result.secondary_combined_af ?? result.angle_factor, 3)]);
    }
    rows.push(["Sling length", fmt(result.sling_length, 3, "m")]);
    if (result.wrap_length > 0) rows.push(["Wrap length", fmt(result.wrap_length, 3, "m")]);
    rows.push(["Free leg", fmt(result.free_leg_length, 3, "m")]);
    rows.push(["Headroom above load", fmt(result.hook_to_load, 3, "m"), "", "big"]);
    if (result.external_angle !== null) {
      rows.push(["Choke angle (external)", fmt(result.external_angle, 1, "°")]);
      rows.push(["Choke angle (internal)", fmt(result.internal_angle, 1, "°")]);
      rows.push(["Choke reduction", fmt(result.choke_reduction_factor, 2)]);
    }
    rows.push(["Reeve factor", fmt(result.base_reeve_factor, 2)]);
    rows.push(["Rated on", `${result.rating_legs} leg${result.rating_legs === 1 ? "" : "s"}`]);
    rows.push(["Tension per leg", fmt(result.tension_each, 2, "kg")]);
    rows.push(["Maximum tension", fmt(result.max_tension, 2, "kg"), "caution"]);
    rows.push(["Required WLL", fmt(result.required_wll, 2, "kg"), "caution", "big"]);
    resultRows.replaceChildren(kvRows(rows));
    shapeNote.replaceChildren(
      el("p", { class: "faint small", style: { margin: "10px 2px 0" }, text: result.shape_note }),
    );

    const tagRowsList = [["Tag WLL", fmt(result.tag_wll, 1, "kg")]];
    if (result.tag_wll_angle > 0) {
      tagRowsList.push(["Tag WLL angle", fmt(result.tag_wll_angle, 1, "°")]);
      tagRowsList.push(["Tag angle factor", fmt(result.tag_angle_factor, 3)]);
    }
    if (result.leg_capacity !== null) {
      tagRowsList.push(["Per-leg capacity", fmt(result.leg_capacity, 2, "kg")]);
      tagRowsList.push(["Utilisation", fmt(result.utilisation, 2, "%"), result.equipment_ok ? "ok" : "over"]);
    }
    tagRowsList.push([
      "Status",
      result.equipment_status,
      result.equipment_ok === true ? "ok" : result.equipment_ok === false ? "over" : "",
    ]);
    tagRows.replaceChildren(kvRows(tagRowsList));

    const nested = result.nested !== 0;
    secondaryResultCard.style.display = nested ? "" : "none";
    if (nested) {
      secRows.replaceChildren(
        kvRows([
          ["Physical legs", String(result.secondary_legs ?? "—")],
          ["Rated on", result.secondary_rating_legs ? `${result.secondary_rating_legs} legs` : "—"],
          ["Secondary angle", fmt(result.secondary_angle, 2, "°")],
          ["Secondary factor", fmt(result.secondary_af, 3)],
          ["Combined factor", fmt(result.secondary_combined_af, 3)],
          ["Secondary height", fmt(result.secondary_height, 3, "m")],
          ["Total combined length", fmt(result.total_combined_length, 2, "m")],
          ...(result.pile_level_difference !== null
            ? [["Pile level difference", fmt(result.pile_level_difference, 3, "m"), "caution"]]
            : []),
          ["Highest secondary tension", fmt(result.secondary_tension, 2, "kg"), "caution"],
          ["Secondary required WLL", fmt(result.secondary_required_wll, 2, "kg"), "caution"],
          ...(result.secondary_leg_capacity !== null
            ? [
                ["Secondary per-leg capacity", fmt(result.secondary_leg_capacity, 2, "kg")],
                ["Secondary utilisation", fmt(result.secondary_utilisation, 2, "%"), result.secondary_equipment_ok ? "ok" : "over"],
                ["Secondary status", result.secondary_equipment_status, result.secondary_equipment_ok ? "ok" : "over"],
              ]
            : []),
        ]),
      );
      const groups = result.secondary_groups || [];
      if (groups.length > 1) {
        groupRows.replaceChildren(
          el("div", { class: "faint small", style: { margin: "12px 2px 6px" }, text: "Leg groups, outermost first" }),
          el(
            "div",
            { class: "table-wrap" },
            el(
              "table",
              { class: "grid" },
              el("thead", {}, el("tr", {}, el("th", { text: "Offset" }), el("th", { text: "Tension" }))),
              el(
                "tbody",
                {},
                ...groups.map((group) =>
                  el(
                    "tr",
                    {},
                    el("td", { text: `${fmt(group.offset, 2, "m")}${group.governing ? " (governing)" : ""}` }),
                    el("td", { class: `num ${group.governing ? "caution" : ""}`.trim(), text: fmt(group.tension, 2, "kg") }),
                  ),
                ),
              ),
            ),
          ),
        );
      } else {
        groupRows.replaceChildren();
      }
    }
  }

  function hitchOptions(catalog) {
    const group = (name, keys) => ({
      group: name,
      options: keys
        .map((key) => catalog.hitches.find((hitch) => hitch.key === key))
        .filter(Boolean)
        .map((hitch) => ({ key: hitch.key, label: hitch.label })),
    });
    return [
      group("Single leg", ["direct_hook", "round_choke", "rect_choke", "round_basket", "rect_basket", "vertical_round_basket", "vertical_rect_basket"]),
      group("Two legs", ["two_leg_direct", "two_leg_round_choke", "two_leg_rect_choke", "two_leg_round_basket", "two_leg_rect_basket", "two_leg_vertical_round_basket", "two_leg_vertical_rect_basket"]),
      group("Nested systems", ["two_leg_nested_2leg", "two_leg_nested_3leg", "two_leg_nested_4leg"]),
      group("Bridles", ["three_leg_direct", "four_leg_direct"]),
    ];
  }

  return { root, sync, render };
}
