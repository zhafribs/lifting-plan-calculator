// Windows 11 Fluent theme helpers.
//
// The platform and navigation decisions are pure so they can be tested without
// a browser. The CSS layer consumes the values these functions produce.

const DEFAULT_ACCENT_LIGHT = "#0067c0";
const DEFAULT_ACCENT_DARK = "#60cdff";

function normaliseHex(value) {
  if (typeof value !== "string") return null;
  const match = value.trim().match(/^#?([0-9a-f]{6})$/i);
  return match ? `#${match[1].toLowerCase()}` : null;
}

function channels(hex) {
  const clean = hex.slice(1);
  return [
    Number.parseInt(clean.slice(0, 2), 16),
    Number.parseInt(clean.slice(2, 4), 16),
    Number.parseInt(clean.slice(4, 6), 16),
  ];
}

function channelHex(value) {
  return Math.max(0, Math.min(255, Math.round(value)))
    .toString(16)
    .padStart(2, "0");
}

function shade(hex, amount) {
  const [r, g, b] = channels(hex);
  const next = amount < 0 ? 0 : 255;
  const mix = (value) => value + (next - value) * Math.abs(amount);
  return `#${channelHex(mix(r))}${channelHex(mix(g))}${channelHex(mix(b))}`;
}

function luminance(hex) {
  const [r, g, b] = channels(hex).map((value) => {
    const channel = value / 255;
    return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** The CSS platform scope. `--platform-preview` is the Linux QA hook. */
export function platformScope(info = {}) {
  const preview = info.platform_preview ?? info.platformPreview;
  return preview === "windows" || info.platform === "windows" ? "windows" : "default";
}

/** The rail starts compact on a narrow window and expanded otherwise. */
export function initialNavMode(width) {
  return Number(width) < 1000 ? "compact" : "expanded";
}

export function nextNavMode(mode) {
  return mode === "compact" ? "expanded" : "compact";
}

export function navToggleLabel(mode) {
  return mode === "compact" ? "Expand navigation" : "Collapse navigation";
}

/** Accent tokens for the root CSS variables. */
export function accentVars(hex, dark = false) {
  const accent = normaliseHex(hex) || (dark ? DEFAULT_ACCENT_DARK : DEFAULT_ACCENT_LIGHT);
  const [r, g, b] = channels(accent);
  const accentContrast = luminance(accent) > 0.55 ? "#000000" : "#ffffff";
  return {
    accent,
    accentSoft: `rgba(${r}, ${g}, ${b}, ${dark ? 0.16 : 0.12})`,
    accentHover: shade(accent, dark ? 0.14 : -0.14),
    accentContrast,
  };
}
