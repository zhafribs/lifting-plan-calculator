// A minimal Canvas2D-compatible surface for the printed diagram. The graph's
// paint code already speaks these primitives; emitting SVG instead of pixels
// keeps the line work and labels vector in the PDF.

import { printColour } from "./print-palette.js";

const TAU = Math.PI * 2;

function n(value) {
  return Math.round(value * 100) / 100;
}

function escapeXml(value) {
  return String(value).replace(
    /[&<>"']/g,
    (character) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&apos;" })[
        character
      ],
  );
}

function colourAttrs(value, kind, backdrop) {
  const colour = printColour(value, backdrop);
  const rgba = /^rgba\(\s*([^)]+)\)$/i.exec(colour);
  if (!rgba) return `${kind}="${escapeXml(colour)}"`;
  const parts = rgba[1].split(",").map((part) => part.trim());
  return `${kind}="rgb(${parts[0]}, ${parts[1]}, ${parts[2]})" ${kind}-opacity="${escapeXml(parts[3])}"`;
}

export function createSvgSurface({
  width,
  height,
  scale = 1,
  measure = null,
  fontFamily: fontOverride = null,
  fontData = null,
  fontName = "InterDiagram",
  measureFamily = null,
  backdrop = "#141922",
} = {}) {
  const elements = [];
  let path = "";
  let current = null;
  let fillStyle = "#000000";
  let strokeStyle = "#000000";
  let lineWidth = 1;
  let lineCap = "butt";
  let dashed = [];
  let font = "10px sans-serif";
  let fontWeight = "400";
  let fontSize = "10";
  let fontFamily = "sans-serif";
  let textAlign = "start";
  let textBaseline = "alphabetic";

  function point(x, y) {
    if (!current) path += `M ${n(x)} ${n(y)}`;
    else path += ` L ${n(x)} ${n(y)}`;
    current = [x, y];
  }

  function arc(cx, cy, radius, start, end, anticlockwise = false) {
    let delta = end - start;
    if (anticlockwise) {
      while (delta > 0) delta -= TAU;
    } else {
      while (delta < 0) delta += TAU;
    }
    const full = Math.abs(Math.abs(delta) - TAU) < 1e-6;
    const sx = cx + radius * Math.cos(start);
    const sy = cy + radius * Math.sin(start);
    point(sx, sy);
    if (full) {
      const ex = cx - radius * Math.cos(start);
      const ey = cy - radius * Math.sin(start);
      const sweep = anticlockwise ? 0 : 1;
      path += ` A ${n(radius)} ${n(radius)} 0 0 ${sweep} ${n(ex)} ${n(ey)}`;
      path += ` A ${n(radius)} ${n(radius)} 0 0 ${sweep} ${n(sx)} ${n(sy)}`;
      current = [sx, sy];
      return;
    }
    const ex = cx + radius * Math.cos(end);
    const ey = cy + radius * Math.sin(end);
    const large = Math.abs(delta) > Math.PI ? 1 : 0;
    const sweep = delta >= 0 ? 1 : 0;
    path += ` A ${n(radius)} ${n(radius)} 0 ${large} ${sweep} ${n(ex)} ${n(ey)}`;
    current = [ex, ey];
  }

  function arcTo(x1, y1, x2, y2, radius) {
    const [x0, y0] = current ?? [x1, y1];
    if (radius === 0) {
      point(x1, y1);
      return;
    }
    const v1 = [x0 - x1, y0 - y1];
    const v2 = [x2 - x1, y2 - y1];
    const len1 = Math.hypot(...v1);
    const len2 = Math.hypot(...v2);
    if (len1 === 0 || len2 === 0) {
      point(x1, y1);
      return;
    }
    const u1 = [v1[0] / len1, v1[1] / len1];
    const u2 = [v2[0] / len2, v2[1] / len2];
    const cosine = Math.max(-1, Math.min(1, u1[0] * u2[0] + u1[1] * u2[1]));
    const angle = Math.acos(cosine);
    if (angle < 1e-6 || Math.PI - angle < 1e-6) {
      point(x1, y1);
      return;
    }
    const distance = radius / Math.tan(angle / 2);
    const t1 = [x1 + u1[0] * distance, y1 + u1[1] * distance];
    const t2 = [x1 + u2[0] * distance, y1 + u2[1] * distance];
    point(...t1);
    const cross = (t1[0] - x1) * (t2[1] - y1) - (t1[1] - y1) * (t2[0] - x1);
    const sweep = cross < 0 ? 1 : 0;
    path += ` A ${n(radius)} ${n(radius)} 0 0 ${sweep} ${n(t2[0])} ${n(t2[1])}`;
    current = t2;
  }

  function setFont(value) {
    font = String(value);
    const match = /^(\d+)\s+([\d.]+)px\s+(.+)$/.exec(font);
    if (match) {
      fontWeight = match[1];
      fontSize = match[2];
      fontFamily = fontOverride || match[3];
    } else if (fontOverride) {
      fontFamily = fontOverride;
    }
    if (measure) measure.font = effectiveFont(measureFamily || fontFamily);
  }

  function effectiveFont(family = fontFamily) {
    return `${fontWeight} ${fontSize}px ${family}`;
  }

  const surface = {
    get fillStyle() {
      return fillStyle;
    },
    set fillStyle(value) {
      fillStyle = value;
    },
    get strokeStyle() {
      return strokeStyle;
    },
    set strokeStyle(value) {
      strokeStyle = value;
    },
    get lineWidth() {
      return lineWidth;
    },
    set lineWidth(value) {
      lineWidth = value;
    },
    get lineCap() {
      return lineCap;
    },
    set lineCap(value) {
      lineCap = value;
    },
    get font() {
      return font;
    },
    set font(value) {
      setFont(value);
    },
    get textAlign() {
      return textAlign;
    },
    set textAlign(value) {
      textAlign = value;
    },
    get textBaseline() {
      return textBaseline;
    },
    set textBaseline(value) {
      textBaseline = value;
    },
    beginPath() {
      path = "";
      current = null;
    },
    moveTo(x, y) {
      point(x, y);
    },
    lineTo(x, y) {
      point(x, y);
    },
    arc,
    arcTo,
    closePath() {
      if (path) path += " Z";
    },
    fill() {
      if (!path) return;
      elements.push(
        `<path d="${escapeXml(path)}" ${colourAttrs(fillStyle, "fill", backdrop)} stroke="none"/>`,
      );
    },
    stroke() {
      if (!path) return;
      const dash = dashed.length ? ` stroke-dasharray="${dashed.map(n).join(" ")}"` : "";
      elements.push(
        `<path d="${escapeXml(path)}" fill="none" ${colourAttrs(strokeStyle, "stroke", backdrop)} stroke-width="${n(lineWidth)}" stroke-linecap="${escapeXml(lineCap)}"${dash}/>`,
      );
    },
    fillRect(x, y, w, h) {
      elements.push(
        `<rect x="${n(x)}" y="${n(y)}" width="${n(w)}" height="${n(h)}" ${colourAttrs(fillStyle, "fill", backdrop)}/>`,
      );
    },
    fillText(text, x, y) {
      const anchor = textAlign === "center" ? "middle" : textAlign === "right" ? "end" : "start";
      const baseline =
        textBaseline === "middle"
          ? ' dominant-baseline="central"'
          : textBaseline === "top"
            ? ' dominant-baseline="hanging"'
            : textBaseline === "bottom"
              ? ' dominant-baseline="text-after-edge"'
              : "";
      elements.push(
        `<text x="${n(x)}" y="${n(y)}" ${colourAttrs(fillStyle, "fill", backdrop)} font-family="${escapeXml(fontFamily)}" font-size="${n(Number(fontSize))}" font-weight="${escapeXml(fontWeight)}" text-anchor="${anchor}"${baseline}>${escapeXml(text)}</text>`,
      );
      return { width: measure ? measure.measureText(text).width : 0 };
    },
    measureText(text) {
      if (!measure) return { width: 0 };
      measure.font = effectiveFont(measureFamily || fontFamily);
      return measure.measureText(text);
    },
    setLineDash(values) {
      dashed = Array.isArray(values) ? values : [];
    },
    setTransform() {
      throw new Error("The SVG surface does not support canvas transforms.");
    },
    toString() {
      const style = fontData
        ? `<style>@font-face{font-family:"${fontName}";src:url(${fontData}) format("woff2");font-weight:100 900;font-style:normal;}</style>`
        : "";
      return `<svg xmlns="http://www.w3.org/2000/svg" width="${n(width * scale)}" height="${n(height * scale)}" viewBox="0 0 ${n(width)} ${n(height)}">${style}${elements.join("")}</svg>`;
    },
  };

  return surface;
}
