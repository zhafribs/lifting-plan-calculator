// Shared building blocks: cards, fields, status badges, meter, tag blocks.

import { el, svgIcon, clear } from "./dom.js";
import { fmt } from "./format.js";

export function card({ title, note, actions, body, foot, class: extra = "" }) {
  const head = el(
    "div",
    { class: "card-head" },
    svgIcon(sparkFor(title), 16),
    el("h2", { text: title }),
    note ? el("span", { class: "note", text: note }) : null,
    el("span", { class: "spacer" }),
    ...(actions || []),
  );
  const children = [head];
  if (body) children.push(el("div", { class: "card-body" }, body));
  if (foot) children.push(el("div", { class: "card-foot" }, foot));
  return el("section", { class: `card ${extra}`.trim() }, children);
}

function sparkFor(title) {
  const t = (title || "").toLowerCase();
  if (t.includes("report") || t.includes("summary")) return "report";
  if (t.includes("crane") || t.includes("capacity")) return "crane";
  if (t.includes("tag") || t.includes("sling") || t.includes("rigging")) return "sling";
  if (t.includes("tilt") || t.includes("case") || t.includes("tandem")) return "tandem";
  if (t.includes("graph") || t.includes("range") || t.includes("diagram")) return "graph";
  if (t.includes("load") || t.includes("weight")) return "weight";
  return "about";
}

export function field({ label, unit, hint, input }) {
  return el(
    "label",
    { class: "field" },
    label ? el("span", { text: label }) : null,
    el("span", { class: "input-wrap" }, input, unit ? el("span", { class: "unit", text: unit }) : null),
    hint ? el("span", { class: "hint", text: hint }) : null,
  );
}

// A bare input with its unit suffix, for grids whose labels live in a header.
// The input is padded by the unit's own width, so "kg/m3" cannot overlap the
// digits the way a fixed padding would.
export function withUnit(input, unit) {
  if (unit && input.classList.contains("num")) {
    input.style.paddingRight = `${Math.ceil(unit.length * 7.2 + 18)}px`;
  }
  return el(
    "span",
    { class: "input-wrap" },
    input,
    unit ? el("span", { class: "unit", text: unit }) : null,
  );
}

// A numeric text input that reports parsed values and flags unparsable text.
export function numberInput({ value, onInput, placeholder, unit, disabled = false }) {
  const input = el("input", {
    class: "num",
    type: "text",
    inputmode: "decimal",
    autocomplete: "off",
    spellcheck: "false",
    placeholder: placeholder || "0",
    disabled,
  });
  input.dataset.raw = "";
  input.value = value === null || value === undefined ? "" : String(value);
  input.addEventListener("input", () => {
    const text = input.value;
    input.dataset.raw = text;
    const trimmed = text.trim();
    let parsed = null;
    let bad = false;
    if (trimmed !== "" && trimmed !== "-" && trimmed !== "." && trimmed !== "-.") {
      parsed = Number(trimmed);
      bad = !Number.isFinite(parsed);
    }
    input.classList.toggle("bad", bad);
    if (!bad && onInput) onInput(parsed);
  });
  return input;
}

export function textInput({ value, onInput, placeholder, disabled = false }) {
  const input = el("input", {
    type: "text",
    autocomplete: "off",
    placeholder: placeholder || "",
    disabled,
  });
  input.value = value ?? "";
  input.addEventListener("input", () => {
    if (onInput) onInput(input.value);
  });
  return input;
}

export function selectInput({ options, value, onChange, disabled = false }) {
  const select = el("select", { disabled });
  for (const option of options) {
    if (option.group) {
      const group = el("optgroup", { label: option.group });
      for (const child of option.options) {
        group.append(el("option", { value: child.key, text: child.label }));
      }
      select.append(group);
    } else {
      select.append(el("option", { value: option.key, text: option.label }));
    }
  }
  if (value !== null && value !== undefined) select.value = value;
  select.addEventListener("change", () => {
    if (onChange) onChange(select.value);
  });
  return select;
}

export function kvRows(rows) {
  const box = el("div", { class: "kv" });
  for (const row of rows) {
    if (!row) continue;
    const [label, value, kind] = row;
    box.append(
      el(
        "div",
        { class: `kv-row${row[3] ? ` ${row[3]}` : ""}` },
        el("span", { class: "k", text: label }),
        typeof value === "string" ? el("span", { class: `v ${kind || ""}`, text: value }) : value,
      ),
    );
  }
  return box;
}

export function statusBadge(band, text, sub) {
  const kind = band || "";
  return el(
    "div",
    { class: `badge ${kind}`.trim() },
    el("span", { class: "dot" }),
    el(
      "span",
      {},
      el("div", { class: "text", text }),
      sub ? el("div", { class: "sub", text: sub }) : null,
    ),
  );
}

export function meter(percent, band) {
  const width = Math.max(0, Math.min(100, percent ?? 0));
  return el(
    "div",
    { class: "meter" },
    el(
      "div",
      { class: "meter-track" },
      el("div", { class: `meter-fill ${band || ""}`, style: { width: `${width}%` } }),
    ),
    el(
      "div",
      { class: "meter-marks" },
      el("span", { class: "mark75", text: "75% allowable" }),
      el("span", { class: "mark100", text: "100%" }),
    ),
  );
}

// The installed-tag block shared by the uniform, nonuniform and tandem forms.
// `legs` values are the engine's SLING_LEG_OPTIONS; the angle hides for a
// single leg, exactly as the engine resolves it to zero.
export function tagBlock({
  prefix,
  inputs,
  legOptions,
  showSecondary = false,
  onChange,
}) {
  const fields = [
    field({
      label: `${prefix} tag WLL`,
      unit: "kg",
      input: numberInput({
        value: inputs.tag_wll,
        onInput: (value) => onChange({ tag_wll: value ?? 0 }),
      }),
    }),
    field({
      label: `${prefix} sling legs`,
      input: selectInput({
        options: legOptions.map((legs) => ({
          key: String(legs),
          label: legs === 1 ? "1 — single leg" : String(legs),
        })),
        value: String(inputs.sling_legs ?? 1),
        onChange: (value) => onChange({ sling_legs: Number(value) }),
      }),
    }),
  ];
  const needsAngle = (inputs.sling_legs ?? 1) >= 2;
  if (needsAngle) {
    fields.push(
      field({
        label: `${prefix} tag WLL angle`,
        unit: "deg",
        input: numberInput({
          value: inputs.tag_wll_angle,
          onInput: (value) => onChange({ tag_wll_angle: value ?? 45 }),
        }),
      }),
    );
  }
  return { node: el("div", { class: "fields" }, fields), showSecondary };
}

export function reportButton(label, onClick) {
  return el("button", { class: "btn primary", onClick }, svgIcon("report", 15), label);
}

export function exampleButton(onClick, label = "Load example") {
  return el("button", { class: "btn ghost", onClick }, svgIcon("example", 14), label);
}

export function clearButton(onClick) {
  return el("button", { class: "btn danger", onClick }, svgIcon("reset", 14), "Clear");
}

export function emptyState(text, action) {
  return el("div", { class: "empty" }, text, action ? el("div", { class: "mt8" }, action) : null);
}

// A small helper for the standard "note this sample" list on result cards.
export function note(text) {
  return el("p", { class: "faint small", style: { margin: "10px 2px 0" }, text });
}

export function renderInto(node, children) {
  clear(node);
  for (const child of children.flat(Infinity)) {
    if (child === null || child === undefined) continue;
    node.append(child);
  }
}

export { fmt };
