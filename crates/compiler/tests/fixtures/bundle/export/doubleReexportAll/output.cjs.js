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
function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}
module.exports.__commonJs = __commonJs;
module.exports._export_star = _export_star;
module.exports._interop_require_wildcard = _interop_require_wildcard;
Object.defineProperty(exports, "__esModule", {
    value: true
});


//index.js:
 // module_id: reexport.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var __commonJs = farm_runtime_js_ns.__commonJs, _export_star = farm_runtime_js_ns._export_star, _interop_require_wildcard = farm_runtime_js_ns._interop_require_wildcard;
var reexport_cjs = __commonJs({
    "reexport.ts": (module, exports)=>{
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        _export_star(require("node:fs"), exports);
        _export_star(require("node:cluster"), exports);
        const readFile = 123;
        module.exports.name = 123;
    }
});
var reexport_ns = _interop_require_wildcard(reexport_cjs()), Worker = reexport_cjs()["Worker"], readFile = reexport_cjs()["readFile"];

// module_id: foo.ts

// module_id: index.ts
console.log({
    readFile: readFile,
    Worker: Worker
});
_export_star(reexport_cjs(), module.exports);
