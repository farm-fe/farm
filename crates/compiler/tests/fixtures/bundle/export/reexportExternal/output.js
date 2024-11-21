//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: foo.ts
import { readFile } from "node:fs";
const foo = 'foo';

// module_id: reexport.ts

// module_id: index.ts
export { foo as bar };
export { readFile as rf } from "node:fs";
