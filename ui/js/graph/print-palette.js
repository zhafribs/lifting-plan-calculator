// The report prints the diagram on white, but the Graph tab draws on the dark
// screen palette. The colours are inverted here, in the bitmap, rather than by a
// CSS filter on the image: a filtered image is rasterised again by the print
// pipeline (at 300 dpi), which softens a capture that is otherwise sharp.
//
// The matrix is the one CSS applies for `invert(1) hue-rotate(180deg)
// brightness(1.08)`, so the printed diagram looks exactly as it did.

const ROTATION = [
  [-0.5748, 1.4304, 0.1444],
  [0.4252, 0.4304, 0.1444],
  [0.4252, 1.4304, -0.8556],
];
const BRIGHTNESS = 1.08;

function clamp01(value) {
  return value < 0 ? 0 : value > 1 ? 1 : value;
}

function channel(row, red, green, blue) {
  const mixed = row[0] * red + row[1] * green + row[2] * blue;
  return Math.round(clamp01(clamp01(mixed) * BRIGHTNESS) * 255);
}

// The same mapping for a CSS colour, so an SVG uses exactly what the bitmap
// filter produced. Alpha is preserved.
export function printColour(value) {
  if (typeof value !== "string") return value;
  const text = value.trim();
  let red;
  let green;
  let blue;
  let alpha = null;
  const hex = /^#([0-9a-f]{6})$/i.exec(text);
  if (hex) {
    const number = Number.parseInt(hex[1], 16);
    red = (number >> 16) & 255;
    green = (number >> 8) & 255;
    blue = number & 255;
  } else {
    const rgb = /^rgba?\(\s*([\d.]+)[ ,]+([\d.]+)[ ,]+([\d.]+)(?:[ ,/]+([\d.]+))?\s*\)$/i.exec(
      text,
    );
    if (!rgb) return value;
    red = Number(rgb[1]);
    green = Number(rgb[2]);
    blue = Number(rgb[3]);
    alpha = rgb[4] === undefined ? null : Number(rgb[4]);
  }
  const source = [1 - red / 255, 1 - green / 255, 1 - blue / 255];
  const out = ROTATION.map((row) => channel(row, ...source));
  return alpha === null
    ? `rgb(${out[0]}, ${out[1]}, ${out[2]})`
    : `rgba(${out[0]}, ${out[1]}, ${out[2]}, ${alpha})`;
}

// Rewrites RGBA bytes in place; alpha is left alone.
export function printify(data) {
  for (let i = 0; i < data.length; i += 4) {
    const red = 1 - data[i] / 255;
    const green = 1 - data[i + 1] / 255;
    const blue = 1 - data[i + 2] / 255;
    data[i] = channel(ROTATION[0], red, green, blue);
    data[i + 1] = channel(ROTATION[1], red, green, blue);
    data[i + 2] = channel(ROTATION[2], red, green, blue);
  }
  return data;
}
