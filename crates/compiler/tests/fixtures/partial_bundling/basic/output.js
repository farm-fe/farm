//__farm_runtime.8774b52b.mjs:
 (globalThis || window || self || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function (modules, entryModule) {
            var cache = {};

            function dynamicRequire(id) {
              return Promise.resolve(require(id));
            }
          
            function require(id) {
              if (cache[id]) return cache[id].exports;
          
              var module = {
                id: id,
                exports: {}
              };
          
              modules[id](module, module.exports, require, dynamicRequire);
              cache[id] = module;
              return module.exports;
            }
          
            require(entryModule);
          })({"d2214aaa": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    console.log("runtime/index.js")(globalThis || window || self || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
},}, "d2214aaa");

//debounce_6f74.js:
 (function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'debounce_6f74.js';
                (globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.register(key, modules[key]);
            }
        })({"01609b59": function(module, exports, farmRequire, farmDynamicRequire) {
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
},});

//index.js:
 import "./__farm_runtime.8774b52b.mjs";import "./index_2faa.js";import "./index_64d2.js";(globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.setExternalModules({ "module": { ...((globalThis || window || self || {})['module'] || {}), __esModule: true } });(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_dcdc.js';
                (globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.register(key, modules[key]);
            }
        })({"b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
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
},});(globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources(['index_2faa.js','index_64d2.js']);(globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({ '01609b59': [{ path: 'debounce_6f74.js', type: 'script' },{ path: 'index_2faa.js', type: 'script' },] });var farmModuleSystem = (globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default || entry;

//index_2faa.js:
 (function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_2faa.js';
                (globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.register(key, modules[key]);
            }
        })({"a5831d05": function(module, exports, farmRequire, farmDynamicRequire) {
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
},});

//index_64d2.js:
 (function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_64d2.js';
                (globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.register(key, modules[key]);
            }
        })({"726cd210": function(module, exports, farmRequire, farmDynamicRequire) {
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
},});