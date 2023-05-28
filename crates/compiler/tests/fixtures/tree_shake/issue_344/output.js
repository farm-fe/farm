//index.js:
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
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "4ef5be87": function(module, exports, farmRequire, dynamicRequire) {
        module.exports = {
            program: function() {}
        };
    },
    "abc9a879": function(module, exports, farmRequire, dynamicRequire) {
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
            program: function() {
                return program;
            },
            createCommand: function() {
                return createCommand;
            },
            createArgument: function() {
                return createArgument;
            },
            createOption: function() {
                return createOption;
            },
            CommanderError: function() {
                return CommanderError;
            },
            InvalidArgumentError: function() {
                return InvalidArgumentError;
            },
            InvalidOptionArgumentError: function() {
                return InvalidOptionArgumentError;
            },
            Command: function() {
                return Command;
            },
            Argument: function() {
                return Argument;
            },
            Option: function() {
                return Option;
            },
            Help: function() {
                return Help;
            }
        });
        var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
        var _command = _interop_require_default._(farmRequire("4ef5be87"));
        const { program , createCommand , createArgument , createOption , CommanderError , InvalidArgumentError , InvalidOptionArgumentError , Command , Argument , Option , Help  } = _command.default;
    },
    "b5d64806": function(module, exports, farmRequire, dynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        var _esm = farmRequire("abc9a879");
        (0, _esm.program)();
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("b5d64806");
