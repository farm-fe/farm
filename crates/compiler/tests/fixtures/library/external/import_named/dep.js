import { existsSync as r1, exists as r2 } from "node:fs";

console.log(r1("dep.js"));
console.log(r2("dep.js"));
