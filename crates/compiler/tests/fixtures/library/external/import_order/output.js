//index.js:
 import { dep3 } from "/external/dep3";
import { readFile } from "node:fs";
import { resolve } from "node:path";
import { cpus } from "node:os";
import { spawn } from "node:child_process";
; // module_id: dep3.ts
; // module_id: dep1.ts
; // module_id: dep2.ts
; // module_id: index.ts
console.log(readFile, resolve, cpus, spawn, dep3);
