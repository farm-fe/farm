const esm = require("../esm.mjs");
const bar = require("./bar.cjs");
module.exports = `bar + ${esm.default} + ${bar.bar}`;
