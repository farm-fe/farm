//index.js:
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
},}, "d2214aaa");(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_dcdc.js';
                (globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.register(key, modules[key]);
            }
        })({"b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    "";
    console.log("Hello, world!");
},});(globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");

//index_337c.css:
 body {
  color: red;
}