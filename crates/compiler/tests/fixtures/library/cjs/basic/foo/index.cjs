const esm = require("../esm.mjs");
const curEsm = require("./esm.mjs");

console.log("foo should be executed before zoo");
const zoo = require("../zoo.cjs");

module.exports = `foo + ${esm.default} + ${curEsm.default} + ${zoo.zoo}`;
