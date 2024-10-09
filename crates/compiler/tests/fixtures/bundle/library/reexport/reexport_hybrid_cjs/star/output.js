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
 // module_id: foo.ts
import { __commonJs, _interop_require_default } from "./farm_runtime.js";
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

// module_id: index.ts
var foo_default = _interop_require_default(foo_cjs()).default, bar = foo_cjs()["bar"], foo = foo_cjs()["foo"];
export { bar, foo, foo_default as default };
