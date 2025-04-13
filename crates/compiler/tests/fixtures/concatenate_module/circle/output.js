//index.js:
 (function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        const foo = reexport + "foo";
        const reexport = "reexport";
        var reexport_mjs_namespace_farm_internal_ = {
            get foo () {
                return foo;
            },
            get reexport () {
                return reexport;
            },
            __esModule: true
        };
        console.log(reexport_mjs_namespace_farm_internal_);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");