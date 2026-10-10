// Geometry engine for the working-range diagram.
//
// A faithful JavaScript port of the "Crane Graph" geometry module (originally
// pure Python, recovered from the old crane-graph AppImage). All lengths are
// metres. Angles are degrees measured counter-clockwise from the +X axis; the
// Y axis points up, y = 0 is ground level.
//
// The crane sits at a pivot point (crane_x, crane_y). The boom rises from the
// pivot at a boom angle. In the jib configurations a jib (or extended jib) is
// pinned to the boom tip at a fixed offset angle, so the whole assembly is a
// rigid body that rotates about the pivot; the tip always traces a circular
// arc centred on the pivot as the boom angle sweeps its envelope.

export const CONFIG_BOOM = "boom";
export const CONFIG_BOOM_JIB = "boom_jib";
export const CONFIG_BOOM_EXT_JIB = "boom_ext_jib";

export const CONFIGS = [CONFIG_BOOM, CONFIG_BOOM_JIB, CONFIG_BOOM_EXT_JIB];

export const CONFIG_LABELS = {
  [CONFIG_BOOM]: "Boom length",
  [CONFIG_BOOM_JIB]: "Boom length + jib",
  [CONFIG_BOOM_EXT_JIB]: "Boom length + extended jib",
};

// Absolute mechanical limits of the boom angle (deg from horizontal).
export const ANGLE_MIN = 0.0;
export const ANGLE_MAX = 85.0;

export const DEFAULT_STATE = {
  config: CONFIG_BOOM,
  boom_length: 30.0,
  jib_length: 12.0,
  ext_jib_length: 24.0,
  jib_offset: 0.0,
  angle: 45.0,
  env_min: 0.0,
  env_max: 80.0,
  crane_x: 0.0,
  crane_y: 0.0,
};

export class GeometryError extends Error {
  constructor(message) {
    super(message);
    this.name = "GeometryError";
  }
}

export function clamp(value, lo, hi) {
  if (lo > hi) throw new GeometryError("clamp: lower bound exceeds upper bound.");
  if (value < lo) return lo;
  if (value > hi) return hi;
  return value;
}

// Wrap an angle to the range [-180, 180). Written to match Python's `%` (the
// result keeps the sign of the divisor), unlike JavaScript's remainder.
export function normalizeAngle(angleDeg) {
  return (((angleDeg + 180.0) % 360.0) + 360.0) % 360.0 - 180.0;
}

// Jib length in metres for the selected boom configuration.
//   boom         -> 0 (no jib fitted)
//   boom_jib     -> jib_length
//   boom_ext_jib -> ext_jib_length
export function effectiveJibLength(config, jibLength, extJibLength) {
  if (config === CONFIG_BOOM) return 0.0;
  let length;
  if (config === CONFIG_BOOM_JIB) length = Number(jibLength);
  else if (config === CONFIG_BOOM_EXT_JIB) length = Number(extJibLength);
  else throw new GeometryError(`Unknown configuration: ${config}`);
  if (length < 0) throw new GeometryError("Jib length cannot be negative.");
  return length;
}

export function boomTip(pivot, boomLength, angleDeg) {
  if (boomLength < 0) throw new GeometryError("Boom length cannot be negative.");
  const a = (angleDeg * Math.PI) / 180.0;
  return [pivot[0] + boomLength * Math.cos(a), pivot[1] + boomLength * Math.sin(a)];
}

// World coordinates of the free end of the boom + jib assembly. With no jib
// this is the boom tip; otherwise it is the jib tip, the jib being pinned at
// the boom tip and offset **clockwise** from the boom axis by `jibOffsetDeg`
// (a positive offset lowers the jib below the boom line).
export function assemblyTip(pivot, boomLength, jibLength, jibOffsetDeg, angleDeg) {
  const tip = boomTip(pivot, boomLength, angleDeg);
  if (jibLength <= 0) return tip;
  const a = ((angleDeg - jibOffsetDeg) * Math.PI) / 180.0;
  return [tip[0] + jibLength * Math.cos(a), tip[1] + jibLength * Math.sin(a)];
}

