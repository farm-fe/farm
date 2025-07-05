//index.js:
 import fs$2 from 'node:fs';
import * as fs$1 from 'node:fs';
; // module_id: a.ts
const fs = 'a.ts';
console.log(fs);
; // module_id: b.ts
console.log('b.ts', fs$1);
; // module_id: index.ts
console.log('index.ts', fs$2);
