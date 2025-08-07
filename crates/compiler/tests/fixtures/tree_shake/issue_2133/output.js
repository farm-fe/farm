//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());window['__farm_default_namespace__'].m.se({
    "fake-module": window['fake-module'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire("e3e06d35");
        farmRequire("fe72c3ec");
    },
    "e3e06d35": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_fake_module = farmRequire("fake-module");
        var _a1;
        [_a1] = [
            _f_fake_module.target
        ], _a1.namespace = 234;
        var arrayAssign = new Proxy(_f_fake_module.target.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS, {
            get (target22, prop, receiver) {
                return Reflect.get(target22, prop, receiver);
            }
        });
        console.log({
            arrayAssign
        });
    },
    "fe72c3ec": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_fake_module = farmRequire("fake-module");
        var _a1, _a2;
        (_a2 = (_a1 = _f_fake_module.target).__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS) != null ? _a2 : _a1.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS = [];
        var singleAssign = new Proxy(_f_fake_module.target.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS, {
            get (target22, prop, receiver) {
                return Reflect.get(target22, prop, receiver);
            }
        });
        console.log({
            singleAssign
        });
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");