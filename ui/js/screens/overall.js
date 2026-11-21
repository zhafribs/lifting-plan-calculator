// Overall Weight: the load lines and the tackle, and the figure every other
// screen checks against.

import { el, svgIcon } from "../dom.js";
import {
  card,
  clearButton,
  emptyState,
  exampleButton,
  kvRows,
  numberInput,
  reportButton,
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
          body: el("div", {}, loadRows, loadEmpty, el("div", { class: "mt12" }, addLoad)),
        }),
        card({
          title: "Lifting tackle",
          note: "Rigging that rides with the load",
          body: el(
            "div",
            {},
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
        card({
          title: "Actions",
          body: el(
            "div",
            { class: "row wrap" },
            exampleButton(() =>
              store.update((s) => {
                s.load_items = [
                  {
                    ...blankLoadItem(),
                    name: "Steel Plate",
                    load_type: "fixed",
                    qty: 1,
                    weight_kg: 1500,
                  },
                ];
                s.tackle_items = [
                  {
                    ...blankTackleItem(),
                    description: "2-Leg Chain Sling",
                    qty: 1,
                    weight_kg: 80,
                  },
                ];
              }),
            ),
            reportButton("Show report", () => ctx.showReport(["crane"])),
            clearButton(() =>
              store.update((s) => {
                s.load_items = [blankLoadItem()];
                s.tackle_items = [blankTackleItem()];
              }),
            ),
          ),
          foot: el("span", {
            class: "faint",
            text: "Clear removes every load and tackle line.",
          }),
        }),
      ),
    ),
  );

  // The fields one line type needs. The cell is re-painted whenever the type
  // changes, so the form always shows the measurement it is actually reading.
  function measurementFields(item, index) {
    const set = (patch) => store.update((state) => Object.assign(state.load_items[index], patch));
    const num = (key, unit, label) => {
      const input = numberInput({
        value: item[key],
        onInput: (value) => set({ [key]: value ?? 0 }),
      });
      input.setAttribute("aria-label", label);
      const wrap = withUnit(input, unit);
      wrap.dataset.unit = unit;
      return wrap;
    };
    if (item.load_type === "per_metre") {
      return [
        num("length_m", "m", "Length"),
        num("kg_per_metre", "kg/m", "Weight per metre"),
      ];
    }
    if (item.load_type === "per_volume") {
      return [num("volume_m3", "m3", "Volume"), num("density_kg_m3", "kg/m3", "Density")];
    }
    return [num("weight_kg", "kg", "Weight")];
  }

  function buildRows(state) {
    loadRows.replaceChildren();
    tackleRows.replaceChildren();
    loadTotalNodes = [];
    tackleTotalNodes = [];

    state.load_items.forEach((item, index) => {
      const total = el("span", { class: "line-total empty", text: "—" });
      const measure = el("div", { class: "row-measure" });
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
      const nameInput = textInput({
        value: item.name,
        placeholder: "e.g. steel plate",
        onInput: (value) =>
          store.update((s) => {
            s.load_items[index].name = value;
          }),
      });
      nameInput.className = "row-name";
      nameInput.setAttribute("aria-label", "Description");
      const typeSelect = selectInput({
        options: catalog.load_types.map((type) => ({ key: type.key, label: type.label })),
        value: item.load_type,
        onChange: (value) => {
          store.update((s) => {
            s.load_items[index].load_type = value;
          });
          paintMeasure();
        },
      });
      typeSelect.classList.add("row-mode");
      typeSelect.setAttribute("aria-label", "Measured by");
      const qtyInput = numberInput({
        value: item.qty,
        placeholder: "qty",
        onInput: (value) =>
          store.update((s) => {
            s.load_items[index].qty = Math.max(1, Math.round(value ?? 1));
          }),
      });
      qtyInput.classList.add("row-qty");
      qtyInput.setAttribute("aria-label", "Quantity");
      loadRows.append(
        el(
          "div",
          { class: "row-card load-row" },
          el("div", { class: "row-identity" }, nameInput, total),
          el(
            "div",
            { class: "row-working" },
            typeSelect,
            el("span", { class: "row-times", "aria-hidden": "true", text: "×" }),
            qtyInput,
            measure,
          ),
          remove,
        ),
      );
      loadTotalNodes.push(total);
    });

    state.tackle_items.forEach((item, index) => {
      const total = el("span", { class: "line-total empty", text: "—" });
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
      const descInput = textInput({
        value: item.description,
        placeholder: "e.g. 4-leg chain sling",
        onInput: (value) =>
          store.update((s) => {
            s.tackle_items[index].description = value;
          }),
      });
      descInput.className = "row-name";
      descInput.setAttribute("aria-label", "Description");
      const qtyInput = numberInput({
        value: item.qty,
        placeholder: "qty",
        onInput: (value) =>
          store.update((s) => {
            s.tackle_items[index].qty = Math.max(0, Math.round(value ?? 0));
          }),
      });
      qtyInput.classList.add("row-qty");
      qtyInput.setAttribute("aria-label", "Quantity");
      const weightInput = numberInput({
        value: item.weight_kg,
        onInput: (value) =>
          store.update((s) => {
            s.tackle_items[index].weight_kg = value ?? 0;
          }),
      });
      weightInput.setAttribute("aria-label", "Weight each");
      const weight = withUnit(weightInput, "kg");
      weight.dataset.unit = "kg";
      tackleRows.append(
        el(
          "div",
          { class: "row-card tackle-row" },
          el("div", { class: "row-identity" }, descInput, total),
          el(
            "div",
            { class: "row-working" },
            el("span", { class: "row-times", "aria-hidden": "true", text: "×" }),
            qtyInput,
            el("div", { class: "row-measure" }, weight),
          ),
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
    const setLineTotal = (node, weight) => {
      if (!node) return;
      if (weight === null || weight === undefined) {
        node.textContent = "—";
        node.classList.add("empty");
        return;
      }
      node.classList.remove("empty");
      node.replaceChildren(
        el("span", { class: "total-value", text: fmt(weight, 2, "") }),
        el("span", { class: "total-unit", text: "kg" }),
      );
    };
    (solved.line_weights || []).forEach((weight, index) =>
      setLineTotal(loadTotalNodes[index], weight),
    );
    (solved.tackle_subtotals || []).forEach((weight, index) =>
      setLineTotal(tackleTotalNodes[index], weight),
    );

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
