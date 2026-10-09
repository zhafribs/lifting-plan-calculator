// Tiny DOM helpers: enough structure to build screens without a framework.

export function el(tag, props = {}, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props || {})) {
    if (value === null || value === undefined || value === false) continue;
    if (key === "class") node.className = value;
    else if (key === "text") node.textContent = value;
    else if (key === "html") node.innerHTML = value;
    else if (key === "dataset") Object.assign(node.dataset, value);
    else if (key === "style" && typeof value === "object") Object.assign(node.style, value);
    else if (key.startsWith("on") && typeof value === "function") {
      node.addEventListener(key.slice(2).toLowerCase(), value);
    } else if (key === "value") node.value = value;
    else if (typeof value === "boolean") node.setAttribute(key, value ? "" : null);
    else node.setAttribute(key, value);
  }
  append(node, children);
  return node;
}

export function append(node, children) {
  for (const child of children.flat(Infinity)) {
    if (child === null || child === undefined || child === false) continue;
    node.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
}

export function clear(node) {
  while (node.firstChild) node.removeChild(node.firstChild);
}

export function svgIcon(name, size = 19) {
  const paths = ICONS[name] || ICONS.dot;
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("width", size);
  svg.setAttribute("height", size);
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "1.7");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  svg.innerHTML = paths;
  return svg;
}

const ICONS = {
  dot: '<circle cx="12" cy="12" r="3.2"/>',
  weight:
    '<path d="M6.5 8.5h11l1.6 9.5a2 2 0 0 1-2 2.3H6.9a2 2 0 0 1-2-2.3z"/><path d="M9 8.5a3 3 0 1 1 6 0"/>',
  crane:
    '<path d="M4 21V5.5L15.5 4M4 6.5h12.5M15.5 4v4"/><path d="M15.5 8.5v3.2"/><path d="M15.5 11.7a2.4 2.4 0 1 0 0 4.8 2.4 2.4 0 0 0 0-4.8z"/><path d="M4 21h6M7 21v-3"/>',
  sling:
    '<path d="M12 3.5v3.2"/><path d="M12 6.7 6.5 15.5v4h11v-4z"/><path d="M12 6.7l5.5 8.8"/><path d="M6.5 19.5h11"/>',
  graph:
    '<path d="M4.5 19.5V4.5M4.5 19.5h16"/><path d="M7.5 15.5l3.5-4.5 3 2.5 4-6"/>',
  nonuniform:
    '<path d="M12 3.5v3"/><path d="M12 6.5 4.5 16v3.5"/><path d="M12 6.5 19.5 13v6.5"/><path d="M2.5 19.5h4M17.5 19.5h4"/><circle cx="12" cy="12.2" r="1.3"/>',
  tandem:
    '<path d="M3.5 4.5v4M20.5 4.5v4"/><path d="M3.5 8.5a2 2 0 1 0 0 4M20.5 8.5a2 2 0 1 1 0 4"/><path d="M3.5 12.5 12 16.5l8.5-4"/><rect x="7" y="16.5" width="10" height="4" rx="1.2"/>',
  report:
    '<path d="M7 3.5h7.5L19 8v12.5H7z"/><path d="M14.5 3.5V8H19"/><path d="M10 12h6M10 15.5h6M10 18h3.5"/>',
  about:
    '<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5.5"/><circle cx="12" cy="7.8" r="0.9" fill="currentColor" stroke="none"/>',
  plus: '<path d="M12 5.5v13M5.5 12h13"/>',
  close: '<path d="M6.5 6.5l11 11M17.5 6.5l-11 11"/>',
  file: '<path d="M7 3.5h7.5L19 8v12.5H7z"/><path d="M14.5 3.5V8H19"/>',
  print:
    '<path d="M7 8.5V4h10v4.5"/><rect x="4" y="8.5" width="16" height="7.5" rx="1.5"/><path d="M7 14h10v6H7z"/>',
  external: '<path d="M13.5 5H19v5.5"/><path d="M19 5l-8 8"/><path d="M17 13.5V19H5V7h5.5"/>',
  example: '<path d="M12 4v16M4 12h16"/>',
  reset: '<path d="M4.5 10a7.5 7.5 0 1 1 2.2 5.3"/><path d="M4.5 10V5.5M4.5 10H9"/>',
};

export function debounce(fn, wait = 60) {
  let timer = null;
  return (...args) => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      fn(...args);
    }, wait);
  };
}
