//index.js:
 import { createRequire as __farmNodeCreateRequire } from "module";
var __farmNodeRequire = __farmNodeCreateRequire(import.meta.url);
import fs from "node:fs";
; // module_id: index.ts
const os = __farmNodeRequire('node:os');
console.log(fs.read, os.cpus);
var index_ts_default = {
    read: fs.read,
    c: 1
};
const foo = 'foo';
const bar = 'bar';
export { bar as bar, index_ts_default as default, foo as foo };
