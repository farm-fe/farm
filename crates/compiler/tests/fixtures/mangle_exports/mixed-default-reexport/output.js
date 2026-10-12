//index.js:
 (function(){if(typeof window==='undefined'||!window['__farm_default_namespace__']||!window['__farm_default_namespace__'].m){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_ddf1ac1dd74c1a8307895afa712b1c49_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return named;
        });
        exports.default = 42;
        const named = 7;
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire.w(farmRequire("05ee5ec7"));
        farmRequire._(exports, "default", _f_dep);
        farmRequire._(exports, "named", _f_dep, "a");
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_default__=__farm_entry__.default;var __farm_entry_named__=__farm_entry__.named;export {__farm_entry_default__ as default,__farm_entry_named__ as named};