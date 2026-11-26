// The working-range diagram's colours, per scheme.
//
// The dark set is the palette the chart has always had. The light set is the
// same colours through the print filter, so the screen in a light scheme matches
// the report's chart. One entry is screen-only: the warning red, darkened so the
// below-ground label stays readable (see below). The print capture always uses
// the dark set: the filter is applied there, and applying it twice would wash
// the page out.

import { printColour } from "./print-palette.js";

export const DIAGRAM_DARK = {
  bg: "#141922",
  grid: "#1f2735",
  axis: "#5f7191",
  axisText: "#7d8ba3",
  envFill: "rgba(92,179,255,0.10)",
  envStroke: "rgba(92,179,255,0.69)",
  envEdge: "rgba(92,179,255,0.35)",
  boomArc: "rgba(143,183,255,0.33)",
  boom: "#ffb703",
  jib: "#5cb3ff",
  joint: "#e8eaf0",
  dimR: "rgba(255,183,3,0.67)",
  dimH: "rgba(143,183,255,0.67)",
  angle: "rgba(223,230,240,0.59)",
  pillBg: "rgba(16,21,30,0.88)",
  text: "#dfe6f0",
  handle: "#ffffff",
  craneBase: "#34405a",
  craneBody: "#2f6fbf",
  craneWeight: "#233047",
  danger: "#ff6b6b",
};

// The print capture always starts here, whatever the screen is showing: the
// filter is what turns it into the page's light chart, and filtering twice
// would wash the page out.
export const PRINT_PALETTE = DIAGRAM_DARK;

// The light set is the printed chart, not a second design: some of its greys
// (grid, axis ticks) are deliberately faint, below the 4.5:1 text floor, because
// they are the page's own values and the screen is meant to match the page.
/** The dark palette through the print filter: the light scheme's chart. */
export const DIAGRAM_LIGHT = {
  ...Object.fromEntries(
    Object.entries(DIAGRAM_DARK).map(([key, value]) => [key, printColour(value)]),
  ),
  // The one screen value that leaves the print palette: the printed warning red
  // is 3:1 on the light page, and the below-ground label has to be read. The
  // app's light warning red keeps it at about 6.6:1 on the pill.
  danger: "#a33333",
};

export function diagramPalette(scheme) {
  return scheme === "light" ? DIAGRAM_LIGHT : DIAGRAM_DARK;
}
