// The shell: rail, topbar, screens, and the publish cycle that keeps every
// screen looking at the same solved state.

import { el, svgIcon } from "./dom.js";
import { api, errorText } from "./api.js";
import { createStore } from "./store.js";
import { createScreenRegistry } from "./screen-registry.js";
import { fmt } from "./format.js";
import {
  platformScope,
  initialNavMode,
  nextNavMode,
  navToggleLabel,
} from "./windows-theme.js";
import {
  applyTheme,
  desktopIsDark,
  normalisePaletteChoice,
  normalisePreference,
} from "./theme.js";
import { createOverallScreen } from "./screens/overall.js";
import { createCraneScreen } from "./screens/crane.js";
import { createOutriggerScreen } from "./screens/outrigger.js";
import { createGraphScreen } from "./screens/graph.js";
import { createUniformScreen } from "./screens/uniform.js";
import { createNonuniformScreen } from "./screens/nonuniform.js";
import { createTandemScreen } from "./screens/tandem.js";
import { createReportScreen } from "./screens/report.js";
import { createAboutScreen } from "./screens/about.js";
import { createSettingsScreen } from "./screens/settings.js";

// WebKitGTK draws a native `title` tooltip as one large black window. Replace
// it with a small bubble of our own; the text is moved to `data-tip` on first
// hover so the native one never gets the chance to open.
function installTooltips() {
  const tip = el("div", { class: "app-tip" });
  document.body.append(tip);
  let current = null;
  let timer = 0;

  function hide() {
    clearTimeout(timer);
    tip.classList.remove("show");
    current = null;
  }

  function show(target) {
    const text = target.dataset.tip;
    if (!text) return;
    current = target;
    tip.textContent = text;
    tip.classList.add("show");
    const box = target.getBoundingClientRect();
    const size = tip.getBoundingClientRect();
    if (target.closest(".rail")) {
      tip.style.left = `${box.right + 10}px`;
      tip.style.top = `${Math.max(8, box.top + box.height / 2 - size.height / 2)}px`;
      return;
    }
    const above = box.top - size.height - 6;
    tip.style.left = `${Math.min(
      Math.max(8, box.left + box.width / 2 - size.width / 2),
      window.innerWidth - size.width - 8,
    )}px`;
    tip.style.top = `${above < 8 ? box.bottom + 6 : above}px`;
  }

  const owner = (event) => {
    const node = event.target instanceof Element
      ? event.target.closest("[title], [data-tip]")
      : null;
    return node && node.tagName !== "IFRAME" ? node : null;
  };

  // Take the text away from the platform as soon as the control is met:
  // WebKitGTK opens its own tooltip on a title that is still present.
  function adopt(target) {
    if (!target.hasAttribute("title")) return;
    target.dataset.tip = target.getAttribute("title");
    target.removeAttribute("title");
  }

  document.addEventListener("pointerover", (event) => {
    const target = owner(event);
    if (!target) {
      hide();
      return;
    }
    adopt(target);
    if (target === current) return;
    clearTimeout(timer);
    timer = setTimeout(() => show(target), 250);
  });
  document.addEventListener("pointerout", (event) => {
    const target = owner(event);
    if (!target) return;
    if (event.relatedTarget instanceof Node && target.contains(event.relatedTarget)) return;
    hide();
  });
  document.addEventListener("pointerdown", hide);
  // Keyboard users get the same hint: focus shows it at once, like the buttons
  // in the collapsed rail that have no visible label.
  document.addEventListener("focusin", (event) => {
    const target = owner(event);
    if (!target) return;
    adopt(target);
    show(target);
  });
  document.addEventListener("focusout", hide);
  window.addEventListener("scroll", hide, true);
  window.addEventListener("blur", hide);
}

const NAV = [
  {
    group: "Lift",
    items: [
      { key: "overall", label: "Overall Weight", icon: "weight", shortcut: "1", title: "Overall Weight", subtitle: "The load lines and the tackle, summed to the gross load at hook." },
      { key: "crane", label: "Crane", icon: "crane", shortcut: "2", title: "Crane", subtitle: "The crane and its load chart, checked at the working radius under the 75% rule." },
      { key: "outrigger", label: "Outrigger Pad Size", icon: "pad", shortcut: "3", title: "Outrigger Pad Size", subtitle: "The bearing area under the crane's outrigger, from the BS 8004 presumed bearing table." },
      { key: "graph", label: "Graph", icon: "graph", shortcut: "4", title: "Working Range Diagram", subtitle: "Working range drawn to scale, with radius, heights and clearances read out." },
    ],
  },
  {
    group: "Rigging",
    items: [
      { key: "uniform", label: "Uniform Load", icon: "sling", shortcut: "5", title: "Uniform Load", subtitle: "One continuous sling; every effective leg shares the load equally." },
      { key: "nonuniform", label: "Nonuniform Load", icon: "nonuniform", shortcut: "6", title: "Nonuniform Load", subtitle: "Two legs sharing one headroom with the centre of gravity off the midpoint." },
      { key: "tandem", label: "Tandem", icon: "tandem", shortcut: "7", title: "Tandem", subtitle: "Two independent cranes sharing one load, with the declared tilt." },
    ],
  },
  {
    group: "Plan",
    items: [
      { key: "report", label: "Summary", icon: "report", shortcut: "8", title: "Lifting Plan", subtitle: "The printable report, typeset with its own equations." },
    ],
  },
  {
    group: "Help",
    items: [
      { key: "settings", label: "Settings", icon: "settings", title: "Settings", subtitle: "The appearance, and where its colours come from." },
      { key: "about", label: "About", icon: "about", title: "About", subtitle: "What this app does, and the person behind it." },
    ],
  },
];

