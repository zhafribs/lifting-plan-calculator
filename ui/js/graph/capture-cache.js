// One cache for the report's diagram capture. The canvas render and PNG encode
// are the expensive part of a report rebuild, and most edits only touch the
// report text or the sign-off, not the crane configuration.
//
// The cache is deliberately tiny: the live preview and the print-quality image
// are the only two variants the report asks for.

export function captureKey(state, width, height, scale, format = "png") {
  const fields = Object.keys(state)
    .sort()
    .map((name) => [name, state[name]]);
  return `${width}x${height}@${scale}:${format}:${JSON.stringify(fields)}`;
}

export function createCaptureCache(limit = 2) {
  const entries = new Map();

  return {
    get(key) {
      if (!entries.has(key)) return null;
      const value = entries.get(key);
      entries.delete(key);
      entries.set(key, value);
      return value;
    },
    set(key, value) {
      entries.delete(key);
      entries.set(key, value);
      while (entries.size > limit) {
        entries.delete(entries.keys().next().value);
      }
    },
    clear() {
      entries.clear();
    },
    get size() {
      return entries.size;
    },
  };
}
