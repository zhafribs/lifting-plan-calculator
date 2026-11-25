// Screens are built on first visit, then reused. The registry is pure so the
// caching and retry rules can be tested without a webview.

export function createScreenRegistry(factories) {
  const cache = new Map();
  // Not `key in factories`: an inherited name such as "toString" must not look
  // like a screen. hasOwnProperty.call is the same check, kept explicit.
  const has = (key) => Object.prototype.hasOwnProperty.call(factories, key);

  return {
    has,
    get(key) {
      if (!has(key)) return null;
      if (!cache.has(key)) cache.set(key, factories[key]());
      return cache.get(key);
    },
  };
}
