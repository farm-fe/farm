//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: a.ts
import fs$1, * as node_fs_ns from "node:fs";
const fs = 'a.ts';
console.log(fs);
var a_default = 'a.ts';

// module_id: b.ts
console.log('b.ts', node_fs_ns);

// module_id: index.ts
console.log('index.ts', fs$1);
