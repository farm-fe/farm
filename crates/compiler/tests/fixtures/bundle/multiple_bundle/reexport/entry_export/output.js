//bundle1.js:
 // module_id: bundle2.ts
const named = 'bundle2 named';
var bundle2_default = 'bundle2 default';
export { bundle2_default, named };


//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: index.ts
import { bundle2_default, named } from "./bundle1.js";
export { bundle2_default as default, named };
