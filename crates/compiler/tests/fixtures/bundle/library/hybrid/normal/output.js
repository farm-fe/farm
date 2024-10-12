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
import __farmNodeModule from 'module';
var __nodeRequireInstance = __farmNodeModule.createRequire(import.meta.url);
function _nodeRequire() {
    return __nodeRequireInstance.apply(null, arguments);
}
export { __commonJs, _interop_require_default, _nodeRequire };


//index.js:
 // module_id: index.ts
import { __commonJs, _interop_require_default, _nodeRequire } from "./farm_runtime.js";
import fs from "node:fs";
var index_cjs = __commonJs({
    "index.ts": (module, exports)=>{
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
        const os = _nodeRequire('node:os');
        console.log(fs.read, os.cpus);
        var _default = {
            read: fs.read,
            c: 1
        };
        const foo = 'foo';
        const bar = 'bar';
    }
});
var index_default = _interop_require_default(index_cjs()).index_default, bar = index_cjs()["bar"], foo = index_cjs()["foo"];
export { bar, foo, index_default as default };