// Tip position relative to the pivot at boom angle 0 (assembly frame). Because
// the boom + jib assembly is rigid, the world tip is simply this offset rotated
// by the boom angle. The jib offset is clockwise, so its vertical component is
// negative.
export function tipOffsetFromPivot(boomLength, jibLength, jibOffsetDeg) {
  if (boomLength < 0 || jibLength < 0) {
    throw new GeometryError("Boom and jib lengths cannot be negative.");
  }
  const a = (jibOffsetDeg * Math.PI) / 180.0;
  return [boomLength + jibLength * Math.cos(a), -jibLength * Math.sin(a)];
}

// Distance from pivot to the assembly tip = envelope arc radius (m).
export function envelopeRadius(boomLength, jibLength, jibOffsetDeg) {
  const [vx, vy] = tipOffsetFromPivot(boomLength, jibLength, jibOffsetDeg);
  return Math.hypot(vx, vy);
}

export function envelopePoints(pivot, radius, angleMin, angleMax, steps = 72) {
  if (radius < 0) throw new GeometryError("Envelope radius cannot be negative.");
  if (steps < 1) throw new GeometryError("Envelope needs at least one step.");
  const pts = [];
  for (let i = 0; i <= steps; i += 1) {
    const a = ((angleMin + ((angleMax - angleMin) * i) / steps) * Math.PI) / 180.0;
    pts.push([pivot[0] + radius * Math.cos(a), pivot[1] + radius * Math.sin(a)]);
  }
  return pts;
}

// Boom angle (deg) that places the assembly tip on `target`. Inverse of
// `assemblyTip`, used when the user drags the tip handle on the diagram. The
// result is normalized to [-180, 180); callers should clamp it to the limits.
export function angleForTip(pivot, boomLength, jibLength, jibOffsetDeg, target) {
  const [vx, vy] = tipOffsetFromPivot(boomLength, jibLength, jibOffsetDeg);
  if (Math.hypot(vx, vy) < 1e-9) {
    throw new GeometryError("Boom length must be greater than 0.");
  }
  const base = (Math.atan2(vy, vx) * 180.0) / Math.PI;
  const targetAng = (Math.atan2(target[1] - pivot[1], target[0] - pivot[0]) * 180.0) / Math.PI;
  return normalizeAngle(targetAng - base);
}

// Working radius (m): the tip's horizontal distance from the x = 0 axis, the
// basis the load chart's radii are measured from — not from the crane's own
// position.
export function workingRadius(tip) {
  return tip[0];
}

// Boom angle (deg) that gives the assembly a working radius of `radius`: the
// inverse of the horizontal reach, taking the upper solution (tip above the
// pivot). With no jib this reduces to acos(radius / boomLength).
export function angleForWorkingRadius(boomLength, jibLength, jibOffsetDeg, radius) {
  const [vx, vy] = tipOffsetFromPivot(boomLength, jibLength, jibOffsetDeg);
  const envelope = Math.hypot(vx, vy);
  if (envelope < 1e-9) {
    throw new GeometryError("Boom length must be greater than 0.");
  }
  if (radius < 0) {
    throw new GeometryError("Working radius cannot be negative.");
  }
  const base = (Math.atan2(vy, vx) * 180.0) / Math.PI;
  const ratio = Math.min(1, radius / envelope);
  return normalizeAngle((Math.acos(ratio) * 180.0) / Math.PI - base);
}

// Tip height (m) above ground level (y = 0).
export function tipHeight(tip) {
  return tip[1];
}

export function tipForState(state) {
  const pivot = [state.crane_x, state.crane_y];
  const jib = effectiveJibLength(state.config, state.jib_length, state.ext_jib_length);
  return assemblyTip(pivot, state.boom_length, jib, state.jib_offset, state.angle);
}

// Derived values for the readout panel: the active jib length, assembly tip
// position, working radius, heights and envelope radius.
export function readouts(state) {
  const pivot = [state.crane_x, state.crane_y];
  const jib = effectiveJibLength(state.config, state.jib_length, state.ext_jib_length);
  const tip = assemblyTip(pivot, state.boom_length, jib, state.jib_offset, state.angle);
  const boomEnd = boomTip(pivot, state.boom_length, state.angle);
  return {
    config: state.config,
    jib_active: jib > 0,
    jib_length: jib,
    pivot,
    boom_end: boomEnd,
    tip,
    working_radius: workingRadius(tip),
    tip_height: tipHeight(tip),
    boom_tip_height: tipHeight(boomEnd),
    height_above_crane: tip[1] - pivot[1],
    envelope_radius: envelopeRadius(state.boom_length, jib, state.jib_offset),
    below_ground: tip[1] < 0,
  };
}