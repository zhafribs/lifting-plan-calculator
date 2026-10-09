// The shell: rail, topbar, screens, and the publish cycle that keeps every
// screen looking at the same solved state.

import { el, svgIcon } from "./dom.js";
import { api, errorText } from "./api.js";
import { createStore } from "./store.js";
import { fmt } from "./format.js";
import { createOverallScreen } from "./screens/overall.js";
import { createCraneScreen } from "./screens/crane.js";
import { createGraphScreen } from "./screens/graph.js";
import { createUniformScreen } from "./screens/uniform.js";
import { createNonuniformScreen } from "./screens/nonuniform.js";
import { createTandemScreen } from "./screens/tandem.js";
import { createReportScreen } from "./screens/report.js";
import { createAboutScreen } from "./screens/about.js";

const NAV = [
  {
    group: "Lift",
    items: [
      { key: "overall", label: "Overall Weight", icon: "weight", shortcut: "1", title: "Overall Weight", subtitle: "The load lines and the tackle, summed to the gross load at hook." },
      { key: "crane", label: "Crane", icon: "crane", shortcut: "2", title: "Crane", subtitle: "The crane and its load chart, checked at the working radius under the 75% rule." },
      { key: "graph", label: "Graph", icon: "graph", shortcut: "3", title: "Working Range Diagram", subtitle: "Working range drawn to scale, with radius, heights and clearances read out." },
    ],
  },
  {
    group: "Rigging",
    items: [
      { key: "uniform", label: "Uniform Load", icon: "sling", shortcut: "4", title: "Uniform Load", subtitle: "One continuous sling; every effective leg shares the load equally." },
      { key: "nonuniform", label: "Nonuniform Load", icon: "nonuniform", shortcut: "5", title: "Nonuniform Load", subtitle: "Two legs sharing one headroom with the centre of gravity off the midpoint." },
      { key: "tandem", label: "Tandem", icon: "tandem", shortcut: "6", title: "Tandem", subtitle: "Two independent cranes sharing one load, with the declared tilt." },
    ],
  },
  {
    group: "Plan",
    items: [
      { key: "report", label: "Summary", icon: "report", shortcut: "7", title: "Lifting Plan", subtitle: "The printable report, typeset with its own equations." },
    ],
  },
  {
    group: "Help",
    items: [
      { key: "about", label: "About", icon: "about", title: "About", subtitle: "What this app does, and the person behind it." },
    ],
  },
];

const ORDER = NAV.flatMap((group) => group.items).map((item) => item.key);

// Screens whose wide layout splits into two independently scrolling panes:
// the form column and the results column each keep their own scroll position.
const SPLIT_KEYS = new Set([
  "overall",
  "crane",
  "graph",
  "uniform",
  "nonuniform",
  "tandem",
  "report",
]);

