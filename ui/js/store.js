// The single shared document, and the solve cycle on top of it.
//
// One state object is the whole app: the load lines, the crane form, the sling
// form and the two nonuniform scenarios. Every edit asks the engine to
// re-solve everything, and then tells the screens. Nothing is stored between
// launches: each run starts from a blank sheet.

import { debounce } from "./dom.js";
import { api } from "./api.js";

let idCounter = Date.now() % 1000000;

export function newId() {
  idCounter += 1;
  return idCounter;
}

export function blankLoadItem() {
  return {
    id: newId(),
    name: "",
    load_type: "fixed",
    qty: 1,
    weight_kg: 0,
    kg_per_metre: 0,
    length_m: 0,
    volume_m3: 0,
    density_kg_m3: 0,
  };
}

export function blankTackleItem() {
  return { id: newId(), description: "", qty: 1, weight_kg: 0 };
}

export function blankChartRow() {
  return { radius: 0, capacity: 0, angle: 0, boom: 0 };
}

function defaultCrane() {
  return {
    name: "",
    capacity_t: 0,
    rows: [blankChartRow()],
    chart_points: [],
    working_radius: 0,
    chart_source: "",
    chart_name: "",
    chart_label: "",
    chart_booms: [],
    chart_boom: null,
    chart_radii: [],
    chart_position: null,
    chart_jibs: [],
    jib_config: "boom",
    jib_length: null,
    jib_offset: null,
    chart_jib_points: [],
    boom_angle_deg: null,
  };
}

function defaultSling() {
  return {
    hitch: "direct_hook",
    sling_length: 0,
    diameter: 0,
    side: 0,
    bottom: 0,
    choke_angle: 60,
    angle_basis: "internal",
    tag_wll: 0,
    tag_wll_angle: 45,
    pick_distance: 0,
    pick_length: 0,
    pick_width: 0,
    secondary_length: 0,
    secondary_spread: 0,
    secondary_tag_wll: 0,
    secondary_tag_wll_angle: 45,
  };
}

function defaultNonuniform(scenario) {
  return {
    scenario,
    sling_length: 0,
    pick_distance: 0,
    cog_from_pick1: 0,
    pick_point2_height: 0,
    base_reeve_factor: 1,
    tag_wll1: 0,
    tag_wll1_angle: 45,
    sling_legs1: 1,
    tag_wll2: 0,
    tag_wll2_angle: 45,
    sling_legs2: 1,
    dist_a: 0,
    dist_b: 0,
    cog_height: 0,
    tilt_angle: 0,
    tag_wll_a: 0,
    tag_wll_a_angle: 45,
    sling_legs_a: 1,
    tag_wll_b: 0,
    tag_wll_b_angle: 45,
    sling_legs_b: 1,
    crane_rated_a: 0,
    crane_rated_b: 0,
  };
}

/// A blank sheet: one load line and one tackle line, exactly the forms the
/// operator starts typing into.
export function defaultState() {
  return {
    load_items: [blankLoadItem()],
    tackle_items: [blankTackleItem()],
    crane: defaultCrane(),
    sling: defaultSling(),
    nonuniform: defaultNonuniform("asym_2leg_shortening"),
    tandem: defaultNonuniform("tandem_aligned_pivot"),
    // Presentation only; the engine ignores it.
    chart_table: null,
  };
}

