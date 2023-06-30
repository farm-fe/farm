//index.js:
 var entry = function() {
    var __farm_global_this__ = {
        __FARM_TARGET_ENV__: "browser"
    };
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
        "d2214aaa": function(module, exports, farmRequire, dynamicRequire) {
            "use strict";
            console.log("runtime/index.js");
            __farm_global_this__.__farm_module_system__.setPlugins([]);
        }
    }, "d2214aaa");
    (function(modules) {
        for(var key in modules){
            __farm_global_this__.__farm_module_system__.register(key, modules[key]);
        }
    })({
        "052dab48": function(module, exports, farmRequire, dynamicRequire) {
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
            var _default = {
                main: "./main.tsx"
            };
        },
        "b5d64806": function(module, exports, farmRequire, dynamicRequire) {
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
            var _module = farmRequire("module");
            var _config = farmRequire("edceee38");
            var _util = _interop_require_default._(farmRequire("052dab48"));
            var _default = (0, _config.defineFarmConfig)({
                compilation: {
                    input: _util.default,
                    external: _module.builtinModules
                }
            });
        },
        "edceee38": function(module, exports, farmRequire, dynamicRequire) {
            "use strict";
            Object.defineProperty(exports, "__esModule", {
                value: true
            });
            Object.defineProperty(exports, "defineFarmConfig", {
                enumerable: true,
                get: function() {
                    return defineFarmConfig;
                }
            });
            function defineFarmConfig(userConfig) {
                return userConfig;
            }
        }
    });
    var farmModuleSystem = __farm_global_this__.__farm_module_system__;
    farmModuleSystem.bootstrap();
    return farmModuleSystem.require("b5d64806");
}();
export default entry.default;
