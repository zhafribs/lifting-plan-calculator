// Nonuniform Load: the 2-leg asymmetric bridle with a shortening grab hook.

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

export function createNonuniformScreen(ctx) {
  const { store, catalog } = ctx;
  const set = (patch) => store.update((s) => Object.assign(s.nonuniform, patch));
  const num = (key, value) => numberInput({ value, onInput: (v) => set({ [key]: v }) });

  const lengthInput = num("sling_length", 0);
  const spanInput = num("pick_distance", 0);
  const cogInput = num("cog_from_pick1", 0);
  const stepInput = num("pick_point2_height", 0);

  const legFields = (prefix, wllKey, angleKey, legsKey) => {
    const wll = num(wllKey, 0);
    const angle = num(angleKey, 45);
    const legs = selectInput({
      options: catalog.sling_leg_options.map((value) => ({
        key: String(value),
        label: value === 1 ? "1 — single leg" : String(value),
      })),
      value: "1",
      onChange: (value) => set({ [legsKey]: Number(value) }),
    });
    const angleField = field({ label: `${prefix} tag WLL angle`, unit: "deg", input: angle });
    const node = el(
      "div",
      { class: "fields" },
      field({ label: `${prefix} tag WLL`, unit: "kg", input: wll }),
      field({ label: `${prefix} sling legs`, input: legs }),
      angleField,
    );
    return { node, wll, angle, legs, angleField };
  };

  const tag1 = legFields("Sling 1", "tag_wll1", "tag_wll1_angle", "sling_legs1");
  const tag2 = legFields("Sling 2", "tag_wll2", "tag_wll2_angle", "sling_legs2");

  const badge = el("div", {});
  const geometryRows = el("div", {});
  const comparisonRows = el("div", {});
  const tagRows = el("div", {});
  const tagFooter = el("div", { class: "faint small", style: { margin: "10px 2px 0" } });
  const errorText = el("div", { class: "error-text", style: { display: "none" } });

  const root = el(
    "div",
    { class: "page" },
    el(
      "div",
      { class: "grid-main" },
      el(
        "div",
        { class: "stack" },
        card({
          title: "2-Leg Asymmetric (Shortening Grab Hook)",
          note: "The COG may be measured from either pick point",
          body: el(
            "div",
            {},
            el(
              "div",
              { class: "fields" },
              field({ label: "Sling length", unit: "m", input: lengthInput }),
              field({ label: "Distance between pick points", unit: "m", input: spanInput }),
              field({ label: "COG from pick point 1", unit: "m", input: cogInput }),
              field({
                label: "Height difference (2 above 1)",
                unit: "m",
                input: stepInput,
                hint: "Negative means pick point 2 sits lower.",
              }),
            ),
            note("The solver works out which leg is long and shortens the other with the grab hook."),
          ),
        }),
        card({ title: "Installed tags", body: el("div", { class: "stack" }, tag1.node, tag2.node) }),
      ),
      el(
        "div",
        { class: "stack sticky" },
        errorText,
        card({ title: "Bridle result", body: el("div", {}, badge, geometryRows) }),
        card({ title: "Sling comparison", body: comparisonRows }),
        card({ title: "Installed tag check", body: el("div", {}, tagRows, tagFooter) }),
        card({
          title: "Actions",
          body: el(
            "div",
            { class: "row wrap" },
            exampleButton(() => {
              const example = catalog.example_nonuniform.find((entry) => entry[0] === "asym_2leg_shortening");
              if (example) store.update((s) => (s.nonuniform = { ...example[1] }));
            }),
            reportButton("Show report", () => ctx.showReport(["crane", "nonuniform"])),
            clearButton(() =>
              store.update((s) => {
                s.nonuniform = blankNonuniform("asym_2leg_shortening");
              }),
            ),
          ),
        }),
      ),
    ),
  );

  function blankNonuniform(scenario) {
    return {
      scenario,
      sling_length: 0,
      pick_distance: 0,
      cog_from_pick1: 0,
      pick_point2_height: 0,
      base_reeve_factor: 1,
      tag_wll1: 0,
      tag_wll1_angle: 45,
      sling_legs1: 1,
      tag_wll2: 0,
      tag_wll2_angle: 45,
      sling_legs2: 1,
      dist_a: 0,
      dist_b: 0,
      cog_height: 0,
      tilt_angle: 0,
      tag_wll_a: 0,
      tag_wll_a_angle: 45,
      sling_legs_a: 1,
      tag_wll_b: 0,
      tag_wll_b_angle: 45,
      sling_legs_b: 1,
      crane_rated_a: 0,
      crane_rated_b: 0,
    };
  }

  function sync(state) {
    const inputs = state.nonuniform;
    setIfIdle(lengthInput, inputs.sling_length);
    setIfIdle(spanInput, inputs.pick_distance);
    setIfIdle(cogInput, inputs.cog_from_pick1);
    setIfIdle(stepInput, inputs.pick_point2_height);
    syncTag(tag1, inputs.tag_wll1, inputs.tag_wll1_angle, inputs.sling_legs1);
    syncTag(tag2, inputs.tag_wll2, inputs.tag_wll2_angle, inputs.sling_legs2);
  }

  function syncTag(block, wll, angle, legs) {
    setIfIdle(block.wll, wll);
    setIfIdle(block.angle, angle);
    if (document.activeElement !== block.legs) block.legs.value = String(legs ?? 1);
    block.angleField.style.display = (legs ?? 1) >= 2 ? "" : "none";
  }

  function setIfIdle(input, value) {
    if (document.activeElement !== input) input.value = String(value ?? 0);
  }

  function render(solved) {
    if (!solved) return;
    const section = solved.nonuniform;
    const untouched = (store.state.nonuniform.sling_length ?? 0) <= 0;
    if (!section.ok || section.result?.kind !== "asym") {
      errorText.style.display = section.error && !untouched ? "" : "none";
      errorText.textContent = section.error || "";
      badge.replaceChildren();
      geometryRows.replaceChildren(
        untouched
          ? el("p", { class: "muted small", style: { margin: "4px 2px" }, text: "Enter the sling length and the pick points to solve the bridle." })
          : [],
      );
      comparisonRows.replaceChildren();
      tagRows.replaceChildren();
      tagFooter.textContent = "";
      return;
    }
    errorText.style.display = "none";
    const r = section.result;

    const ok = r.equipment_ok;
    badge.replaceChildren(
      statusBadge(
        ok === true ? "ok" : ok === false ? "over" : "",
        ok === true ? "Both tags meet their required WLL" : ok === false ? "At least one tag is below its required WLL" : "Enter both tag WLLs",
        `Governing leg: Sling ${r.governing_leg} — ${fmt(r.required_wll, 2, "kg")}`,
      ),
    );

    geometryRows.replaceChildren(
      kvRows([
        ["Span to leg 1", fmt(r.span1, 3, "m")],
        ["Span to leg 2", fmt(r.span2, 3, "m")],
        ["Long leg", `Sling ${r.long_leg}`],
        ["Short leg", `Sling ${r.short_leg} — shortened to fit`, "caution"],
        ["Headroom", fmt(r.headroom, 3, "m"), "", "big"],
        ["Drop from leg 1", fmt(r.drop1, 3, "m")],
        ["Drop from leg 2", fmt(r.drop2, 3, "m")],
        ["Short leg length", fmt(r.short_leg_length, 3, "m")],
        r.needs_lengthening
          ? ["Lengthening required", fmt(r.lengthening, 3, "m"), "over"]
          : ["Shortening", `${fmt(r.shortening_mm, 0)} mm`, "caution"],
        ["Length leg 1", fmt(r.length1, 3, "m")],
        ["Length leg 2", fmt(r.length2, 3, "m")],
        ["Angle leg 1", fmt(r.angle1, 2, "°")],
        ["Angle leg 2", fmt(r.angle2, 2, "°")],
        ["Angle factor leg 1", fmt(r.angle_factor1, 3)],
        ["Angle factor leg 2", fmt(r.angle_factor2, 3)],
        ["Maximum tension", fmt(r.max_tension, 2, "kg"), "caution"],
        ["Required WLL", fmt(r.required_wll, 2, "kg"), "caution", "big"],
        ["Setup SWL", fmt(r.setup_swl, 2, "kg")],
      ]),
    );

    comparisonRows.replaceChildren(
      el(
        "div",
        { class: "table-wrap" },
        el(
          "table",
          { class: "grid" },
          el(
            "thead",
            {},
            el("tr", {}, el("th", { text: "Figure" }), el("th", { class: "num", text: "Sling 1" }), el("th", { class: "num", text: "Sling 2" })),
          ),
          el(
            "tbody",
            {},
            ...[
              ["Load share", fmt(r.share1, 2, "kg"), fmt(r.share2, 2, "kg")],
              ["Line tension", fmt(r.tension1, 2, "kg"), fmt(r.tension2, 2, "kg")],
              ["Required WLL", fmt(r.required_wll1, 2, "kg"), fmt(r.required_wll2, 2, "kg")],
              ["Tag WLL", fmt(r.tag_wll1, 1, "kg"), fmt(r.tag_wll2, 1, "kg")],
              ["Chosen / rated legs", `${r.sling_legs1} / ${r.rated_legs1}`, `${r.sling_legs2} / ${r.rated_legs2}`],
              ["Per-leg capacity", fmt(r.leg_capacity1, 2, "kg"), fmt(r.leg_capacity2, 2, "kg")],
              ["Utilisation", fmt(r.utilisation1, 2, "%"), fmt(r.utilisation2, 2, "%")],
            ].map(([label, one, two]) =>
              el("tr", {}, el("td", { text: label }), el("td", { class: "num", text: one }), el("td", { class: "num", text: two })),
            ),
          ),
        ),
      ),
    );

    tagRows.replaceChildren(
      kvRows([
        ["Sling 1 per-leg capacity", r.leg_capacity1 === null ? "Not entered" : fmt(r.leg_capacity1, 2, "kg")],
        ["Sling 1 utilisation", r.utilisation1 === null ? "—" : fmt(r.utilisation1, 2, "%")],
        ["Sling 2 per-leg capacity", r.leg_capacity2 === null ? "Not entered" : fmt(r.leg_capacity2, 2, "kg")],
        ["Sling 2 utilisation", r.utilisation2 === null ? "—" : fmt(r.utilisation2, 2, "%")],
      ]),
    );
    const okState = r.equipment_ok;
    tagFooter.textContent =
      okState === null
        ? "Enter both tag WLLs to check the installed equipment."
        : okState
          ? "Both tags meet their required WLL."
          : "At least one tag is below its required WLL.";
    tagFooter.className = okState === false ? "error-text" : "faint small";
  }

  return { root, sync, render };
}
