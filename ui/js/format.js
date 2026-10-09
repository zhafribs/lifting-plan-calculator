// Number formatting shared by every screen.
//
// Mirrors the engine's `figures.rs`: thousands separated, a fixed number of
// decimals, a dot decimal separator, and an em dash for "nothing to show".

export const NOTHING = "\u2014";

// Insert commas into the integer part of an already-formatted number.
export function groupDigits(text) {
  const negative = text.startsWith("-");
  const body = negative ? text.slice(1) : text;
  const [integer, fraction] = body.split(".");
  let grouped = "";
  for (let i = 0; i < integer.length; i += 1) {
    if (i > 0 && (integer.length - i) % 3 === 0) grouped += ",";
    grouped += integer[i];
  }
  const out = fraction === undefined ? grouped : `${grouped}.${fraction}`;
  return negative ? `-${out}` : out;
}

// A fixed-decimals figure with thousands separators.
export function grouped(value, decimals = 0) {
  if (value === null || value === undefined || Number.isNaN(value)) return NOTHING;
  if (!Number.isFinite(value)) return grouped(0, decimals);
  return groupDigits(value.toFixed(decimals));
}

// Format a figure, or an em dash when there is nothing to show.
export function fmt(value, decimals = 0, unit = "") {
  if (value === null || value === undefined || Number.isNaN(value)) return NOTHING;
  if (!Number.isFinite(value)) return fmt(0, decimals, unit);
  const text = groupDigits(value.toFixed(decimals));
  return unit ? `${text} ${unit}` : text;
}

export function fmtKg(value, decimals = 2) {
  return fmt(value, decimals, "kg");
}

export function fmtM(value, decimals = 3) {
  return fmt(value, decimals, "m");
}

export function fmtDeg(value, decimals = 1) {
  return fmt(value, decimals, "\u00b0");
}

export function fmtPct(value, decimals = 2) {
  return fmt(value, decimals, "%");
}

// Parse a text input as a number; empty / partial entries read as null.
export function parseNumber(text) {
  const trimmed = String(text ?? "").trim();
  if (trimmed === "" || trimmed === "-" || trimmed === "." || trimmed === "-.") return null;
  const value = Number(trimmed);
  return Number.isFinite(value) ? value : null;
}

// A dropdown option list from the engine's keyed pairs.
export function optionLabel(options, key) {
  const found = options.find((option) => option.key === key);
  return found ? found.label : key ?? NOTHING;
}
