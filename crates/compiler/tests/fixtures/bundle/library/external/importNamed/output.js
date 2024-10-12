//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: index.ts
import { readFile as readFile$1, readFileSync as readFileSync$1 } from "node:fs";
const readFile = 1;
const readFileSync = 2;
console.log({
    readFile: readFile,
    readFileSync: readFileSync,
    r1: readFile$1,
    r2: readFileSync$1
});
