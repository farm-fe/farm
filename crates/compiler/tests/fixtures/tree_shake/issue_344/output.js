//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"d2214aaa":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"d2214aaa");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_51e4.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"4ef5be87":function  (module, exports, farmRequire, farmDynamicRequire) {
    module.exports = {
        program: function() {}
    };
}
,
"abc9a879":function  (module, exports, farmRequire, farmDynamicRequire) {
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
        Argument: function() {
            return Argument;
        },
        Command: function() {
            return Command;
        },
        CommanderError: function() {
            return CommanderError;
        },
        Help: function() {
            return Help;
        },
        InvalidArgumentError: function() {
            return InvalidArgumentError;
        },
        InvalidOptionArgumentError: function() {
            return InvalidOptionArgumentError;
        },
        Option: function() {
            return Option;
        },
        createArgument: function() {
            return createArgument;
        },
        createCommand: function() {
            return createCommand;
        },
        createOption: function() {
            return createOption;
        },
        program: function() {
            return program;
        }
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _command = _interop_require_default._(farmRequire("4ef5be87"));
    const { program, createCommand, createArgument, createOption, CommanderError, InvalidArgumentError, InvalidOptionArgumentError, Command, Argument, Option, Help } = _command.default;
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _esm = farmRequire("abc9a879");
    (0, _esm.program)();
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");