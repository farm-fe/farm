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
 // module_id: foo.ts
import { __commonJs } from "./farm_runtime.js";
var foo_cjs = __commonJs({
    "foo.ts": (module, exports)=>{
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "default", {
            enumerable: true,
            get: function() {
                return _default;
            }
        });
        module.exports.cjs = true;
        var _default = 'foo';
    }
});

// module_id: index.ts
var foo_default = foo_cjs()["default"];
export { foo_default as default };
