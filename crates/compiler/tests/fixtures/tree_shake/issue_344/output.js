//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "4ef5be87": function(module, exports, farmRequire, farmDynamicRequire) {
        module.exports = {
            program: function() {}
        };
    },
    "abc9a879": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Argument", function() {
            return Argument;
        });
        farmRequire.o(exports, "Command", function() {
            return Command;
        });
        farmRequire.o(exports, "CommanderError", function() {
            return CommanderError;
        });
        farmRequire.o(exports, "Help", function() {
            return Help;
        });
        farmRequire.o(exports, "InvalidArgumentError", function() {
            return InvalidArgumentError;
        });
        farmRequire.o(exports, "InvalidOptionArgumentError", function() {
            return InvalidOptionArgumentError;
        });
        farmRequire.o(exports, "Option", function() {
            return Option;
        });
        farmRequire.o(exports, "createArgument", function() {
            return createArgument;
        });
        farmRequire.o(exports, "createCommand", function() {
            return createCommand;
        });
        farmRequire.o(exports, "createOption", function() {
            return createOption;
        });
        farmRequire.o(exports, "program", function() {
            return program;
        });
        var _f_command = farmRequire.i(farmRequire("4ef5be87"));
        var { program, createCommand, createArgument, createOption, CommanderError, InvalidArgumentError, InvalidOptionArgumentError, Command, Argument, Option, Help } = farmRequire.f(_f_command);
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_esm = farmRequire("abc9a879");
        _f_esm.program();
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");