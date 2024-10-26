//bundle1.js:
 // module_id: bundle2_namespace.ts
const ns_named = 'ns named';
const ns_default = 'ns default';
var bundle2_namespace_ns = {
    ns_named: ns_named,
    ns_default: ns_default,
    __esModule: true
};

// module_id: bundle2.ts
export { bundle2_namespace_ns };


//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: index.ts
import { bundle2_namespace_ns } from "./bundle1.js";
console.log({
    ns: bundle2_namespace_ns
});
