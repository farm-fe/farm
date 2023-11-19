//__farm_runtime.js:
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
},}, "d2214aaa");

//index.js:
 import "./__farm_runtime.js";(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_7d8a.js';
                (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.register(key, modules[key]);
            }
        })({"363fc137": function(module, exports, farmRequire, farmDynamicRequire) {
    console.log("utils.js");
},
"b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    const fs = farmRequire("e4b1dea3");
    const utils = farmRequire("363fc137");
    console.log(fs, utils);
},});(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setInitialLoadedResources(['index_7ecc.js']);(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");

//index_7ecc.js:
 (function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_7ecc.js';
                (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.register(key, modules[key]);
            }
        })({"e4b1dea3": function(module, exports, farmRequire, farmDynamicRequire) {
    console.log("fs-extra");
},});