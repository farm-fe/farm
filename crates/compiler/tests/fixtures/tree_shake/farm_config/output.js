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
    "edceee38": function(module, exports, require, dynamicRequire) {
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
    },
    "b5d64806": function(module, exports, require, dynamicRequire) {
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
        var _interopRequireDefault = require("@swc/helpers/lib/_interop_require_default.js").default;
        var _module = require("module");
        var _config = require("edceee38");
        var _util = _interopRequireDefault(require("052dab48"));
        var _default = (0, _config.defineFarmConfig)({
            compilation: {
                input: _util.default,
                external: _module.builtinModules
            }
        });
    },
    "052dab48": function(module, exports, require, dynamicRequire) {
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
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("b5d64806").default;
export default entry;
