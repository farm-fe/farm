//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__
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
module.exports._interop_require_default = _interop_require_default;
module.exports._interop_require_wildcard = _interop_require_wildcard;
Object.defineProperty(exports, "__esModule", {
    value: true
});


//index.js:
 // module_id: a.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var _interop_require_default = farm_runtime_js_ns._interop_require_default, _interop_require_wildcard = farm_runtime_js_ns._interop_require_wildcard;
var node_fs_ns = _interop_require_wildcard(require("node:fs"));
var fs$1 = _interop_require_default(node_fs_ns).default;
const fs = 'a.ts';
console.log(fs);
var a_default = 'a.ts';

// module_id: b.ts
console.log('b.ts', node_fs_ns);

// module_id: index.ts
console.log('index.ts', fs$1);
