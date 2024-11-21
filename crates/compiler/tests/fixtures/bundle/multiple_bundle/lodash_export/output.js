//bundle1.js:
 // module_id: bundle2.ts
import { __commonJs } from "./farm_runtime.js";
var bundle2_cjs = __commonJs({
    "bundle2.ts": (module, exports)=>{
        function lodash() {}
        lodash.merge = function() {};
        const _ = lodash;
        (module.exports = _)._ = _;
    }
});
export { bundle2_cjs };


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
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
export { __commonJs, _interop_require_default };


//index.js:
 // module_id: index.ts
import { _interop_require_default } from "./farm_runtime.js";
import { bundle2_cjs } from "./bundle1.js";
var lodash = _interop_require_default(bundle2_cjs()).default, merge = bundle2_cjs()["merge"];
console.log(lodash, merge);
