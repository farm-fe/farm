//index.js:
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
    "d2214aaa": function(module, exports, require, dynamicRequire) {
        "use strict";
        console.log("runtime/index.js");
        __farm_global_this__.__farm_module_system__.setPlugins([]);
    }
}, "d2214aaa");
(function(modules) {
    for(var key in modules){
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "b5d64806": function(module, exports, require, dynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        noop();
        console.log("Hello, world!");
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("b5d64806").default;
export default entry;


//92a5e8c3.css:
 body {
  color: red;
}