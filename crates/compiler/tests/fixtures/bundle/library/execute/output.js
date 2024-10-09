//bundle1.js:
 // module_id: bundle2.ts
console.log('hello world');
var bundle2_default = {};
export { bundle2_default };


//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: index.ts
import "./bundle1.js";
