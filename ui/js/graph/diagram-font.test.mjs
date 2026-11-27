import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

import { DIAGRAM_FONT } from "./diagram-font.js";

const PREFIX = "data:font/woff2;base64,";
// What `pyftsubset ui/fonts/InterVariable.woff2 --unicodes="U+0020-007E,U+00B0"
// --flavor=woff2 --layout-features='*'` produces, as the module records. Pinning
// it means a regenerated base64 is a deliberate change, not a silent one.
const SHA256 = "4a5713d1cbf18b7acdfbe07279a589766976e446fac1ee6faa58fac055440f9f";
const BYTES = 72292;

test("the embedded subset is the reviewed file", () => {
  assert.ok(DIAGRAM_FONT.startsWith(PREFIX));
  const bytes = Buffer.from(DIAGRAM_FONT.slice(PREFIX.length), "base64");
  assert.equal(bytes.length, BYTES);
  assert.equal(createHash("sha256").update(bytes).digest("hex"), SHA256);
});