const SHORTCUTS = new Map(
  NAV.flatMap((group) => group.items)
    .filter((item) => item.shortcut)
    .map((item) => [item.shortcut, item.key]),
);

// Screens whose wide layout splits into two independently scrolling panes:
// the form column and the results column each keep their own scroll position.
const SPLIT_KEYS = new Set([
  "overall",
  "crane",
  "outrigger",
  "graph",
  "uniform",
  "nonuniform",
  "tandem",
  "report",
]);

async function main() {
  window.uilog && window.uilog("app main start");
  installTooltips();
  const rail = document.getElementById("rail");
  const topbar = document.getElementById("topbar");
  const screenEl = document.getElementById("screen");
  const toastBox = document.getElementById("toasts");
  // Set once the operator picks a rail width by hand; from then on, resizing
  // the window no longer overrides the choice.
  let navPinned = false;

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

  // --- appearance -----------------------------------------------------------
  // One neutral palette for every desktop. Windows paints its Fluent layer over
  // it through a platform marker; Linux keeps the neutral shell. The scheme
  // follows the desktop unless the operator chose one in About, and the accent
  // is the desktop's own when it publishes one.
  const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");
  let appearance = {
    preference: "system",
    accent: null,
    accentSource: "default",
    scheme: null,
    palette: null,
    paletteChoice: "desktop",
  };
  // The last scheme handed to the native chrome, so the 20-second palette poll
  // does not send the same IPC on every pass.
  let lastWindowTheme = null;

  function paintAppearance() {
    const applied = applyTheme(document.documentElement, {
      preference: appearance.preference,
      systemDark: desktopIsDark(appearance.scheme, darkQuery.matches),
      accent: appearance.accent,
      accentSource: appearance.accentSource,
      palette: appearance.palette,
      paletteChoice: appearance.paletteChoice,
    });
    window.uilog &&
      window.uilog(
        `theme: scheme=${applied.scheme} preference=${appearance.preference} ` +
          `accent=${applied.vars.accent} source=${appearance.accentSource} tinted=${applied.tinted}`,
      );
    // Windows draws the title bar and Mica from the native window theme, so an
    // in-app override has to reach the window too or the chrome keeps the
    // desktop's scheme. Other platforms ignore the call.
    const requestedTheme =
      appearance.preference === "system" ? "system" : applied.scheme;
    if (requestedTheme !== lastWindowTheme) {
      lastWindowTheme = requestedTheme;
      api.setWindowTheme(requestedTheme).catch(() => {});
    }
  }

  async function loadAppearance() {
    try {
      const info = await api.appearance();
      appearance = {
        preference: normalisePreference(info?.preference),
        accent: typeof info?.accent === "string" ? info.accent : null,
        accentSource:
          typeof info?.accent_source === "string" ? info.accent_source : "default",
        scheme:
          info?.scheme === "dark" || info?.scheme === "light" ? info.scheme : null,
        palette:
          info?.palette && typeof info.palette === "object" ? info.palette : null,
        paletteChoice: normalisePaletteChoice(info?.palette_choice),
      };
    } catch (error) {
      // A desktop that cannot answer still gets the fallback palette.
      window.uilog && window.uilog(`appearance unavailable: ${errorText(error)}`);
    }
    paintAppearance();
  }

  async function setThemePreference(preference) {
    appearance.preference = normalisePreference(preference);
    paintAppearance();
    try {
      await api.setThemePreference(appearance.preference);
    } catch (error) {
      toast(errorText(error), "error");
    }
  }

  async function setPaletteChoice(palette) {
    appearance.paletteChoice = normalisePaletteChoice(palette);
    paintAppearance();
    try {
      await api.setPaletteChoice(appearance.paletteChoice);
    } catch (error) {
      toast(errorText(error), "error");
    }
  }

  function applyPlatform(info) {
    const platform = platformScope(info || {});
    document.documentElement.dataset.platform = platform;
    // The rail starts collapsed in a narrow window. Windows keeps its 1000px
    // rule; the Linux breakpoint matches the stylesheet's 1099px.
    const compact =
      platform === "windows"
        ? initialNavMode(window.innerWidth) === "compact"
        : window.innerWidth < 1100;
    setNavMode(compact ? "compact" : "expanded");
  }

  darkQuery.addEventListener("change", () => {
    if (appearance.preference === "system") loadAppearance();
  });

  // A wallpaper change rewrites the desktop's palette without an event. While
  // the window is in front, a slow poll keeps the shell in step; in the
  // background it costs nothing.
  setInterval(() => {
    if (document.hasFocus()) loadAppearance();
  }, 20000);

  // A wallpaper change rewrites the palette file without telling anyone. One
  // cheap read when the window is focused keeps the tint current, without a
  // timer running in the background.
  let lastAppearanceRead = 0;
  window.addEventListener("focus", () => {
    const now = Date.now();
    if (now - lastAppearanceRead < 10000) return;
    lastAppearanceRead = now;
    loadAppearance();
  });

  await loadAppearance();

  function toast(text, kind = "") {
    const node = el("div", { class: `toast ${kind}`.trim(), text });
    toastBox.append(node);
    setTimeout(() => node.remove(), 6000);
  }

  const store = createStore({ onError: (error) => toast(errorText(error), "error") });
  const ctx = {
    store,
    catalog,
    toast,
    go,
    showReport,
    setThemePreference,
    setPaletteChoice,
    themePreference: () => appearance.preference,
    paletteChoice: () => appearance.paletteChoice,
    accentSource: () => appearance.accentSource,
    accent: () => appearance.accent,
  };

  // The update reminder strip, between the topbar and the screen. Hidden until
  // GitHub says there is a newer release.
  const updateBar = el("div", { class: "update-bar" });
  screenEl.parentElement.insertBefore(updateBar, screenEl);

  // The strip is a reminder, not a second update page: the actions live in
  // Settings, so there is one place to look and one set of paths to keep.
  function showUpdate(info) {
    updateBar.replaceChildren(
      el("span", {
        class: "msg",
        text: `Version ${info.latest} is available — you have ${info.current}.`,
      }),
      el("span", { class: "spacer" }),
      el(
        "button",
        { class: "btn ghost small", onClick: () => api.openExternal(info.release_url).catch(() => {}) },
        "Release notes",
      ),
      el(
        "button",
        {
          class: "btn primary small",
          onClick: () => {
            updateBar.replaceChildren();
            updateBar.classList.remove("on");
            go("settings");
          },
        },
        "Show updates",
      ),
      el(
        "button",
        {
          class: "btn ghost small",
          title: "Hide this reminder until the next launch",
          onClick: () => {
            updateBar.replaceChildren();
            updateBar.classList.remove("on");
          },
        },
        "Dismiss",
      ),
    );
    updateBar.classList.add("on");
  }

  function screen(key, create) {
    return () => {
      const started = performance.now();
      const instance = create();
      window.uilog &&
        window.uilog(`screen ${key}: ${Math.round(performance.now() - started)}ms`);
      return instance;
    };
  }

  const screens = createScreenRegistry({
    overall: screen("overall", () => createOverallScreen(ctx)),
    crane: screen("crane", () => createCraneScreen(ctx)),
    outrigger: screen("outrigger", () => createOutriggerScreen(ctx)),
    graph: screen("graph", () => createGraphScreen(ctx)),
    uniform: screen("uniform", () => createUniformScreen(ctx)),
    nonuniform: screen("nonuniform", () => createNonuniformScreen(ctx)),
    tandem: screen("tandem", () => createTandemScreen(ctx)),
    report: screen("report", () => {
      // The Graph screen registers the offscreen renderer Summary captures.
      // Build it without showing it, so the diagram exists on a first visit.
      screens.get("graph");
      return createReportScreen(ctx);
    }),
    settings: screen("settings", () => createSettingsScreen(ctx)),
    about: screen("about", () => createAboutScreen(ctx)),
  });

  let currentKey = null;
  let current = null;

  // --- rail ----------------------------------------------------------------
  // The nav list scrolls on its own so the footer (name, version, contact)
  // stays pinned and visible even in the smallest window.
  //
  // The header is the Fluent NavigationView's: a hamburger that folds the rail
  // to its icon strip. It is hidden outside the Windows layer, so the base
  // layout keeps its original shape.
  const railToggle = el("button", {
    class: "rail-toggle",
    type: "button",
    title: navToggleLabel("expanded"),
    "aria-label": navToggleLabel("expanded"),
    "aria-expanded": "true",
    onClick: () => {
      navPinned = true;
      setNavMode(nextNavMode(document.documentElement.dataset.nav));
    },
  });
  railToggle.append(svgIcon("menu", 18));
  const railHeader = el(
    "div",
    { class: "rail-header" },
    railToggle,
    el("span", { class: "rail-title", text: catalog.app_name }),
  );

  function setNavMode(mode) {
    const root = document.documentElement;
    const next = mode === "compact" ? "compact" : "expanded";
    root.dataset.nav = next;
    const label = navToggleLabel(next);
    railToggle.title = label;
    railToggle.setAttribute("aria-label", label);
    railToggle.setAttribute("aria-expanded", next === "expanded" ? "true" : "false");
  }

  // Until the toggle is used, the rail follows the window, as it did before the
  // button existed. Windows keeps the mode it chose at launch.
  window.addEventListener("resize", () => {
    if (navPinned || document.documentElement.dataset.platform === "windows") return;
    setNavMode(window.innerWidth < 1100 ? "compact" : "expanded");
  });

  const railNav = el("nav", { class: "rail-nav" });
  const navButtons = {};
  for (const group of NAV) {
    railNav.append(el("div", { class: "nav-group", text: group.group }));
    for (const item of group.items) {
      const tipText = item.shortcut ? `${item.label} (Ctrl+${item.shortcut})` : item.label;
      const button = el(
        "button",
        {
          class: "nav-item",
          onClick: () => go(item.key),
          // The hover tooltip, and the name the rail keeps when collapsed.
          title: tipText,
          "aria-label": tipText,
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
    railHeader,
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
    const next = screens.get(key);
    if (!next) return;
    currentKey = key;
    current = next;
    for (const [name, button] of Object.entries(navButtons)) {
      button.classList.toggle("active", name === key);
      if (name === key) button.setAttribute("aria-current", "page");
      else button.removeAttribute("aria-current");
    }
    const meta = NAV.flatMap((group) => group.items).find((item) => item.key === key);
    title.textContent = meta.title;
    subtitle.textContent = meta.subtitle;
    screenEl.replaceChildren(current.root);
    screenEl.classList.toggle("split-screen", SPLIT_KEYS.has(key));
    screenEl.scrollTop = 0;
    if (current.sync) current.sync(store.state);
    if (current.render) current.render(store.solved);
    // Screens that cost something to redraw refresh here, when they are opened,
    // rather than on every state publish.
    if (current.show) current.show();
  }

  function showReport(sections) {
    const report = screens.get("report");
    if (report.setSections) report.setSections(sections);
    go("report");
  }

  // A re-solve rewrites the screens' DOM, and WebKit reacts by scrolling a
  // focused control back into view — which yanks the page away from where the
  // operator left it. Snapshot every scroll container around the repaint and
  // put it back, right away and on the next frame (the scroll can happen
  // during layout, after the mutations).
  function scrollSnapshot() {
    return {
      screen: screenEl.scrollTop,
      panes: [...screenEl.querySelectorAll(".pane")].map((pane) => [pane, pane.scrollTop]),
    };
  }

  function restoreScroll(scrolls) {
    screenEl.scrollTop = scrolls.screen;
    for (const [pane, top] of scrolls.panes) pane.scrollTop = top;
  }

  store.subscribe(() => {
    updateChips();
    if (!current) return;
    const scrolls = scrollSnapshot();
    if (current.sync) current.sync(store.state);
    if (current.render) current.render(store.solved);
    restoreScroll(scrolls);
    requestAnimationFrame(() => restoreScroll(scrolls));
  });

  window.addEventListener("keydown", (event) => {
    if (!event.ctrlKey || event.altKey || event.metaKey) return;
    const key = SHORTCUTS.get(event.key);
    if (!key) return;
    event.preventDefault();
    go(key);
  });

  updateChips();
  try {
    const info = await api.appInfo();
    window.uilog && window.uilog(`app_info: ${JSON.stringify(info)}`);
    ctx.platform = info.platform;
    applyPlatform(info);
    // A quiet start-of-day reminder: GitHub is asked once per launch, and an
    // offline machine (or a rate-limited one) simply sees no banner.
    api
      .checkUpdate()
      .then((update) => {
        if (update && update.newer) showUpdate(update);
      })
      .catch(() => {});
    // QA hook (mirrors --screen=): boot the uniform tab onto a chosen hitch.
    if (
      info.start_hitch &&
      catalog.hitches.some((hitch) => hitch.key === info.start_hitch)
    ) {
      store.update((state) => (state.sling.hitch = info.start_hitch));
    }
    // QA hook (mirrors --screen=): import a load chart at launch, and land it
    // on a chosen working radius.
    if (info.start_chart) {
      store.importChartPath(info.start_chart).then(() => {
        const radius = Number(info.chart_radius);
        if (Number.isFinite(radius) && radius > 0) {
          store.chartAction({ action: "radius", radius });
        }
      });
    }
    go(screens.has(info.start_screen) ? info.start_screen : "overall");
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
