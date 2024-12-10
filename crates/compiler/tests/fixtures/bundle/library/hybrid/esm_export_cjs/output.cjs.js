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
function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}
module.exports.__commonJs = __commonJs;
module.exports._export_star = _export_star;
Object.defineProperty(exports, "__esModule", {
    value: true
});


//index.js:
 // module_id: index.ts
var farm_runtime_js_ns = require("./farm_runtime.js");
var __commonJs = farm_runtime_js_ns.__commonJs, _export_star = farm_runtime_js_ns._export_star;
var index_cjs = __commonJs({
    "index.ts": (module, exports)=>{
        "use strict";
        module.exports = {
            name: 'foo',
            age: 18
        };
    }
});
_export_star(index_cjs(), module.exports);
Object.defineProperty(exports, "__esModule", {
    value: true
});
