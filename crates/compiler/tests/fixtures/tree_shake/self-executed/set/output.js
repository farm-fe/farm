//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function (modules, entryModule) {
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
          })({"ec853507": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
console.log("runtime/index.js")(globalThis || window || self || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);

},}, "ec853507");(function (modules) {
            for (var key in modules) {
<<<<<<< HEAD:crates/compiler/tests/fixtures/tree_shake/self-executed/output.js
              modules[key].__farm_resource_pot__ = 'index_ddf1.js';
                (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(key, modules[key]);
=======
              modules[key].__farm_resource_pot__ = 'index_ecb7.js';
                (globalThis || window || self || global)['__farm_default_namespace__'].__farm_module_system__.register(key, modules[key]);
>>>>>>> f76b04db (feat: statement analyze & used_export map):crates/compiler/tests/fixtures/tree_shake/self-executed/set/output.js
            }
        })({"569704c1": function(module, exports, farmRequire, farmDynamicRequire) {
// prototype
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
    console.log("a");
}

},
"b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
var _a = /*#__PURE__*/ _interop_require_default._(farmRequire("569704c1"));
console.log(_a.default);

},});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");