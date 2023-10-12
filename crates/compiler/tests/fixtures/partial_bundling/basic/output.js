//debounce_6f74.js:
 (function(modules) {
    for(var key in modules){
        var __farm_global_this__ = (globalThis || window || global || self)[__farm_namespace__];
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "01609b59": function(module, exports, farmRequire, farmDynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "default", {
            enumerable: true,
            get: function() {
                return debounce;
            }
        });
        var _utils = farmRequire("a5831d05");
        function debounce(fn) {
            (0, _utils.debug)("debounce");
            return fn;
        }
    }
});


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
(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setInitialLoadedResources(['index_2faa.js','index_64d2.js']);(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setDynamicModuleResourcesMap({ '01609b59': [{ path: 'debounce_6f74.js', type: 'script' },{ path: 'index_2faa.js', type: 'script' },] });(function(modules) {
    for(var key in modules){
        var __farm_global_this__ = (globalThis || window || global || self)[__farm_namespace__];
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
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
        var _module = _interop_require_default._(farmRequire("module"));
        var _merge = _interop_require_default._(farmRequire("726cd210"));
        function defineConfig(config) {
            (0, _merge.default)(config, {
                compilation: {
                    input: {
                        main: "./main.tsx"
                    },
                    external: _module.default.builtinModules
                }
            });
            return config;
        }
        farmDynamicRequire("01609b59").then((debounce)=>{
            console.log(debounce);
        });
        var _default = defineConfig({});
    }
});
var farmModuleSystem = (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default;

//index_2faa.js:
 (function(modules) {
    for(var key in modules){
        var __farm_global_this__ = (globalThis || window || global || self)[__farm_namespace__];
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "a5831d05": function(module, exports, farmRequire, farmDynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "debug", {
            enumerable: true,
            get: function() {
                return debug;
            }
        });
        function debug(msg) {
            console.log(msg);
        }
    }
});


//index_64d2.js:
 (function(modules) {
    for(var key in modules){
        var __farm_global_this__ = (globalThis || window || global || self)[__farm_namespace__];
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "726cd210": function(module, exports, farmRequire, farmDynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "default", {
            enumerable: true,
            get: function() {
                return merge;
            }
        });
        var _utils = farmRequire("a5831d05");
        function merge(a, b) {
            (0, _utils.debug)("merge");
            return a + b;
        }
    }
});
