//dep_8b00.js:
 (function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'dep_8b00.js';
                (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.register(key, modules[key]);
            }
        })({"05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
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
        default: function() {
            return _default;
        },
        dep: function() {
            return dep;
        }
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _dep1 = _interop_require_default._(farmRequire("ef0c4c9d"));
    const dep = "dep";
    function _default() {
        return (0, _dep1.default)();
    }
    console.log("side effect in dep.ts");
},
"ef0c4c9d": function(module, exports, farmRequire, farmDynamicRequire) {
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
},});

//index.js:
 (globalThis || window || global || self).__farm_namespace__ = '__farm_default_namespace__';(globalThis || window || global || self)[__farm_namespace__] = {__FARM_TARGET_ENV__: 'browser'};(function (modules, entryModule) {
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
    console.log("runtime/index.js")(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setPlugins([]);
},}, "d2214aaa");(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_dcdc.js';
                (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.register(key, modules[key]);
            }
        })({"b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    farmDynamicRequire("05ee5ec7").then((dep)=>{
        console.log(dep);
    });
},});(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setDynamicModuleResourcesMap({ '05ee5ec7': [{ path: 'dep_8b00.js', type: 'script' },] });var farmModuleSystem = (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");