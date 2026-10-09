// Tandem: two independent cranes sharing one load, in the two declared lug
// configurations.

import { el } from "../dom.js";
import {
  card,
  clearButton,
  exampleButton,
  field,
  kvRows,
  meter,
  numberInput,
  note,
  reportButton,
  selectInput,
  statusBadge,
} from "../components.js";
import { fmt } from "../format.js";

export function createTandemScreen(ctx) {
  const { store, catalog } = ctx;
  const set = (patch) => store.update((s) => Object.assign(s.tandem, patch));
  const num = (key, value) => numberInput({ value, onInput: (v) => set({ [key]: v }) });

  const methodInput = selectInput({
    options: catalog.tandem_methods.map((method) => ({ key: method.key, label: method.label })),
    value: "tandem_aligned_pivot",
    onChange: (value) => store.update((s) => (s.tandem.scenario = value)),
  });

  const distAInput = num("dist_a", 0);
  const distBInput = num("dist_b", 0);
  const cogHeightInput = num("cog_height", 0);
  const tiltInput = num("tilt_angle", 0);

  const methodCard = card({
    title: "Method",
    body: el("div", {}, el("div", { class: "fields" }, field({ label: "Tandem method", input: methodInput })), el("div", { class: "method-note mt12" })),
  });

  const legsFor = (prefix, wllKey, angleKey, legsKey) => {
    const wll = num(wllKey, 0);
    const angle = num(angleKey, 45);
    const legs = selectInput({
      options: catalog.sling_leg_options.map((value) => ({ key: String(value), label: value === 1 ? "1 — single leg" : String(value) })),
      value: "1",
      onChange: (value) => set({ [legsKey]: Number(value) }),
    });
    const angleField = field({ label: `${prefix} tag WLL angle`, unit: "deg", input: angle });
    return {
      node: el("div", { class: "fields" }, field({ label: `${prefix} tag WLL`, unit: "kg", input: wll }), field({ label: `${prefix} sling legs`, input: legs }), angleField),
      wll,
      angle,
      legs,
      angleField,
    };
  };

  const crane1 = legsFor("Crane 1", "tag_wll_a", "tag_wll_a_angle", "sling_legs_a");
  const crane2 = legsFor("Crane 2", "tag_wll_b", "tag_wll_b_angle", "sling_legs_b");
  const ratedA = num("crane_rated_a", 0);
  const ratedB = num("crane_rated_b", 0);

  const geomRows = el("div", {});
  const caseA = el("div", {});
  const caseB = el("div", {});
  const caseC = el("div", {});
  const caseCCard = card({ title: "Case C — full 90° vertical rotation", body: caseC });
  const governingRows = el("div", {});
  const craneChecks = el("div", {});
  const craneFooter = el("div", { class: "faint small", style: { margin: "10px 2px 0" } });
  const errorText = el("div", { class: "error-text", style: { display: "none" } });
  const badge = el("div", {});

  const root = el(
    "div",
    { class: "page" },
    el(
      "div",
      { class: "grid-main" },
      el(
        "div",
        { class: "stack pane" },
        methodCard,
        card({
          title: "Lift",
          note: "The span is the sum of the two COG distances",
          body: el(
            "div",
            { class: "fields" },
            field({ label: "Distance, crane 1 hook to COG", unit: "m", input: distAInput }),
            field({ label: "Distance, crane 2 hook to COG", unit: "m", input: distBInput }),
            field({ label: "COG depth below baseline", unit: "m", input: cogHeightInput }),
            field({ label: "Cargo tilt angle", unit: "deg", input: tiltInput }),
          ),
        }),
        card({ title: "Crane 1", body: el("div", {}, crane1.node, el("div", { class: "fields mt12" }, field({ label: "Crane 1 rated capacity", unit: "kg", input: ratedA }))) }),
        card({ title: "Crane 2", body: el("div", {}, crane2.node, el("div", { class: "fields mt12" }, field({ label: "Crane 2 rated capacity", unit: "kg", input: ratedB }))) }),
      ),
      el(
        "div",
        { class: "stack sticky pane" },
        errorText,
        card({ title: "Tilted geometry", body: el("div", {}, badge, geomRows) }),
        card({ title: "Case A — level baseline", body: caseA }),
        card({ title: "Case B — declared tilt", body: caseB }),
        caseCCard,
        card({ title: "Governing case, per crane", body: governingRows }),
        card({ title: "Crane checks", body: el("div", {}, craneChecks, craneFooter) }),
        card({
          title: "Actions",
          body: el(
            "div",
            { class: "row wrap" },
            exampleButton(() => {
              const example = catalog.example_nonuniform.find((entry) => entry[0] === store.state.tandem.scenario);
              if (example) store.update((s) => (s.tandem = { ...example[1] }));
            }),
            reportButton("Show report", () => ctx.showReport(["crane", "tandem"])),
            clearButton(() =>
              store.update((s) => {
                const scenario = s.tandem.scenario;
                s.tandem = { ...blankTandem(), scenario };
              }),
            ),
          ),
        }),
      ),
    ),
  );

  function blankTandem() {
    return {
      scenario: "tandem_aligned_pivot",
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
    const inputs = state.tandem;
    if (document.activeElement !== methodInput) methodInput.value = inputs.scenario;
    setIfIdle(distAInput, inputs.dist_a);
    setIfIdle(distBInput, inputs.dist_b);
    setIfIdle(cogHeightInput, inputs.cog_height);
    setIfIdle(tiltInput, inputs.tilt_angle);
    syncLegs(crane1, inputs.tag_wll_a, inputs.tag_wll_a_angle, inputs.sling_legs_a);
    syncLegs(crane2, inputs.tag_wll_b, inputs.tag_wll_b_angle, inputs.sling_legs_b);
    setIfIdle(ratedA, inputs.crane_rated_a);
    setIfIdle(ratedB, inputs.crane_rated_b);

    const method = catalog.tandem_methods.find((entry) => entry.key === inputs.scenario) || catalog.tandem_methods[0];
    const noteBox = methodCard.querySelector(".method-note");
    noteBox.replaceChildren(
      el("p", { class: "muted small", style: { margin: "0 0 8px" }, text: method.use_for }),
      el("p", { class: "small", style: { margin: 0 } }, el("b", { text: "Crane 2 projection: " }), method.rule),
      el("p", { class: "faint small", style: { margin: "8px 0 0" }, text: `${method.lug2} ${method.span_behaviour} ${method.max_rotation}` }),
    );
  }

  function syncLegs(block, wll, angle, legs) {
    setIfIdle(block.wll, wll);
    setIfIdle(block.angle, angle);
    if (document.activeElement !== block.legs) block.legs.value = String(legs ?? 1);
    block.angleField.style.display = (legs ?? 1) >= 2 ? "" : "none";
  }

  function setIfIdle(input, value) {
    if (document.activeElement !== input) input.value = String(value ?? 0);
  }

  function caseRows(r, suffix) {
    const rows = [
      [`Crane 1 share (${suffix})`, fmt(r[`share_a_${suffix}`] ?? r.share_a_level, 2, "kg")],
      [`Crane 1 tension (${suffix})`, fmt(r[`tension_a_${suffix}`] ?? r.tension_a_level, 2, "kg")],
      [`Crane 1 required WLL (${suffix})`, fmt(r[`required_a_${suffix}`] ?? r.required_a_level, 2, "kg")],
      [`Crane 2 share (${suffix})`, fmt(r[`share_b_${suffix}`] ?? r.share_b_level, 2, "kg")],
      [`Crane 2 tension (${suffix})`, fmt(r[`tension_b_${suffix}`] ?? r.tension_b_level, 2, "kg")],
      [`Crane 2 required WLL (${suffix})`, fmt(r[`required_b_${suffix}`] ?? r.required_b_level, 2, "kg")],
    ];
    return kvRows(rows);
  }

  function render(solved) {
    if (!solved) return;
    const section = solved.tandem;
    const untouched = (store.state.tandem.dist_a ?? 0) <= 0;
    if (!section.ok || section.result?.kind !== "tandem") {
      errorText.style.display = section.error && !untouched ? "" : "none";
      errorText.textContent = section.error || "";
      badge.replaceChildren();
      geomRows.replaceChildren(
        untouched
          ? el("p", { class: "muted small", style: { margin: "4px 2px" }, text: "Enter both COG distances to solve the tandem lift." })
          : [],
      );
      caseA.replaceChildren();
      caseB.replaceChildren();
      caseC.replaceChildren();
      governingRows.replaceChildren();
      craneChecks.replaceChildren();
      craneFooter.textContent = "";
      return;
    }
    errorText.style.display = "none";
    const r = section.result;

    badge.replaceChildren(
      statusBadge(
        r.full_rotation ? "ok" : "caution",
        r.full_rotation ? "Can be stood fully upright" : "Rotation is limited",
        `Tilt limit: ${r.full_rotation ? "full 90°" : fmt(r.max_tilt_angle, 1, "°")}`,
      ),
    );

    geomRows.replaceChildren(
      kvRows([
        ["Level span between hooks", fmt(r.span, 3, "m")],
        ["COG depth below baseline", fmt(r.cog_height, 3, "m")],
        ["Cargo tilt", fmt(r.tilt_angle, 2, "°")],
        ["COG shift under lug 1", fmt(r.cog_shift, 3, "m")],
        ["Lug 2 inward travel", fmt(r.lug2_travel, 3, "m")],
        ["Crane 1 projected distance", fmt(r.dist_a_new, 3, "m")],
        ["Crane 2 projected distance", fmt(r.dist_b_new, 3, "m")],
        ["Span after the tilt", fmt(r.span_new, 3, "m")],
        ["Hook height difference", fmt(r.height_difference, 3, "m"), "", "big"],
      ]),
    );

    caseA.replaceChildren(caseRows(r, "level"));
    caseB.replaceChildren(caseRows(r, "tilt"));
    caseCCard.style.display = r.full_rotation ? "" : "none";
    if (r.full_rotation) caseC.replaceChildren(caseRows(r, "vertical"));

    governingRows.replaceChildren(
      kvRows([
        ["Crane 1 required WLL", fmt(r.required_wll_a, 2, "kg")],
        ["Crane 1 sized by", governingWords(r.governing_case_a)],
        ["Crane 2 required WLL", fmt(r.required_wll_b, 2, "kg")],
        ["Crane 2 sized by", governingWords(r.governing_case_b)],
        ["Maximum tension", fmt(r.max_tension, 2, "kg"), "caution"],
        ["Required WLL", fmt(r.required_wll, 2, "kg"), "caution", "big"],
        ["Setup SWL", fmt(r.setup_swl, 2, "kg")],
      ]),
    );

    craneChecks.replaceChildren(
      ...[
        ["Crane 1", r.tag_wll_a, r.sling_legs_a, r.rated_legs_a, r.leg_capacity_a, r.utilisation_a, r.crane_rated_a, r.crane_usage_a, r.crane_usage_state_a],
        ["Crane 2", r.tag_wll_b, r.sling_legs_b, r.rated_legs_b, r.leg_capacity_b, r.utilisation_b, r.crane_rated_b, r.crane_usage_b, r.crane_usage_state_b],
      ].map(([label, tagWll, chosen, rated, capacity, utilisation, craneRated, usage, usageState]) =>
        el(
          "div",
          { class: "mt12" },
          el("div", { class: "muted small", style: { marginBottom: "6px" }, text: label }),
          kvRows([
            ["Tag WLL", fmt(tagWll, 1, "kg")],
            ["Sling legs", `${chosen} chosen, rated on ${rated}`],
            ["Per-leg capacity", capacity === null ? "Not entered" : fmt(capacity, 2, "kg")],
            ["Tag utilisation", utilisation === null ? "—" : fmt(utilisation, 2, "%"), utilisation === null ? "" : utilisation <= 100 ? "ok" : "over"],
            ["Crane rated capacity", craneRated ? fmt(craneRated, 1, "kg") : "Not entered"],
            ["Crane usage", usage === null ? "—" : fmt(usage, 2, "%"), usage === null ? "" : bandClass(usageState)],
          ]),
          usage === null ? el("div", {}) : meter(usage, bandClass(usageState)),
        ),
      ),
    );

    const risk = r.crane_risk_flag;
    craneFooter.textContent =
      risk === null
        ? "Enter both crane rated capacities to check each crane against the 75% window."
        : risk
          ? "CHECK — at least one crane is booked outside the 75% window. That is a judgement about the crane, not the slings."
          : "Both cranes are inside the 75% window.";
    craneFooter.className = risk === true ? "error-text" : "faint small";
  }

  function bandClass(band) {
    return band === "within" ? "ok" : band === "caution" ? "caution" : band === "over" ? "over" : "";
  }

  function governingWords(key) {
    if (key === "level") return "level baseline";
    if (key === "tilt") return "declared tilt";
    if (key === "vertical") return "full 90° vertical rotation";
    return "—";
  }

  return { root, sync, render };
}
