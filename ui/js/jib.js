// Which jib configurations a workbook's jib tables actually offer.
//
// The plain jib is the workbook's shortest jib table and the extended jib its
// longest, so a workbook with one jib length has no "extended jib" to select.
// Both the Crane tab and the Graph tab grey that option out rather than let
// the operator pick a table the chart does not carry.

export const JIB_CONFIGS = [
  ["boom", "Boom length"],
  ["boom_jib", "Boom length + jib"],
  ["boom_ext_jib", "Boom length + extended jib"],
];

/** The distinct jib lengths a workbook offers, shortest first. */
export function jibLengths(jibs) {
  return [...new Set((jibs || []).map((jib) => jib.length))].sort((a, b) => a - b);
}

/** Which configurations the jib tables allow, by configuration key. */
export function jibAvailability(jibs) {
  const lengths = jibLengths(jibs);
  return {
    boom: true,
    boom_jib: lengths.length >= 1,
    boom_ext_jib: lengths.length >= 2,
  };
}

/** The jib length a configuration selects: the shortest or the longest. */
export function jibLengthFor(jibs, config) {
  const lengths = jibLengths(jibs);
  if (!lengths.length) return null;
  return config === "boom_ext_jib" ? lengths[lengths.length - 1] : lengths[0];
}
