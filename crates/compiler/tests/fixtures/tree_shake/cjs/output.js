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
    "3da733a3": function(module, exports, farmRequire, farmDynamicRequire) {
        module.exports = function() {
            return 'b';
        };
    },
    "a3823798": function(module, exports, farmRequire, farmDynamicRequire) {
        const b = farmRequire("3da733a3");
        function a() {
            return b();
        }
        module.exports = {
            a,
            b
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_cjs_1 = farmRequire.i(farmRequire("a3823798"));
        console.log(farmRequire.f(_f_cjs_1));
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");