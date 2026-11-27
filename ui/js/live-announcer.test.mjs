import test from "node:test";
import assert from "node:assert/strict";

import { createAnnouncer } from "./live-announcer.js";

function harness(gap = 100) {
  const writes = [];
  const tasks = [];
  const announcer = createAnnouncer({
    set: (text) => writes.push(text),
    gap,
    schedule: (fn, ms) => tasks.push({ fn, ms }),
  });
  return {
    announcer,
    writes,
    tasks,
    run() {
      while (tasks.length) tasks.shift().fn();
    },
  };
}

test("two messages in one frame are both written, in order", () => {
  const { announcer, writes, run } = harness();
  announcer.say("one");
  announcer.say("two");
  run();
  assert.deepEqual(writes, ["", "one", "", "two"]);
});

test("a message is written alone and the gap follows it", () => {
  const { announcer, writes, tasks, run } = harness(250);
  announcer.say("only");
  assert.deepEqual(writes, [""]);
  assert.deepEqual(tasks.map((task) => task.ms), [0]);
  run();
  assert.deepEqual(writes, ["", "only"]);
  assert.deepEqual(tasks, []);
});

test("each message waits out the gap before the next", () => {
  const { announcer, tasks } = harness(250);
  announcer.say("one");
  announcer.say("two");
  tasks.shift().fn(); // write "one"
  const gap = tasks.shift();
  assert.equal(gap.ms, 250);
  gap.fn(); // pump: the second message's write is scheduled, then its gap
  assert.equal(tasks[0].ms, 0);
  tasks.shift().fn(); // write "two"
  assert.equal(tasks[0].ms, 250);
});

test("a repeated message is not queued twice", () => {
  const { announcer, writes, run } = harness();
  announcer.say("again");
  announcer.say("again");
  announcer.say("again");
  run();
  assert.deepEqual(writes, ["", "again"]);
});

test("a burst of distinct messages is bounded", () => {
  const { announcer, writes, run } = harness();
  for (let index = 0; index < 20; index += 1) announcer.say(`message ${index}`);
  run();
  const spoken = writes.filter(Boolean);
  assert.ok(spoken.length <= 9, `spoke ${spoken.length} messages`);
  assert.equal(spoken[spoken.length - 1], "message 19");
});

test("a missing setter is a programming error", () => {
  assert.throws(() => createAnnouncer({}), TypeError);
});
