//bundle1.js:
 // module_id: bundle2.ts
var index_c86a_js_ns = require("./index.js");
var default_default$1 = index_c86a_js_ns.default_default$1, foo_named = index_c86a_js_ns.foo_named, namespace_ns = index_c86a_js_ns.namespace_ns;
console.log({
    bundle2_default: default_default$1,
    bundle2_namespace: namespace_ns,
    bundle2_named: foo_named
});
module.exports.default_default$1 = default_default$1;
module.exports.foo_named = foo_named;
module.exports.namespace_ns = namespace_ns;
Object.defineProperty(exports, "__esModule", {
    value: true
});


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
module.exports.default = index_default;
Object.defineProperty(exports, "__esModule", {
    value: true
});
module.exports.default_default$1 = default_default$1;
module.exports.foo_named = foo_named;
module.exports.foo_ns = foo_ns;
module.exports.namespace_default = namespace_default;
module.exports.namespace_ns = namespace_ns;
