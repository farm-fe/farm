//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());window['__farm_default_namespace__'].m.se({
    "module": window['module'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "052dab48": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = {
            main: './main.tsx'
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_module = farmRequire('module');
        var _f_config = farmRequire("edceee38");
        var _f_util = farmRequire.i(farmRequire("052dab48"));
        exports.default = _f_config.defineFarmConfig({
            compilation: {
                input: farmRequire.f(_f_util),
                external: _f_module.builtinModules
            }
        });
    },
    "edceee38": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "defineFarmConfig", function() {
            return defineFarmConfig;
        });
        function defineFarmConfig(userConfig) {
            return userConfig;
        }
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_default__=__farm_entry__.default;export {__farm_entry_default__ as default};