async function main() {
  window.uilog && window.uilog("app main start");
  const rail = document.getElementById("rail");
  const topbar = document.getElementById("topbar");
  const screenEl = document.getElementById("screen");
  const toastBox = document.getElementById("toasts");

  let catalog;
  try {
    catalog = await api.catalog();
    window.uilog && window.uilog(`catalog ok: ${catalog.hitches.length} hitches`);
  } catch (error) {
    window.uilog && window.uilog(`catalog failed: ${errorText(error)}`);
    screenEl.replaceChildren(
      el("div", { class: "page" }, el("p", { class: "error-text", text: `The application could not start: ${errorText(error)}` })),
    );
    return;
  }

  function toast(text, kind = "") {
    const node = el("div", { class: `toast ${kind}`.trim(), text });
    toastBox.append(node);
    setTimeout(() => node.remove(), 6000);
  }

  const store = createStore({ onError: (error) => toast(errorText(error), "error") });
  const ctx = { store, catalog, toast, go, showReport };

  const screens = {
    overall: createOverallScreen(ctx),
    crane: createCraneScreen(ctx),
    graph: createGraphScreen(ctx),
    uniform: createUniformScreen(ctx),
    nonuniform: createNonuniformScreen(ctx),
    tandem: createTandemScreen(ctx),
    report: createReportScreen(ctx),
    about: createAboutScreen(ctx),
  };

  let currentKey = null;
  let current = null;

  // --- rail ----------------------------------------------------------------
  // The nav list scrolls on its own so the footer (name, version, contact)
  // stays pinned and visible even in the smallest window.
  const railNav = el("nav", { class: "rail-nav" });
  const navButtons = {};
  for (const group of NAV) {
    railNav.append(el("div", { class: "nav-group", text: group.group }));
    for (const item of group.items) {
      const button = el(
        "button",
        {
          class: "nav-item",
          onClick: () => go(item.key),
          // Also the hover tooltip when the rail is collapsed to icons.
          title: item.shortcut ? `${item.label} (Ctrl+${item.shortcut})` : item.label,
        },
        svgIcon(item.icon),
        el("span", { text: item.label }),
        item.shortcut ? el("span", { class: "key", text: `Ctrl+${item.shortcut}` }) : null,
      );
      navButtons[item.key] = button;
      railNav.append(button);
    }
  }

  rail.append(
    railNav,
    el(
      "div",
      { class: "rail-foot" },
      el("div", { class: "brand", text: catalog.app_name }),
      el("div", { text: `Version ${catalog.version}` }),
      el("div", { text: `${catalog.contact_name}, ${catalog.contact_email}` }),
    ),
  );

  // --- topbar --------------------------------------------------------------
  const title = el("h1", { text: "" });
  const subtitle = el("p", { text: "" });
  const chipGross = el("span", { class: "v", text: "0 kg" });
  const chipRequired = el("span", { class: "v", text: "0 kg" });
  topbar.append(
    el("div", { class: "topbar-titles" }, title, subtitle),
    el("span", { class: "topbar-spacer" }),
    el(
      "div",
      { class: "chips" },
      el("div", { class: "chip" }, el("span", { class: "k", text: "Gross load" }), chipGross),
      el("div", { class: "chip" }, el("span", { class: "k", text: "Required capacity (75%)" }), chipRequired),
    ),
  );

  function updateChips() {
    const totals = store.solved?.totals;
    chipGross.textContent = fmt(totals?.gross ?? 0, 2, "kg");
    chipRequired.textContent = fmt(totals?.required_chart_capacity ?? 0, 2, "kg");
  }

  function go(key) {
    if (!screens[key]) return;
    currentKey = key;
    current = screens[key];
    for (const [name, button] of Object.entries(navButtons)) {
      button.classList.toggle("active", name === key);
    }
    const meta = NAV.flatMap((group) => group.items).find((item) => item.key === key);
    title.textContent = meta.title;
    subtitle.textContent = meta.subtitle;
    screenEl.replaceChildren(current.root);
    screenEl.classList.toggle("split-screen", SPLIT_KEYS.has(key));
    screenEl.scrollTop = 0;
    if (current.sync) current.sync(store.state);
    if (current.render) current.render(store.solved);
  }

  function showReport(sections) {
    if (screens.report.setSections) screens.report.setSections(sections);
    go("report");
  }

  store.subscribe(() => {
    updateChips();
    if (!current) return;
    if (current.sync) current.sync(store.state);
    if (current.render) current.render(store.solved);
  });

  window.addEventListener("keydown", (event) => {
    if (!event.ctrlKey || event.altKey || event.metaKey) return;
    const index = Number(event.key);
    if (index >= 1 && index <= 7) {
      event.preventDefault();
      go(ORDER[index - 1]);
    }
  });

  updateChips();
  try {
    const info = await api.appInfo();
    window.uilog && window.uilog(`app_info: ${JSON.stringify(info)}`);
    // QA hook (mirrors --screen=): boot the uniform tab onto a chosen hitch.
    if (
      info.start_hitch &&
      catalog.hitches.some((hitch) => hitch.key === info.start_hitch)
    ) {
      store.update((state) => (state.sling.hitch = info.start_hitch));
    }
    go(info.start_screen && screens[info.start_screen] ? info.start_screen : "overall");
  } catch (error) {
    window.uilog && window.uilog(`app_info failed: ${errorText(error)}`);
    go("overall");
  }
  // A breadcrumb about the scroll container, taken once the first screen is
  // populated: when scroll is greater than client, the page has something to
  // scroll to and the wheel will reach it.
  window.uilog &&
    window.uilog(
      `screen metrics: client=${screenEl.clientHeight} scroll=${screenEl.scrollHeight}`,
    );
}

main();
