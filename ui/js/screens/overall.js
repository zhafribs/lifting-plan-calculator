// Overall Weight: the load lines and the tackle, and the figure every other
// screen checks against.

import { el, svgIcon } from "../dom.js";
import {
  card,
  emptyState,
  kvRows,
  numberInput,
  selectInput,
  textInput,
  withUnit,
} from "../components.js";
import { fmt } from "../format.js";
import { blankLoadItem, blankTackleItem } from "../store.js";


export function createOverallScreen(ctx) {
  const { store, catalog } = ctx;
  let structure = "";
  let loadTotalNodes = [];
  let tackleTotalNodes = [];

  const loadRows = el("div", { class: "rows" });
  const tackleRows = el("div", { class: "rows" });
  const loadEmpty = emptyState("No load items yet. Add the first thing you are lifting.");
  const tackleEmpty = emptyState(
    "No lifting tackle yet. Add shackles, strops, the hook block — anything that rides with the load.",
  );
  const summaryBox = el("div", {});

  const loadHead = el(
    "div",
    { class: "rows-head load-head" },
    el("span", { text: "Description" }),
    el("span", { text: "Measured by" }),
    el("span", { text: "Qty", style: { textAlign: "center" } }),
    el("span", { text: "Measurement" }),
    el("span", { text: "Subtotal", style: { textAlign: "right" } }),
    el("span", {}),
  );

  const tackleHead = el(
    "div",
    { class: "rows-head tackle-head" },
    el("span", { text: "Description" }),
    el("span", { text: "Qty", style: { textAlign: "center" } }),
    el("span", { text: "Weight each" }),
    el("span", { text: "Subtotal", style: { textAlign: "right" } }),
    el("span", {}),
  );

  const addLoad = el(
    "button",
    {
      class: "btn add wide",
      onClick: () =>
        store.update((state) => {
          state.load_items.push(blankLoadItem());
        }),
    },
    svgIcon("plus", 15),
    "Add load item",
  );

  const addTackle = el(
    "button",
    {
      class: "btn add wide",
      onClick: () =>
        store.update((state) => {
          state.tackle_items.push(blankTackleItem());
        }),
    },
    svgIcon("plus", 15),
    "Add tackle item",
  );

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
          title: "Load weight",
          note: "What the crane actually lifts",
          body: el("div", {}, loadHead, loadRows, loadEmpty, el("div", { class: "mt12" }, addLoad)),
        }),
        card({
          title: "Lifting tackle",
          note: "Rigging that rides with the load",
          body: el(
            "div",
            {},
            tackleHead,
            tackleRows,
            tackleEmpty,
            el("div", { class: "mt12" }, addTackle),
          ),
        }),
      ),
      el(
        "div",
        { class: "stack sticky pane" },
        card({
          title: "Weight summary",
          body: summaryBox,
          foot: el("span", {
            class: "faint",
            text: "Every other screen checks against the overall weight at the hook.",
          }),
        }),
        card({
          title: "Next step",
          body: el(
            "div",
            {},
            el("p", {
              class: "muted small",
              style: { margin: "0 0 12px" },
              text:
                "The crane must be charted for at least the minimum rating shown above " +
                "at the working radius — that is the 75% allowable window.",
            }),
            el(
              "button",
              {
                class: "btn primary wide",
                onClick: () => ctx.go("crane"),
              },
              svgIcon("crane", 15),
              "Check the crane",
            ),
          ),
        }),
      ),
    ),
  );

  // The fields one line type needs. The cell is re-painted whenever the type
  // changes, so the form always shows the measurement it is actually reading.
  function measurementFields(item, index) {
    const set = (patch) => store.update((state) => Object.assign(state.load_items[index], patch));
    const num = (key, unit) =>
      withUnit(
        numberInput({ value: item[key], onInput: (value) => set({ [key]: value ?? 0 }) }),
        unit,
      );
    if (item.load_type === "per_metre") {
      return [num("kg_per_metre", "kg/m"), num("length_m", "m")];
    }
    if (item.load_type === "per_volume") {
      return [num("volume_m3", "m3"), num("density_kg_m3", "kg/m3")];
    }
    return [num("weight_kg", "kg")];
  }

  function buildRows(state) {
    loadRows.replaceChildren();
    tackleRows.replaceChildren();
    loadTotalNodes = [];
    tackleTotalNodes = [];

    state.load_items.forEach((item, index) => {
      const total = el("span", { class: "line-total", text: "—" });
      const measure = el("div", { class: "row", style: { gap: "8px" } });
      const paintMeasure = () => measure.replaceChildren(...measurementFields(item, index));
      paintMeasure();
      const remove = el(
        "button",
        {
          class: "icon-btn remove",
          title: "Remove load line",
          onClick: () =>
            store.update((s) => {
              s.load_items.splice(index, 1);
            }),
        },
        svgIcon("close", 14),
      );
      loadRows.append(
        el(
          "div",
          { class: "row-card load-row" },
          textInput({
            value: item.name,
            placeholder: "e.g. steel plate",
            onInput: (value) =>
              store.update((s) => {
                s.load_items[index].name = value;
              }),
          }),
          selectInput({
            options: catalog.load_types.map((type) => ({ key: type.key, label: type.label })),
            value: item.load_type,
            onChange: (value) => {
              store.update((s) => {
                s.load_items[index].load_type = value;
              });
              paintMeasure();
            },
          }),
          numberInput({
            value: item.qty,
            placeholder: "qty",
            onInput: (value) =>
              store.update((s) => {
                s.load_items[index].qty = Math.max(1, Math.round(value ?? 1));
              }),
          }),
          measure,
          total,
          remove,
        ),
      );
      loadTotalNodes.push(total);
    });

    state.tackle_items.forEach((item, index) => {
      const total = el("span", { class: "line-total", text: "—" });
      const remove = el(
        "button",
        {
          class: "icon-btn remove",
          title: "Remove tackle line",
          onClick: () =>
            store.update((s) => {
              s.tackle_items.splice(index, 1);
            }),
        },
        svgIcon("close", 14),
      );
      tackleRows.append(
        el(
          "div",
          { class: "row-card tackle-row" },
          textInput({
            value: item.description,
            placeholder: "e.g. 4-leg chain sling",
            onInput: (value) =>
              store.update((s) => {
                s.tackle_items[index].description = value;
              }),
          }),
          numberInput({
            value: item.qty,
            placeholder: "qty",
            onInput: (value) =>
              store.update((s) => {
                s.tackle_items[index].qty = Math.max(0, Math.round(value ?? 0));
              }),
          }),
          withUnit(
            numberInput({
              value: item.weight_kg,
              onInput: (value) =>
                store.update((s) => {
                  s.tackle_items[index].weight_kg = value ?? 0;
                }),
            }),
            "kg",
          ),
          total,
          remove,
        ),
      );
      tackleTotalNodes.push(total);
    });

    loadEmpty.style.display = state.load_items.length ? "none" : "";
    tackleEmpty.style.display = state.tackle_items.length ? "none" : "";
  }

  function sync(state) {
    const key = `${state.load_items.map((item) => item.id).join(",")}|${state.tackle_items
      .map((item) => item.id)
      .join(",")}`;
    if (key !== structure) {
      structure = key;
      buildRows(state);
    }
    loadEmpty.style.display = state.load_items.length ? "none" : "";
    tackleEmpty.style.display = state.tackle_items.length ? "none" : "";
  }

  function render(solved) {
    if (!solved) return;
    (solved.line_weights || []).forEach((weight, index) => {
      if (loadTotalNodes[index]) loadTotalNodes[index].textContent = fmt(weight, 2, "kg");
    });
    (solved.tackle_subtotals || []).forEach((weight, index) => {
      if (tackleTotalNodes[index]) tackleTotalNodes[index].textContent = fmt(weight, 2, "kg");
    });

    const totals = solved.totals;
    summaryBox.replaceChildren(
      kvRows([
        ["Total load weight", fmt(totals.load, 2, "kg"), "", "one-line"],
        ["Lifting tackle weight", fmt(totals.tackle, 2, "kg"), "", "one-line"],
        [
          "Overall weight",
          fmt(totals.gross, 2, "kg"),
          totals.gross > 0 ? "accent" : "",
          "big one-line",
        ],
        [
          "Min. crane rating (75%)",
          fmt(totals.required_chart_capacity, 2, "kg"),
          totals.required_chart_capacity > 0 ? "caution" : "",
          "big one-line",
        ],
      ]),
      el("p", {
        class: "faint small",
        style: { margin: "10px 2px 0" },
        text:
          totals.gross > 0
            ? "Look the crane's load chart up at the working radius and make sure it rates at least this figure."
            : "Add a load to size the crane.",
      }),
    );
  }

  return { root, sync, render };
}
