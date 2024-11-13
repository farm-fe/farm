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
            foo: function() {
                return foo;
            }
        });
        const foo = 'foo';
        const bar = 'bar';
        module.exports.cjs = true;
    }
});
var bar = foo_cjs()["bar"], foo = foo_cjs()["foo"];

// module_id: index.ts
export { bar, foo };
