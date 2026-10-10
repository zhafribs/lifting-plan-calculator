// The bridge to the Rust engine. Every call is a Tauri command; the UI never
// calculates a figure itself.

const core = window.__TAURI__?.core;

function invoke(command, args) {
  if (!core || typeof core.invoke !== "function") {
    return Promise.reject(new Error("The Tauri bridge is not available."));
  }
  return core.invoke(command, args);
}

export const api = {
  appInfo: () => invoke("app_info"),
  catalog: () => invoke("catalog"),
  solveAll: (state) => invoke("solve_all", { state }),
  pickChartFile: () => invoke("pick_chart_file"),
  importChart: (state, path) => invoke("import_chart", { state, path }),
  chartUpdate: (crane, action) => invoke("chart_update", { crane, action }),
  buildReport: (state, include) => invoke("build_report", { state, include }),
  saveReport: (html, suggestedName) => invoke("save_report", { html, suggestedName }),
  openReportFile: (path) => invoke("open_report_file", { path }),
  checkUpdate: () => invoke("check_update"),
  installUpdate: (asset) => invoke("install_update", { asset }),
  openExternal: (url) => invoke("open_external", { url }),
  quitApp: () => invoke("quit_app"),
};

// A friendly sentence for anything thrown across the bridge.
export function errorText(error) {
  if (typeof error === "string") return error;
  if (error && typeof error.message === "string") return error.message;
  return "That action could not be completed.";
}
