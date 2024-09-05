//bundle1.js:
 // module_id: bundle2.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var __commonJs = farm_runtime_js_ns.__commonJs;
var bundle2_cjs = __commonJs({
    "bundle2.ts": (module, exports)=>{
        function lodash() {}
        lodash.merge = function() {};
        const _ = lodash;
        (module.exports = _)._ = _;
    }
});
module.exports.bundle2_cjs = bundle2_cjs;


//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
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
module.exports.__commonJs = __commonJs;
module.exports._interop_require_default = _interop_require_default;
Object.defineProperty(exports, "__esModule", {
    value: true
});


//index.js:
 // module_id: index.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var _interop_require_default = farm_runtime_js_ns._interop_require_default;
var bundle1_js_ns = require("./bundle1.js");
var bundle2_cjs = bundle1_js_ns.bundle2_cjs;
var lodash = _interop_require_default(bundle2_cjs()).default, merge = bundle2_cjs()["merge"];
console.log(lodash, merge);
