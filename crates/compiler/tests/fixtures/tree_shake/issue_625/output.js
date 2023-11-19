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
              modules[key].__farm_resource_pot__ = 'index_2c69.js';
                (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.register(key, modules[key]);
            }
        })({"10c43cb2": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "registerTickMethod", {
        enumerable: true,
        get: function() {
            return registerTickMethod;
        }
    });
    const cache = {};
    function registerTickMethod(id, method) {
        cache[id] = method;
    }
},
"11ecb1ee": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "scaleFunc", {
        enumerable: true,
        get: function() {
            return scaleFunc;
        }
    });
    farmRequire("3e3af5b6");
    function scaleFunc() {
        return "tick";
    }
},
"3e3af5b6": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _register = farmRequire("10c43cb2");
    (0, _register.registerTickMethod)("xxx", ()=>console.log("xxx"));
},
"b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "tick", {
        enumerable: true,
        get: function() {
            return tick;
        }
    });
    var _depindex = farmRequire("11ecb1ee");
    function tick() {
        return (0, _depindex.scaleFunc)();
    }
},});(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");var tick=entry.tick;export { tick };