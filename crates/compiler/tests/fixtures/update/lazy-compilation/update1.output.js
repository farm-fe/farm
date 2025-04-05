window['__farm_default_namespace__'].m._rg=true;(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "dep.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = 'dep';
    },
    "dep.ts.farm_dynamic_import_virtual_module": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire.w(farmRequire("dep.ts"));
        var ns = _f_dep;
        module.exports = ns;
    }
});
window['__farm_default_namespace__'].m._rg=false;
