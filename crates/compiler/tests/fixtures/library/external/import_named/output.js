//index.js:
 import { existsSync as r1, exists as r2, readFile as r1$1, readFileSync as r2$1 } from "node:fs";
; // module_id: dep.js
console.log(r1("dep.js"));
console.log(r2("dep.js"));
; // module_id: index.ts
const readFile = 1;
const readFileSync = 2;
console.log({
    readFile,
    readFileSync,
    r1: r1$1,
    r2: r2$1
});
