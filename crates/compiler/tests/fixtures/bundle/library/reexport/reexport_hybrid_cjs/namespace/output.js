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
 // module_id: foo.ts
import { __commonJs, _interop_require_wildcard } from "./farm_runtime.js";
var foo_cjs = __commonJs({
    "foo.ts": (module, exports)=>{
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
            bar: function() {
                return bar;
            },
            default: function() {
                return _default;
            },
            foo: function() {
                return foo;
            }
        });
        var _default = 'foo';
        const foo = 'foo';
        const bar = 'bar';
        module.exports.cjs = true;
    }
});
var foo_ns = _interop_require_wildcard(foo_cjs());

// module_id: index.ts
export { foo_ns as ns };
