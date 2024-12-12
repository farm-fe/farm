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
module.exports.__commonJs = __commonJs;
module.exports._interop_require_default = _interop_require_default;
Object.defineProperty(exports, "__esModule", {
    value: true
});


//index.js:
 // module_id: index.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var __commonJs = farm_runtime_js_ns.__commonJs, _interop_require_default = farm_runtime_js_ns._interop_require_default;
var node_fs_ns = require("node:fs");
var fs = _interop_require_default(node_fs_ns).default;
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
        const os = require('node:os');
        console.log(fs.read, os.cpus);
        var _default = {
            read: fs.read,
            c: 1
        };
        const foo = 'foo';
        const bar = 'bar';
    }
});
var index_default = _interop_require_default(index_cjs()).default, bar = index_cjs()["bar"], foo = index_cjs()["foo"];
module.exports.bar = bar;
module.exports.foo = foo;
module.exports.default = index_default;
Object.defineProperty(exports, "__esModule", {
    value: true
});
