//bundle1.js:
 // module_id: bundle2.ts
import { default_default$1, foo_named, namespace_ns } from "./index.js";
console.log({
    bundle2_default: default_default$1,
    bundle2_namespace: namespace_ns,
    bundle2_named: foo_named
});
export { default_default$1, foo_named, namespace_ns };


//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: default.ts
var default_default$1 = '123';

// module_id: namespace.ts
const foo_ns = 'ns_foo';
var namespace_default = 'ns_default';
var namespace_ns = {
    foo_ns: foo_ns,
    "default": namespace_default,
    __esModule: true
};

// module_id: named.ts
const foo_named = '123';

// module_id: index.ts
console.log(default_default$1, foo_named, namespace_ns);
var index_default = 'index';
export { default_default$1, foo_named, foo_ns, namespace_default, namespace_ns };
export { index_default as default };
