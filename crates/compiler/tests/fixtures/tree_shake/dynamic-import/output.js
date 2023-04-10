//8fbe243c.js:
 import module from "node:module";
global.__farmNodeRequire = module.createRequire(import.meta.url);
global.__farmNodeBuiltinModules = module.builtinModules;
(function(modules, entryModule) {
    var cache = {};
    function require(id) {
        if (cache[id]) return cache[id].exports;
        var module = {
            id: id,
            exports: {}
        };
        modules[id](module, module.exports, require);
        cache[id] = module;
        return module.exports;
    }
    require(entryModule);
})({
    "../../_internal/runtime/index.js.farm-runtime": function(module, exports, require, dynamicRequire) {
        "use strict";
        console.log("runtime/index.js");
        __farm_global_this__.__farm_module_system__.setPlugins([]);
    }
}, "../../_internal/runtime/index.js.farm-runtime");
(function(modules) {
    for(var key in modules){
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "index.ts": function(module, exports, require, dynamicRequire) {
        "use strict";
        dynamicRequire("dep.ts").then((dep)=>{
            console.log(dep);
        });
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("index.ts").default;
export default entry;


//e79a22cc.js:
 (function(modules) {
    for(var key in modules){
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "dep.ts": function(module, exports, require, dynamicRequire) {
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
            dep: function() {
                return dep;
            },
            default: function() {
                return _default;
            }
        });
        var _interopRequireDefault = require("@swc/helpers/lib/_interop_require_default.js").default;
        var _dep1 = _interopRequireDefault(require("dep1.ts"));
        const dep = "dep";
        function _default() {
            return (0, _dep1.default)();
        }
        console.log("side effect in dep.ts");
    },
    "dep1.ts": function(module, exports, require, dynamicRequire) {
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
        function _default() {
            console.log("1111");
        }
    }
});
