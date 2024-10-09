//bundle1.js:
 // module_id: bundle2.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var _interop_require_wildcard = farm_runtime_js_ns._interop_require_wildcard;
var index_1175_js_ns = require("./index.js");
var d = index_1175_js_ns.d, namespace_cjs = index_1175_js_ns.namespace_cjs;
var ns = _interop_require_wildcard(namespace_cjs());
module.exports.d = d;
module.exports.ns = ns;
Object.defineProperty(exports, "__esModule", {
    value: true
});


//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__
function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        if (typeof mod === "function") {
            mod(module, module.exports);
        } else {
            mod[Object.keys(mod)[0]](module, module.exports);
        }
        return module.exports;
    };
}
function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}
function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}
module.exports.__commonJs = __commonJs;
module.exports._interop_require_wildcard = _interop_require_wildcard;
Object.defineProperty(exports, "__esModule", {
    value: true
});


//index.js:
 // module_id: namespace.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var __commonJs = farm_runtime_js_ns.__commonJs;
var bundle1_js_ns = require("./bundle1.js");
var ns = bundle1_js_ns.ns;
var namespace_cjs = __commonJs({
    "namespace.ts": (module, exports)=>{
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        function _export(target, all) {
            for(var name in all)Object.defineProperty(target, name, {
                enumerable: true,
                get: all[name]
            });
        }
        _export(exports, {
            ns_default: function() {
                return ns_default;
            },
            ns_named: function() {
                return ns_named;
            }
        });
        const ns_named = 'ns named';
        const ns_default = 'ns default';
        module.exports.name = '123';
    }
});
var ns_default = namespace_cjs()["ns_default"], ns_named = namespace_cjs()["ns_named"];

// module_id: default.ts
const d = 'default';

// module_id: index.ts
console.log({
    ns: ns,
    bundle2: d
});
module.exports.d = d;
module.exports.namespace_cjs = namespace_cjs;
module.exports.ns_default = ns_default;
module.exports.ns_named = ns_named;
Object.defineProperty(exports, "__esModule", {
    value: true
});
