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
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
module.exports.__commonJs = __commonJs;
module.exports._interop_require_default = _interop_require_default;
module.exports._interop_require_wildcard = _interop_require_wildcard;
Object.defineProperty(exports, "__esModule", {
    value: true
});


//index.js:
 // module_id: lodash.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var __commonJs = farm_runtime_js_ns.__commonJs, _interop_require_default = farm_runtime_js_ns._interop_require_default, _interop_require_wildcard = farm_runtime_js_ns._interop_require_wildcard;
var lodash_cjs = __commonJs({
    "lodash.ts": (module, exports)=>{
        module.exports.name = 'lodash';
        module.exports.default = 'foo';
    }
});
var lodash$1 = _interop_require_default(lodash_cjs()).default, lodash_ns = _interop_require_wildcard(lodash_cjs());

// module_id: a.ts
const lodash = 'a.ts';
console.log(lodash, 'a.ts');
var a_default = 'a.ts';

// module_id: b.ts
console.log('b.ts', lodash_ns);

// module_id: index.ts
console.log('index.ts', lodash$1);
