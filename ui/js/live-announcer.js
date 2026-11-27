// Live-region writes, serialised.
//
// Two messages in one frame would otherwise both clear the region and then both
// set their text in the same frame, so only the last is written. This queue
// writes each message alone, with a gap after it, so both are written.
//
// The scheduler is injected so the queue can be tested without timers.

export function createAnnouncer({
  set,
  gap = 350,
  schedule = (fn, ms) => setTimeout(fn, ms),
} = {}) {
  if (typeof set !== "function") throw new TypeError("createAnnouncer needs a setter");
  const queue = [];
  let busy = false;
  let current = null;

  function pump() {
    if (busy) return;
    const text = queue.shift();
    if (text === undefined) return;
    busy = true;
    current = text;
    // Clear first so a repeated message is still a change to announce.
    set("");
    schedule(() => {
      set(text);
      // Leave a gap so the next message does not replace this one immediately.
      schedule(() => {
        busy = false;
        pump();
      }, gap);
    }, 0);
  }

  return {
    say(text) {
      const message = String(text);
      // A tight failure loop must not queue the same sentence a hundred times:
      // the one being written, or already waiting, is enough.
      if (queue[queue.length - 1] === message) return;
      if (busy && queue.length === 0 && current === message) return;
      // A burst of distinct messages is bounded too: announcements the reader
      // has already missed are not worth queueing minutes of speech for.
      if (queue.length >= 8) queue.shift();
      queue.push(message);
      pump();
    },
  };
}
