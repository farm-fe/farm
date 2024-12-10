//index.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};;((function(){function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        if (typeof mod === "function") {
            mod(module, module.exports);
        } else {
            mod[Object.keys(mod)[0]](module, module.exports);
        }
        return module.exports;
    };
}
var index_js_cjs = __commonJs((module, exports)=>{
    "use strict";
    console.log('runtime/index.js');
    window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
index_js_cjs();
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_51e4.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"4ef5be87":function  (module, exports, farmRequire, farmDynamicRequire) {
    module.exports = {
        program: function() {}
    };
}
,
"abc9a879":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "Argument", function() {
        return Argument;
    });
    module.o(exports, "Command", function() {
        return Command;
    });
    module.o(exports, "CommanderError", function() {
        return CommanderError;
    });
    module.o(exports, "Help", function() {
        return Help;
    });
    module.o(exports, "InvalidArgumentError", function() {
        return InvalidArgumentError;
    });
    module.o(exports, "InvalidOptionArgumentError", function() {
        return InvalidOptionArgumentError;
    });
    module.o(exports, "Option", function() {
        return Option;
    });
    module.o(exports, "createArgument", function() {
        return createArgument;
    });
    module.o(exports, "createCommand", function() {
        return createCommand;
    });
    module.o(exports, "createOption", function() {
        return createOption;
    });
    module.o(exports, "program", function() {
        return program;
    });
    var _f_command = module.i(farmRequire("4ef5be87"));
    var { program, createCommand, createArgument, createOption, CommanderError, InvalidArgumentError, InvalidOptionArgumentError, Command, Argument, Option, Help } = module.f(_f_command);
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_esm = farmRequire("abc9a879");
    _f_esm.program();
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");