// Which jib configurations a workbook's tables allow, without a webview.

import test from "node:test";
import assert from "node:assert/strict";

import {
  JIB_CONFIGS,
  jibAngleAtOrBelow,
  jibAvailability,
  jibLengthFor,
  jibLengths,
} from "./jib.js";

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

test("a boom angle between chart rows uses the lower row", () => {
  const angles = [45, 60, 65, 83];
  // 64 deg is read at 60: the 65 row carries more capacity than 64 can claim.
  assert.equal(jibAngleAtOrBelow(angles, 64), 60);
  assert.equal(jibAngleAtOrBelow(angles, 65), 65);
  assert.equal(jibAngleAtOrBelow(angles, 60), 60);
  assert.equal(jibAngleAtOrBelow(angles, 64.999), 60);
  // Below the chart there is no safe row: the lowest is shown, and the check
  // reports the angle as outside the chart.
  assert.equal(jibAngleAtOrBelow(angles, 30), 45);
  // Above the chart the highest row is the conservative one (capacity rises
  // with the angle, so the highest row is still below the true figure).
  assert.equal(jibAngleAtOrBelow(angles, 90), 83);
  assert.equal(jibAngleAtOrBelow([], 64), null);
});
