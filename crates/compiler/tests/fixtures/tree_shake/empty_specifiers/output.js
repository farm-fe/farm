//index.js:
 (globalThis || window || global || self).__farm_namespace__ = '__farm_default_namespace__';(globalThis || window || global || self)[__farm_namespace__] = {__FARM_TARGET_ENV__: 'browser'};(function(modules, entryModule) {
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
    "d2214aaa": function(module, exports, farmRequire, farmDynamicRequire) {
        "use strict";
        console.log("runtime/index.js")(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setPlugins([]);
    }
}, "d2214aaa");
(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setDynamicModuleResourcesMap({  });(function(modules) {
    for(var key in modules){
        var __farm_global_this__ = (globalThis || window || global || self)[__farm_namespace__];
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "6f462555": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "";
        var _default = "comp";
    },
    "b3d9bc98": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log("resolved.ts");
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
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
        var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
        farmRequire("b3d9bc98");
        var _comp = _interop_require_default._(farmRequire("6f462555"));
        console.log(_comp.default);
        var _default = 2;
    }
});
var farmModuleSystem = (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default || entry;

//index_337c.css:
 .body {
  color: red;
}