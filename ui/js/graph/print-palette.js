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
