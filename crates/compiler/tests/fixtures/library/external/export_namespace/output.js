//index.js:
 import * as os from "node:os";
import * as fs from "node:fs";
; // module_id: dep.ts
; // module_id: index.ts
console.log('dep', fs, fs, os);
export { os as os };
