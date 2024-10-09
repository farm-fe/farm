//bundle1.js:
 // module_id: bundle2.ts
import { _interop_require_wildcard } from "./farm_runtime.js";
import { namespace_cjs } from "./index.js";
var ns = _interop_require_wildcard(namespace_cjs());
export { ns };
export { d } from "./index.js";


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
export { __commonJs, _interop_require_wildcard };


//index.js:
 // module_id: namespace.ts
import { __commonJs } from "./farm_runtime.js";
import { ns } from "./bundle1.js";
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
export { d, namespace_cjs, ns_default, ns_named };