export function createStore({ onError } = {}) {
  // No session persistence by design: every launch starts with a blank sheet,
  // so nothing carries over between runs.
  let state = defaultState();
  let solved = null;
  const listeners = new Set();
  let solveAgain = false;
  let solving = false;

  function emit() {
    for (const listener of listeners) listener();
  }

  async function solveNow() {
    if (solving) {
      solveAgain = true;
      return;
    }
    solving = true;
    try {
      solved = await api.solveAll(state);
      emit();
    } catch (error) {
      if (onError) onError(error);
    } finally {
      solving = false;
      if (solveAgain) {
        solveAgain = false;
        solveNow();
      }
    }
  }

  const scheduleSolve = debounce(solveNow, 70);

  function update(mutator) {
    mutator(state);
    emit();
    scheduleSolve();
  }

  function subscribe(listener) {
    listeners.add(listener);
    return () => listeners.delete(listener);
  }

  function setState(next) {
    state = next;
    emit();
    solveNow();
  }

  async function importChartFromFile() {
    const path = await api.pickChartFile();
    if (!path) return null;
    return importChartPath(path);
  }

  // Import a workbook by path — the file picker's own step, split out for the
  // `--chart=` QA hook.
  async function importChartPath(path) {
    try {
      const result = await api.importChart(state, path);
      update((draft) => {
        draft.crane = result.crane;
        draft.chart_table = result.table;
      });
      return result.crane.chart_label;
    } catch (error) {
      if (onError) onError(error);
      return null;
    }
  }

  async function chartAction(action) {
    try {
      const result = await api.chartUpdate(state.crane, action);
      update((draft) => {
        draft.crane = result.crane;
        if (result.table) draft.chart_table = result.table;
        if (action.action === "manual" || action.action === "forget") draft.chart_table = null;
      });
    } catch (error) {
      if (onError) onError(error);
    }
  }

  // Editing the manual row drops any workbook (the engine's own rule) and lets
  // the working radius follow the row.
  function manualChartEdit(patch) {
    const row = { ...state.crane.rows[0], ...patch };
    return chartAction({ action: "manual", row });
  }

  // The boom angle is the jib charts' own input: setting it here lets the
  // Crane tab drive the jib check without the Graph tab open. The working
  // radius follows from the assembly geometry (the solver derives it).
  function setBoomAngle(angle) {
    update((draft) => {
      draft.crane.boom_angle_deg = angle;
      if (draft.crane.rows[0]) draft.crane.rows[0].angle = angle;
    });
  }

  // Choose a jib configuration and its allowed offset angle, with the
  // workbook-aware rules in one place: both the crane tab's controls and the
  // graph's configuration radios go through it. The configuration picks its
  // jib — the plain jib is the workbook's shortest table, the extended jib the
  // longest — so the only jib figure left to choose is the offset angle the
  // chosen configuration allows. An offset that does not exist for that jib
  // falls back to its first allowed one.
  function chooseJib({ config, offset } = {}) {
    const crane = state.crane;
    const jibs = crane.chart_jibs || [];
    const gross = solved?.totals?.gross ?? 0;
    const nextConfig = config ?? crane.jib_config ?? "boom";
    if (nextConfig === "boom") {
      return chartAction({ action: "jib", config: "boom", length: null, offset: null, gross });
    }
    const lengths = [...new Set(jibs.map((jib) => jib.length))].sort((a, b) => a - b);
    const nextLength =
      nextConfig === "boom_ext_jib" ? lengths[lengths.length - 1] : lengths[0];
    const offsets = jibs
      .filter((jib) => Math.abs(jib.length - nextLength) < 1e-9)
      .map((jib) => jib.offset)
      .sort((a, b) => a - b);
    let nextOffset = offset ?? crane.jib_offset ?? null;
    if (nextOffset === null || !offsets.some((value) => Math.abs(value - nextOffset) < 1e-9)) {
      nextOffset = offsets[0] ?? null;
    }
    return chartAction({
      action: "jib",
      config: nextConfig,
      length: nextLength,
      offset: nextOffset,
      gross,
    });
  }

  // The engine is the source of truth for the initial screen too.
  solveNow();

  return {
    get state() {
      return state;
    },
    get solved() {
      return solved;
    },
    update,
    setState,
    subscribe,
    solveNow,
    importChartFromFile,
    importChartPath,
    chartAction,
    chooseJib,
    setBoomAngle,
    manualChartEdit,
  };
}
