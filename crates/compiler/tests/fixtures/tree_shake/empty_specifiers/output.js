//5b7c213a.js:
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
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "default", {
            enumerable: true,
            get: function() {
                return _default;
            }
        });
        require("resolved.ts");
        var _default = 2;
    },
    "resolved.ts": function(module, exports, require, dynamicRequire) {
        "use strict";
        console.log("resolved.ts");
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("index.ts").default;
export default entry;


