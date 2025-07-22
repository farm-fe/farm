//index.js:
 import * as fs from "node:fs";
import * as os from "node:os";
; // module_id: dep.ts
; // module_id: index.ts
console.log('dep', fs, fs, os);
export { os as os };
