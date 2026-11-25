// The Summary screen's build inputs and its one-at-a-time queue. Keeping both
// pure makes the stale-report and rapid-typing rules testable without a webview.

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === "object") {
    const sorted = {};
    for (const key of Object.keys(value).sort()) sorted[key] = canonical(value[key]);
    return sorted;
  }
  return value;
}

export function buildReportKey(inputs) {
  return JSON.stringify(canonical(inputs));
}

/**
 * Run `task` one at a time. A call made while a task is running replaces any
 * earlier pending call, so the next run uses the latest args instead of a
 * backlog; every caller waits for that final run.
 */
export function createSingleFlight(task) {
  let running = null;
  let pending = null;

  return function run(args) {
    if (running) {
      pending = args;
      return running;
    }

    running = (async () => {
      let current = args;
      let result;
      while (current !== null) {
        pending = null;
        result = await task(current);
        current = pending;
      }
      return result;
    })()
      .catch((error) => {
        pending = null;
        throw error;
      })
      .finally(() => {
        running = null;
      });

    return running;
  };
}
