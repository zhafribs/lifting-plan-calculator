// Which jib configurations a workbook's tables allow, without a webview.

import test from "node:test";
import assert from "node:assert/strict";

import { JIB_CONFIGS, jibAvailability, jibLengthFor, jibLengths } from "./jib.js";

test("distinct jib lengths are deduplicated and sorted", () => {
  const jibs = [
    { length: 12.8, offset: 5 },
    { length: 7.2, offset: 25 },
    { length: 7.2, offset: 45 },
    { length: 12.8, offset: 25 },
  ];
  assert.deepEqual(jibLengths(jibs), [7.2, 12.8]);
  assert.deepEqual(jibLengths([]), []);
  assert.deepEqual(jibLengths(undefined), []);
});

test("one jib length offers the plain jib only", () => {
  const availability = jibAvailability([
    { length: 7.2, offset: 5 },
    { length: 7.2, offset: 25 },
  ]);
  assert.deepEqual(availability, { boom: true, boom_jib: true, boom_ext_jib: false });
});

test("two jib lengths offer the extended jib", () => {
  const availability = jibAvailability([
    { length: 7.2, offset: 5 },
    { length: 12.8, offset: 25 },
  ]);
  assert.deepEqual(availability, { boom: true, boom_jib: true, boom_ext_jib: true });
});

test("the configuration picks the shortest or the longest jib", () => {
  const jibs = [
    { length: 12.8, offset: 5 },
    { length: 7.2, offset: 25 },
  ];
  assert.equal(jibLengthFor(jibs, "boom_jib"), 7.2);
  assert.equal(jibLengthFor(jibs, "boom_ext_jib"), 12.8);
  assert.equal(jibLengthFor([], "boom_ext_jib"), null);
});

test("every configuration option has a label", () => {
  assert.deepEqual(
    JIB_CONFIGS.map(([key]) => key),
    ["boom", "boom_jib", "boom_ext_jib"],
  );
  for (const [, label] of JIB_CONFIGS) assert.ok(label.length > 0);
});
