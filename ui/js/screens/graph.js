// Graph: the crane's working-range diagram.
//
// A port of the standalone "Crane Graph" tool into this app's design system.
// The geometry engine lives in graph/geometry.js (verified against the
// original Python module); the interactive diagram is an HTML5 canvas. The
// inputs, the diagram and the readout share one state object local to this
// screen, so the tab is fully self-contained: nothing here touches the store
// or the solve cycle.

import { el } from "../dom.js";
import { card, field, kvRows, numberInput } from "../components.js";
import * as geo from "../graph/geometry.js";

const DEG = "\u00b0";
const NOTHING = "\u2014";

// Palette, matched to the original Crane Graph canvas.
const COL_BG = "#141922";
const COL_GRID = "#1f2735";
const COL_AXIS = "#5f7191";
const COL_AXIS_TEXT = "#7d8ba3";
const COL_ENV_FILL = "rgba(92,179,255,0.10)";
const COL_ENV_STROKE = "rgba(92,179,255,0.69)";
const COL_ENV_EDGE = "rgba(92,179,255,0.35)";
const COL_BOOM_ARC = "rgba(143,183,255,0.33)";
const COL_BOOM = "#ffb703";
const COL_JIB = "#5cb3ff";
const COL_JOINT = "#e8eaf0";
const COL_DIM_R = "rgba(255,183,3,0.67)";
const COL_DIM_H = "rgba(143,183,255,0.67)";
const COL_ANGLE = "rgba(223,230,240,0.59)";
const COL_PILL_BG = "rgba(16,21,30,0.88)";
const COL_TEXT = "#dfe6f0";
const COL_HANDLE = "#ffffff";

const FONT = ` Inter, "DejaVu Sans", system-ui, sans-serif`;
const TIP_HIT_R = 14;
const CRANE_HIT_R = 20;

// Python's `f"{value:g}"` for the numbers this screen handles: strip the
// decimal point when the value is a whole number, else the shortest repr.
function fmtG(value) {
  return Number.isInteger(value) ? String(Math.round(value)) : String(value);
}

function roundRect(g, x, y, w, h, r) {
  const rr = Math.min(r, w / 2, h / 2);
  g.beginPath();
  g.moveTo(x + rr, y);
  g.arcTo(x + w, y, x + w, y + h, rr);
  g.arcTo(x + w, y + h, x, y + h, rr);
  g.arcTo(x, y + h, x, y, rr);
  g.arcTo(x, y, x + w, y, rr);
  g.closePath();
}

