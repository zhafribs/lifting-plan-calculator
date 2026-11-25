// The Summary screen's build inputs, its one-at-a-time queue, and the quality
// guarantee that keeps a 2x preview out of a printed PDF. Keeping them pure
// makes the stale-report and rapid-typing rules testable without a webview.

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

/**
 * Read the report at the requested quality, rebuilding until it is the thing
 * that was asked for. A preview request can replace a pending export in the
 * queue, so Save/Print re-assert after the run instead of trusting the last
 * writer. Three attempts is enough for a click that races typing; after that a
 * retry is safer than printing the wrong diagram.
 */
export async function ensureQuality(quality, read, rebuild) {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    const current = read();
    if (current.quality === quality) return current.html;
    await rebuild(quality);
    const after = read();
    if (after.quality === quality) return after.html;
  }
  throw new Error("The report was still rebuilding. Try again.");
}
