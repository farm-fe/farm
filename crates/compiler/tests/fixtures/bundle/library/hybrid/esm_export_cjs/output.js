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
export { __commonJs };


//index.js:
 // module_id: index.ts
import { __commonJs } from "./farm_runtime.js";
var index_cjs = __commonJs({
    "index.ts": (module, exports)=>{
        "use strict";
        module.exports = {
            name: 'foo',
            age: 18
        };
    }
});
export default index_cjs();