export function createGraphScreen(ctx) {
  const store = ctx.store;

  // The screen's own document: a copy of the geometry defaults.
  const state = { ...geo.DEFAULT_STATE };
  const view = { scale: 12.0, cx: 0.0, cy: 15.0 };
  const box = { w: 0, h: 0 };
  let dragging = null; // "tip" | "crane" | "pan"
  let lastMouse = null;
  let hover = null;
  let showDims = true;
  let autoFit = true;
  let fitted = false;

  // The canvas is referenced by the transform helpers and the interaction
  // wiring below, so it must exist before either runs.
  const canvas = el("canvas", { class: "graph-canvas" });

  // --- transforms -----------------------------------------------------------

  function measureBox() {
    box.w = canvas.clientWidth || 0;
    box.h = canvas.clientHeight || 0;
    return box.w > 0 && box.h > 0;
  }

  function w2s(x, y) {
    return [(x - view.cx) * view.scale + box.w / 2, box.h / 2 - (y - view.cy) * view.scale];
  }

  function s2w(x, y) {
    return [(x - box.w / 2) / view.scale + view.cx, (box.h / 2 - y) / view.scale + view.cy];
  }

  function gridStep() {
    for (const step of [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000]) {
      if (step * view.scale >= 45.0) return step;
    }
    return 1000.0;
  }

  function visibleWorld() {
    const [x0] = s2w(0, 0);
    const [x1] = s2w(box.w, 0);
    const [, yTop] = s2w(0, 0);
    const [, yBottom] = s2w(0, box.h);
    return [Math.min(x0, x1), Math.max(x0, x1), Math.min(yBottom, yTop), Math.max(yBottom, yTop)];
  }

  function fitView() {
    if (!measureBox()) return;
    const pivot = [state.crane_x, state.crane_y];
    const jib = geo.effectiveJibLength(state.config, state.jib_length, state.ext_jib_length);
    const radius = geo.envelopeRadius(state.boom_length, jib, state.jib_offset);
    const pts = geo.envelopePoints(pivot, radius, state.env_min, state.env_max);
    pts.push(pivot);
    pts.push([pivot[0], 0.0]);
    pts.push([pivot[0] + radius, pivot[1]]);
    pts.push(geo.tipForState(state));
    let x0 = Infinity;
    let x1 = -Infinity;
    let y0 = Infinity;
    let y1 = -Infinity;
    for (const [x, y] of pts) {
      x0 = Math.min(x0, x);
      x1 = Math.max(x1, x);
      y0 = Math.min(y0, y);
      y1 = Math.max(y1, y);
    }
    const pad = 0.1 * Math.max(x1 - x0, y1 - y0, 1.0) + 1.5;
    x0 -= pad;
    x1 += pad;
    y0 -= pad;
    y1 += pad;
    view.scale = geo.clamp(Math.min(box.w / (x1 - x0), box.h / (y1 - y0)), 0.02, 2000.0);
    view.cx = (x0 + x1) / 2.0;
    view.cy = (y0 + y1) / 2.0;
  }

  // --- drawing ---------------------------------------------------------------

  function pill(g, cx, cy, text, colour, size) {
    g.font = `700 ${size}px${FONT}`;
    const tw = g.measureText(text).width;
    const th = size + 2;
    let x = cx - tw / 2 - 6;
    let y = cy - th / 2 - 3;
    const w = tw + 12;
    const h = th + 6;
    x = geo.clamp(x, 2, Math.max(2, box.w - w - 2));
    y = geo.clamp(y, 2, Math.max(2, box.h - h - 2));
    roundRect(g, x, y, w, h, 4);
    g.fillStyle = COL_PILL_BG;
    g.fill();
    g.fillStyle = colour;
    g.textAlign = "center";
    g.textBaseline = "middle";
    g.fillText(text, x + w / 2, y + h / 2);
  }

  function drawGrid(g) {
    const step = gridStep();
    const [x0, x1, y0, y1] = visibleWorld();
    g.strokeStyle = COL_GRID;
    g.lineWidth = 1;
    g.beginPath();
    for (let xs = Math.floor(x0 / step) * step; xs <= x1; xs += step) {
      if (Math.abs(xs) > step * 1e-6) {
        const [sx] = w2s(xs, 0);
        g.moveTo(sx, 0);
        g.lineTo(sx, box.h);
      }
    }
    for (let ys = Math.floor(y0 / step) * step; ys <= y1; ys += step) {
      if (Math.abs(ys) > step * 1e-6) {
        const [, sy] = w2s(0, ys);
        g.moveTo(0, sy);
        g.lineTo(box.w, sy);
      }
    }
    g.stroke();
  }

  function fmtTick(value) {
    return value === Math.round(value) ? String(Math.round(value)) : String(value);
  }

  function drawAxesGround(g) {
    const step = gridStep();
    const [x0, x1, y0, y1] = visibleWorld();
    g.font = `10px${FONT}`;
    g.strokeStyle = COL_AXIS;
    g.lineWidth = 1;
    const gy = w2s(0, 0)[1];

    // ground line (y = 0) with engineering hatch below it
    if (gy >= -2 && gy <= box.h + 2) {
      g.lineWidth = 2;
      g.beginPath();
      g.moveTo(0, gy);
      g.lineTo(box.w, gy);
      g.stroke();
      g.lineWidth = 1;
      g.beginPath();
      for (let xs = 0; xs <= box.w; xs += 11) {
        g.moveTo(xs, gy + 1);
        g.lineTo(xs - 7, gy + 8);
      }
      g.stroke();
      g.fillStyle = COL_AXIS_TEXT;
      g.textAlign = "left";
      g.textBaseline = "alphabetic";
      for (let xs = Math.floor(x0 / step) * step; xs <= x1; xs += step) {
        const [sx] = w2s(xs, 0);
        if (sx >= 14 && sx <= box.w - 14) g.fillText(fmtTick(xs), sx + 3, gy + 20);
      }
    }

    // y axis (x = 0) with labels
    const axX = w2s(0, 0)[0];
    const labelX = axX >= -2 && axX <= box.w + 2 ? axX - 6 : 8.0;
    if (axX >= -2 && axX <= box.w + 2) {
      g.lineWidth = 1;
      g.beginPath();
      g.moveTo(axX, 0);
      g.lineTo(axX, box.h);
      g.stroke();
    }
    g.fillStyle = COL_AXIS_TEXT;
    g.textAlign = "right";
    g.textBaseline = "middle";
    for (let ys = Math.floor(y0 / step) * step; ys <= y1; ys += step) {
      const [, sy] = w2s(0, ys);
      if (sy >= 10 && sy <= box.h - 10) g.fillText(fmtTick(ys), labelX, sy);
    }

    // axis captions
    g.textAlign = "right";
    g.fillText("X / radius (m)", box.w - 8, box.h - 15);
    g.textAlign = "left";
    g.fillText("Y / height (m)", 8, 15);
  }

  function drawEnvelope(g) {
    const pivot = [state.crane_x, state.crane_y];
    const jib = geo.effectiveJibLength(state.config, state.jib_length, state.ext_jib_length);
    const radius = geo.envelopeRadius(state.boom_length, jib, state.jib_offset);
    const pts = geo
      .envelopePoints(pivot, radius, state.env_min, state.env_max, 72)
      .map(([x, y]) => w2s(x, y));
    const [px, py] = w2s(pivot[0], pivot[1]);

    if (pts.length >= 2) {
      // sector fill
      g.beginPath();
      g.moveTo(px, py);
      for (const [sx, sy] of pts) g.lineTo(sx, sy);
      g.closePath();
      g.fillStyle = COL_ENV_FILL;
      g.fill();

      // arc stroke
      g.strokeStyle = COL_ENV_STROKE;
      g.lineWidth = 2;
      g.beginPath();
      g.moveTo(pts[0][0], pts[0][1]);
      for (let i = 1; i < pts.length; i += 1) g.lineTo(pts[i][0], pts[i][1]);
      g.stroke();

      // edge (radius) lines
      g.setLineDash([7, 5]);
      g.strokeStyle = COL_ENV_EDGE;
      g.lineWidth = 1;
      g.beginPath();
      g.moveTo(px, py);
      g.lineTo(pts[0][0], pts[0][1]);
      g.stroke();
      g.beginPath();
      g.moveTo(px, py);
      g.lineTo(pts[pts.length - 1][0], pts[pts.length - 1][1]);
      g.stroke();
      g.setLineDash([]);
    }

    // inner arc traced by the boom tip when a jib is fitted
    if (jib > 0) {
      const arc = geo
        .envelopePoints(pivot, state.boom_length, state.env_min, state.env_max, 48)
        .map(([x, y]) => w2s(x, y));
      g.setLineDash([7, 5]);
      g.strokeStyle = COL_BOOM_ARC;
      g.lineWidth = 1;
      g.beginPath();
      g.moveTo(arc[0][0], arc[0][1]);
      for (let i = 1; i < arc.length; i += 1) g.lineTo(arc[i][0], arc[i][1]);
      g.stroke();
      g.setLineDash([]);
    }

    // envelope angle labels at the arc ends
    g.font = `10px${FONT}`;
    g.fillStyle = COL_JIB;
    g.textAlign = "left";
    g.textBaseline = "alphabetic";
    for (const item of [
      [pts[0], state.env_min],
      [pts[pts.length - 1], state.env_max],
    ]) {
      const [end, angle] = item;
      const vx = end[0] - px;
      const vy = end[1] - py;
      const n = Math.hypot(vx, vy) || 1.0;
      g.fillText(`${fmtG(angle)}${DEG}`, end[0] + (vx / n) * 14 - 14, end[1] + (vy / n) * 14 + 4);
    }
  }

  function drawDimensions(g) {
    const tip = geo.tipForState(state);
    const [px, py] = w2s(state.crane_x, state.crane_y);
    const [tx, ty] = w2s(tip[0], tip[1]);
    const gy = w2s(0, 0)[1];

    // working radius: horizontal dashed line from pivot to tip X
    g.setLineDash([7, 5]);
    g.strokeStyle = COL_DIM_R;
    g.lineWidth = 1;
    g.beginPath();
    g.moveTo(px, py);
    g.lineTo(tx, py);
    g.stroke();
    g.setLineDash([]);
    g.lineWidth = 1.5;
    g.beginPath();
    g.moveTo(tx, py - 5);
    g.lineTo(tx, py + 5);
    g.stroke();
    const radius = geo.workingRadius([state.crane_x, state.crane_y], tip);
    pill(g, (px + tx) / 2.0, py - 12, `R = ${radius.toFixed(2)} m`, COL_BOOM, 10);

    // tip height: dashed line from ground up to the tip
    if (gy >= -50 && gy <= box.h + 50) {
      g.setLineDash([7, 5]);
      g.strokeStyle = COL_DIM_H;
      g.lineWidth = 1;
      g.beginPath();
      g.moveTo(tx, ty);
      g.lineTo(tx, gy);
      g.stroke();
      g.setLineDash([]);
      g.lineWidth = 1.5;
      g.beginPath();
      g.moveTo(tx - 5, gy);
      g.lineTo(tx + 5, gy);
      g.stroke();
      const height = tip[1];
      const colour = height < 0 ? "#ff6b6b" : COL_JIB;
      pill(g, tx + 8, (ty + gy) / 2.0, `H = ${height.toFixed(2)} m`, colour, 10);
    }
  }

  function drawAngleArc(g) {
    const [px, py] = w2s(state.crane_x, state.crane_y);
    const radius = 30.0;
    const angle = state.angle;
    const steps = Math.max(2, Math.floor(Math.abs(angle) / 2) + 1);
    g.strokeStyle = COL_ANGLE;
    g.lineWidth = 1;
    g.beginPath();
    for (let i = 0; i <= steps; i += 1) {
      const a = ((angle * i) / steps) * (Math.PI / 180);
      const x = px + radius * Math.cos(a);
      const y = py - radius * Math.sin(a);
      if (i === 0) g.moveTo(x, y);
      else g.lineTo(x, y);
    }
    g.stroke();
    g.font = `700 9px${FONT}`;
    g.fillStyle = COL_TEXT;
    const mid = (angle / 2.0) * (Math.PI / 180);
    g.textAlign = "left";
    g.textBaseline = "alphabetic";
    g.fillText(
      `${fmtG(angle)}${DEG}`,
      px + (radius + 14) * Math.cos(mid) - 12,
      py - (radius + 14) * Math.sin(mid) + 4,
    );
  }

  function drawCrane(g) {
    const [px, py] = w2s(state.crane_x, state.crane_y);
    const hot = hover === "crane" || dragging === "crane";
    const border = hot ? COL_HANDLE : COL_JIB;

    // outrigger base bar + feet
    g.lineWidth = 1;
    g.strokeStyle = COL_AXIS;
    g.fillStyle = "#34405a";
    roundRect(g, px - 22, py + 4, 44, 8, 3);
    g.fill();
    g.stroke();
    g.fillStyle = "#5f7191";
    roundRect(g, px - 24, py + 11, 9, 6, 2);
    g.fill();
    g.stroke();
    roundRect(g, px + 15, py + 11, 9, 6, 2);
    g.fill();
    g.stroke();

    // superstructure (behind the boom = -X side) + counterweight
    g.strokeStyle = border;
    g.lineWidth = hot ? 2 : 1;
    g.fillStyle = "#2f6fbf";
    roundRect(g, px - 16, py - 16, 15, 16, 3);
    g.fill();
    g.stroke();
    g.fillStyle = "#233047";
    roundRect(g, px - 23, py - 11, 7, 11, 2);
    g.fill();
    g.stroke();

    // slew / pivot
    g.fillStyle = COL_JOINT;
    g.strokeStyle = "#141922";
    g.lineWidth = 1;
    g.beginPath();
    g.arc(px, py, 4, 0, Math.PI * 2);
    g.fill();
    g.stroke();

    pill(g, px, py + 30, `(${fmtG(state.crane_x)}, ${fmtG(state.crane_y)})`, COL_TEXT, 9);
  }

  function drawBoom(g) {
    const pivot = [state.crane_x, state.crane_y];
    const jib = geo.effectiveJibLength(state.config, state.jib_length, state.ext_jib_length);
    const boomEnd = geo.boomTip(pivot, state.boom_length, state.angle);
    const tip = geo.assemblyTip(pivot, state.boom_length, jib, state.jib_offset, state.angle);
    const [px, py] = w2s(pivot[0], pivot[1]);
    const [bx, by] = w2s(boomEnd[0], boomEnd[1]);
    const [tx, ty] = w2s(tip[0], tip[1]);

    g.lineCap = "round";
    g.strokeStyle = COL_BOOM;
    g.lineWidth = 5;
    g.beginPath();
    g.moveTo(px, py);
    g.lineTo(bx, by);
    g.stroke();

    if (jib > 0) {
      g.strokeStyle = COL_JIB;
      g.lineWidth = 3;
      g.beginPath();
      g.moveTo(bx, by);
      g.lineTo(tx, ty);
      g.stroke();
      g.fillStyle = COL_JOINT;
      g.strokeStyle = "#141922";
      g.lineWidth = 1;
      g.beginPath();
      g.arc(bx, by, 4, 0, Math.PI * 2);
      g.fill();
      g.stroke();
    }
    g.lineCap = "butt";
  }

  function drawHandles(g) {
    const tip = geo.tipForState(state);
    const [tx, ty] = w2s(tip[0], tip[1]);

    // Tip location on the diagram itself (always visible), in the same style
    // as the crane location pill, matching the readout's 2-decimal precision.
    pill(g, tx, ty + 24, `(${tip[0].toFixed(2)}, ${tip[1].toFixed(2)})`, COL_TEXT, 9);

    const hot = hover === "tip" || dragging === "tip";
    const radius = hot ? 9 : 7;
    g.fillStyle = COL_BOOM;
    g.strokeStyle = hot ? COL_HANDLE : "#141922";
    g.lineWidth = 2;
    g.beginPath();
    g.arc(tx, ty, radius, 0, Math.PI * 2);
    g.fill();
    g.stroke();
    if (hot) {
      g.font = `700 9px${FONT}`;
      g.fillStyle = COL_TEXT;
      g.textAlign = "left";
      g.textBaseline = "alphabetic";
      g.fillText(`(${tip[0].toFixed(1)}, ${tip[1].toFixed(1)})`, tx + 14, ty + 4);
    }
  }

  function redraw() {
    if (!measureBox()) return;
    const dpr = window.devicePixelRatio || 1;
    const bw = Math.round(box.w * dpr);
    const bh = Math.round(box.h * dpr);
    if (canvas.width !== bw) canvas.width = bw;
    if (canvas.height !== bh) canvas.height = bh;
    const g = canvas.getContext("2d");
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.fillStyle = COL_BG;
    g.fillRect(0, 0, box.w, box.h);
    drawGrid(g);
    drawAxesGround(g);
    drawEnvelope(g);
    if (showDims) drawDimensions(g);
    drawAngleArc(g);
    drawCrane(g);
    drawBoom(g);
    drawHandles(g);
  }

  // --- interaction -----------------------------------------------------------

  function canvasPos(event) {
    const rect = canvas.getBoundingClientRect();
    return { x: event.clientX - rect.left, y: event.clientY - rect.top };
  }

  function hitTest(pos) {
    measureBox();
    const tip = geo.tipForState(state);
    const [tx, ty] = w2s(tip[0], tip[1]);
    const dx = pos.x - tx;
    const dy = pos.y - ty;
    if (dx * dx + dy * dy <= TIP_HIT_R * TIP_HIT_R) return "tip";
    const [px, py] = w2s(state.crane_x, state.crane_y);
    const cx = pos.x - px;
    const cy = pos.y - py;
    if (cx * cx + cy * cy <= CRANE_HIT_R * CRANE_HIT_R) return "crane";
    return null;
  }

  function syncCursor() {
    canvas.style.cursor = dragging ? "grabbing" : "grab";
  }

  function updateHover(pos) {
    const hit = hitTest(pos);
    let next = null;
    if (hit === "tip") next = "tip";
    else if (hit === "crane") next = "crane";
    if (next !== hover) {
      hover = next;
      syncCursor();
      redraw();
    }
  }

  function emitTip(pos) {
    const [wx, wy] = s2w(pos.x, pos.y);
    const jib = geo.effectiveJibLength(state.config, state.jib_length, state.ext_jib_length);
    try {
      const angle = Math.round(
        geo.clamp(
          geo.angleForTip([state.crane_x, state.crane_y], state.boom_length, jib, state.jib_offset, [wx, wy]),
          geo.ANGLE_MIN,
          geo.ANGLE_MAX,
        ) * 10,
      ) / 10;
      if (angle !== state.angle) apply({ angle }, { user: true, fit: false });
    } catch (error) {
      /* dragging into an impossible region: keep the last good angle */
    }
  }

  function emitCrane(pos) {
    const [wx, wy] = s2w(pos.x, pos.y);
    const x = Math.round(geo.clamp(wx, -100.0, 100.0) * 10) / 10;
    const y = Math.round(geo.clamp(wy, -50.0, 100.0) * 10) / 10;
    // The view is intentionally left alone while dragging (the marker must
    // track the cursor); interactionEnded re-fits on release, exactly like
    // the tip drag.
    if (x !== state.crane_x || y !== state.crane_y) apply({ crane_x: x, crane_y: y }, { user: true, fit: false });
  }

  canvas.addEventListener("mousedown", (event) => {
    if (event.button !== 0) return;
    const pos = canvasPos(event);
    const hit = hitTest(pos);
    if (hit === "tip") {
      dragging = "tip";
      emitTip(pos);
    } else if (hit === "crane") {
      dragging = "crane";
      emitCrane(pos);
    } else {
      dragging = "pan";
      lastMouse = pos;
    }
    syncCursor();
    event.preventDefault();
  });

  canvas.addEventListener("mousemove", (event) => {
    const pos = canvasPos(event);
    if (dragging === "tip") emitTip(pos);
    else if (dragging === "crane") emitCrane(pos);
    else if (dragging === "pan") {
      const dx = pos.x - lastMouse.x;
      const dy = pos.y - lastMouse.y;
      view.cx -= dx / view.scale;
      view.cy += dy / view.scale;
      lastMouse = pos;
      redraw();
    } else {
      updateHover(pos);
    }
  });

  canvas.addEventListener("mouseup", (event) => {
    if (event.button !== 0) return;
    if (dragging === "tip" || dragging === "crane") {
      if (autoFit) fitView();
    }
    dragging = null;
    updateHover(canvasPos(event));
    syncCursor();
  });

  canvas.addEventListener("mouseleave", () => {
    if (hover !== null) {
      hover = null;
      syncCursor();
      redraw();
    }
  });

  canvas.addEventListener("dblclick", (event) => {
    if (dragging !== null) return;
    fitView();
    redraw();
    event.preventDefault();
  });

  canvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      const steps = -event.deltaY / 100;
      if (steps === 0) return;
      const factor = 1.25 ** steps;
      const pos = canvasPos(event);
      const [wx, wy] = s2w(pos.x, pos.y);
      view.scale = geo.clamp(view.scale * factor, 0.02, 2000.0);
      view.cx = wx - (pos.x - box.w / 2) / view.scale;
      view.cy = wy - (box.h / 2 - pos.y) / view.scale;
      redraw();
    },
    { passive: false },
  );

  // Resizing the canvas bitmap inside an observer delivery counts as an
  // undelivered notification on WebKitGTK ("ResizeObserver loop completed with
  // undelivered notifications.") and the app's global error handler turns that
  // into an error banner. The split-screen graph canvas is a `height: auto`
  // flex child, so setting the bitmap in redraw() really does resize the
  // element. Deferring to the next animation frame moves the resize out of the
  // delivery cycle and lets the observer settle cleanly.
  let resizeFrame = 0;
  new ResizeObserver(() => {
    cancelAnimationFrame(resizeFrame);
    resizeFrame = requestAnimationFrame(() => {
      measureBox();
      redraw();
    });
  }).observe(canvas);

  // --- state flow ------------------------------------------------------------

  function syncEnabled() {
    const jibOn = state.config !== geo.CONFIG_BOOM;
    const extOn = state.config === geo.CONFIG_BOOM_EXT_JIB;
    setEnabled(fieldNodes.jib_length, jibOn);
    setEnabled(fieldNodes.jib_offset, jibOn);
    setEnabled(fieldNodes.ext_jib_length, extOn);
    for (const [key, radio] of Object.entries(configRadios)) {
      radio.checked = key === state.config;
    }
  }

  function setEnabled(node, on) {
    node.classList.toggle("disabled", !on);
    const input = node.querySelector("input");
    if (input) input.disabled = !on;
  }

  function reflectInputs() {
    for (const key of Object.keys(numInputs)) {
      const input = numInputs[key];
      if (input && document.activeElement !== input) input.value = String(state[key]);
    }
  }

  function updateReadout() {
    const rd = geo.readouts(state);
    const jibActive = rd.jib_active;
    const jibLabel =
      state.config === geo.CONFIG_BOOM_EXT_JIB ? "Extended jib length" : "Jib length";
    readoutBox.replaceChildren(
      kvRows([
        ["Configuration", geo.CONFIG_LABELS[state.config]],
        ["Boom length", `${state.boom_length.toFixed(1)} m`],
        [jibLabel, jibActive ? `${rd.jib_length.toFixed(1)} m` : "not fitted"],
        ["Jib offset angle", jibActive ? `${state.jib_offset.toFixed(1)} deg` : NOTHING],
        ["Working radius", `${rd.working_radius.toFixed(2)} m`],
        ["Tip height above ground", `${rd.tip_height.toFixed(2)} m`],
        ["Tip height above crane", `${rd.height_above_crane.toFixed(2)} m`],
        ["Envelope radius", `${rd.envelope_radius.toFixed(2)} m`],
        ["Envelope sweep", `${fmtG(state.env_min)} - ${fmtG(state.env_max)} deg`],
        ["Boom angle", `${fmtG(state.angle)} deg`],
        ["Crane location (X, Y)", `(${fmtG(state.crane_x)}, ${fmtG(state.crane_y)})`],
        ["Tip location (X, Y)", `(${rd.tip[0].toFixed(2)}, ${rd.tip[1].toFixed(2)})`],
        rd.below_ground
          ? ["Ground clearance", "BELOW GROUND", "over"]
          : ["Ground clearance", "OK", "ok"],
      ]),
    );
  }

  function apply(patch, opts = {}) {
    if (opts.user && workbookMode) deviated = true;
    Object.assign(state, patch);
    syncEnabled();
    if (autoFit && opts.fit !== false && fitted) fitView();
    redraw();
    updateReadout();
    reflectInputs();
    if (opts.user) {
      renderNote();
      publishAngle();
    }
  }

  function setFromForm(key, raw, min, max) {
    const value = Math.min(max, Math.max(min, Number.isFinite(raw) ? raw : 0));
    const patch = { [key]: value };
    if (key === "env_min" && value > state.env_max) patch.env_max = value;
    if (key === "env_max" && value < state.env_min) patch.env_min = value;
    apply(patch, key === "angle" ? { user: true, fit: false } : { user: true });
  }

  // --- the workbook link -----------------------------------------------------
  // When a new-format workbook is loaded in the Crane tab, the diagram follows
  // it: the crane position comes from the workbook, the boom length and the
  // working radius from the Crane tab, and the boom angle drawn here is
  // published back for the Crane tab (and its jib chart) to read. A manual
  // edit on this screen marks a deviation and offers the reset button in
  // place of the card's note; the next Crane tab edit re-syncs.
  let workbookKey = null; // the crane-tab source last applied to the diagram
  let workbookMode = false;
  let deviated = false;

  const noteBox = el("span", { class: "note", text: "Drawn to scale, metres" });

  function workbookSource() {
    const crane = store.state.crane;
    if (!crane || crane.chart_source !== "excel" || !crane.chart_position) return null;
    return {
      x: crane.chart_position[0],
      y: crane.chart_position[1],
      boom: crane.chart_boom ?? 0,
      radius: crane.working_radius ?? 0,
      config: crane.jib_config || "boom",
      jib: crane.jib_length ?? 0,
      offset: crane.jib_offset ?? 0,
    };
  }

  function applyWorkbook(src, fit) {
    const active = src.config === geo.CONFIG_BOOM_JIB || src.config === geo.CONFIG_BOOM_EXT_JIB;
    const jib = active ? src.jib : 0;
    const offset = active ? src.offset : 0;
    const patch = {
      config: geo.CONFIGS.includes(src.config) ? src.config : geo.CONFIG_BOOM,
      crane_x: src.x,
      crane_y: src.y,
      boom_length: src.boom > 0 ? src.boom : state.boom_length,
    };
    if (patch.config === geo.CONFIG_BOOM_JIB) patch.jib_length = jib;
    if (patch.config === geo.CONFIG_BOOM_EXT_JIB) patch.ext_jib_length = jib;
    patch.jib_offset = offset;
    if (src.radius > 0 && patch.boom_length > 0) {
      try {
        const angle = geo.angleForWorkingRadius(patch.boom_length, jib, offset, src.radius);
        // Two decimals, the resolution the crane tab and report print.
        patch.angle = Math.round(geo.clamp(angle, geo.ANGLE_MIN, geo.ANGLE_MAX) * 100) / 100;
      } catch (error) {
        /* the geometry cannot solve a radius: leave the angle as it is */
      }
    }
    apply(patch, { fit: Boolean(fit), user: false });
  }

  function renderNote() {
    if (workbookMode && deviated) {
      noteBox.replaceChildren(
        el(
          "button",
          {
            class: "btn ghost small",
            title: "Restore this diagram to the Crane tab's position, boom length and angle",
            onClick: resetToCrane,
          },
          "Reset to Crane Tab",
        ),
      );
    } else {
      noteBox.textContent = "Drawn to scale, metres";
    }
  }

  function resetToCrane() {
    workbookKey = null;
    deviated = false;
    syncFromWorkbook();
  }

  function syncFromWorkbook() {
    const src = workbookSource();
    if (!src || !(src.boom > 0)) {
      workbookMode = false;
      workbookKey = null;
      deviated = false;
      renderNote();
      return;
    }
    workbookMode = true;
    const key = JSON.stringify(src);
    const changed = key !== workbookKey;
    if (changed || !deviated) {
      workbookKey = key;
      applyWorkbook(src, changed && autoFit);
      deviated = false;
    }
    renderNote();
    publishAngle();
  }

  // The angle the graph draws is the crane tab's "Boom angle" figure. Writing
  // it back only when it actually differs keeps the solve cycle from looping,
  // and only a live working radius publishes: with no radius there is no
  // solved angle to show.
  function publishAngle() {
    if (!workbookMode) return;
    const crane = store.state.crane;
    if (!((crane.working_radius ?? 0) > 0)) return;
    const angle = state.angle;
    if (!(angle > 0)) return;
    const row = (crane.rows && crane.rows[0]) || null;
    const anglePublished =
      crane.boom_angle_deg !== null &&
      crane.boom_angle_deg !== undefined &&
      Math.abs(crane.boom_angle_deg - angle) < 1e-9;
    const rowMirrored = !row || Math.abs((row.angle || 0) - angle) < 1e-9;
    if (anglePublished && rowMirrored) return;
    store.update((draft) => {
      draft.crane.boom_angle_deg = angle;
      if (draft.crane.rows[0]) draft.crane.rows[0].angle = angle;
    });
  }

  // --- inputs ----------------------------------------------------------------

  const readoutBox = el("div", {});
  const numInputs = {};
  const fieldNodes = {};
  const configRadios = {};

  function makeNumber(key, label, unit, min, max) {
    const input = numberInput({
      value: state[key],
      onInput: (value) => setFromForm(key, value ?? 0, min, max),
    });
    numInputs[key] = input;
    const node = field({ label, unit, input });
    fieldNodes[key] = node;
    return node;
  }

  function radioOption(config) {
    const radio = el("input", {
      type: "radio",
      name: "graph-config",
      onChange: () => {
        // With a workbook loaded the configuration belongs to the crane tab,
        // so the change writes through to the shared state. `change` (not
        // `click`) so keyboard selection of the radio group works too.
        if (workbookMode) store.chooseJib({ config });
        apply({ config });
      },
    });
    configRadios[config] = radio;
    if (config === geo.CONFIG_BOOM) radio.checked = true;
    return el(
      "label",
      { class: "check", title: `Use: ${geo.CONFIG_LABELS[config]}` },
      radio,
      el("span", { class: "t", text: geo.CONFIG_LABELS[config] }),
    );
  }

  function checkboxOption(key, label, initial) {
    const input = el("input", {
      type: "checkbox",
      onClick: () => {
        if (key === "dims") {
          showDims = input.checked;
          redraw();
        } else {
          autoFit = input.checked;
        }
      },
    });
    input.checked = initial;
    return el("label", { class: "check" }, input, el("span", { class: "t", text: label }));
  }

  const configCard = card({
    title: "Boom configuration",
    body: el(
      "div",
      { class: "stack-8" },
      el("div", { class: "checks" }, ...geo.CONFIGS.map(radioOption)),
      el(
        "div",
        { class: "fields" },
        makeNumber("boom_length", "Boom length", "m", 1, 150),
        makeNumber("jib_length", "Jib length", "m", 0, 60),
        makeNumber("ext_jib_length", "Extended jib length", "m", 0, 80),
        makeNumber("jib_offset", "Jib offset angle", DEG, -30, 60),
      ),
    ),
  });

  const angleCard = card({
    title: "Boom angle & envelope",
    body: el(
      "div",
      { class: "stack-8" },
      el(
        "div",
        { class: "fields" },
        makeNumber("angle", "Boom angle", DEG, geo.ANGLE_MIN, geo.ANGLE_MAX),
        makeNumber("env_min", "Envelope from", DEG, 0, 85),
        makeNumber("env_max", "Envelope to", DEG, 0, 85),
      ),
      el(
        "div",
        { class: "row wrap" },
        checkboxOption("dims", "Show dimensions", true),
        checkboxOption("auto", "Auto-fit view", true),
        el("span", { class: "spacer" }),
        el(
          "button",
          {
            class: "btn ghost small",
            onClick: () => {
              fitView();
              redraw();
            },
          },
          "Fit view",
        ),
      ),
    ),
  });

  const craneCard = card({
    title: "Crane location on axis (X, Y)",
    body: el(
      "div",
      { class: "stack-8" },
      el(
        "div",
        { class: "fields" },
        makeNumber("crane_x", "Position X", "m", -100, 100),
        makeNumber("crane_y", "Position Y", "m", -50, 100),
      ),
      el(
        "div",
        { class: "row wrap" },
        el("span", { class: "faint small", text: "Quick:" }),
        ...[
          [-3, 0],
          [0, 0],
          [0, 3],
          [3, 0],
        ].map(([x, y]) =>
          el(
            "button",
            { class: "btn ghost small", onClick: () => apply({ crane_x: x, crane_y: y }) },
            `(${fmtG(x)}, ${fmtG(y)})`,
          ),
        ),
      ),
      el("p", {
        class: "faint small",
        style: { margin: "2px 0 0" },
        text: "Or drag the crane marker on the diagram.",
      }),
    ),
  });

  const readoutCard = card({ title: "Readout", body: readoutBox });

  const root = el(
    "div",
    { class: "page graph-page" },
    el(
      "div",
      { class: "graph-layout" },
      el(
        "div",
        { class: "stack pane graph-pane" },
        card({
          title: "Working range diagram",
          note: noteBox,
          body: el("div", { class: "graph-canvas-wrap" }, canvas),
          foot: el("span", {
            class: "faint small",
            text:
              "Drag the boom tip to set the angle; drag the crane marker to set (X, Y); " +
              "the wheel zooms; double-click fits the view.",
          }),
        }),
      ),
      el("div", { class: "stack pane" }, configCard, angleCard, craneCard, readoutCard),
    ),
  );

  // Handles to the live state for the verification harness (and future
  // automation): screen coordinates for the constructs the user can grab.
  canvas.__graph = {
    state: () => ({ ...state }),
    view: () => ({ ...view }),
    worldToScreen: (x, y) => w2s(x, y),
    tipScreen: () => w2s(...geo.tipForState(state)),
    pivotScreen: () => w2s(state.crane_x, state.crane_y),
  };

  syncEnabled();
  updateReadout();

  // The screen is re-rendered each time it is opened (and never rebuilt), so
  // first show fits the view; later shows just repaint what the user left.
  function sync() {
    if (!measureBox()) return;
    if (!fitted) {
      fitted = true;
      fitView();
    }
    syncFromWorkbook();
    redraw();
  }

  return { root, sync, render: () => {} };
}