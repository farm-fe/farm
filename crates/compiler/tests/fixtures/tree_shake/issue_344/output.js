//index.js:
 (globalThis || window || global || self).__farm_namespace__ = '__farm_default_namespace__';(globalThis || window || global || self)[__farm_namespace__] = {__FARM_TARGET_ENV__: 'browser'};(function (modules, entryModule) {
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
          })({"d2214aaa": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    console.log("runtime/index.js")(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setPlugins([]);
},}, "d2214aaa");(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setDynamicModuleResourcesMap({  });(function (modules) {
        for (var key in modules) {
          var __farm_global_this__ = (globalThis || window || global || self)[
            __farm_namespace__
          ];
          modules[key].__farm_resource_pot__ = 'index_51e4.js';
          __farm_global_this__.__farm_module_system__.register(key, modules[key]);
        }
      })({"4ef5be87": function(module, exports, farmRequire, farmDynamicRequire) {
    module.exports = {
        program: function() {}
    };
},
"abc9a879": function(module, exports, farmRequire, farmDynamicRequire) {
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
},
"b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _esm = farmRequire("abc9a879");
    (0, _esm.program)();
},});var farmModuleSystem = (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");