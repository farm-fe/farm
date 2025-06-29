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
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_exports = farmRequire("52961596");
        farmRequire._e(exports, _f_exports);
    },
    "52961596": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return Provider;
        });
        farmRequire.o(exports, "b", function() {
            return useDispatch;
        });
        farmRequire.o(exports, "c", function() {
            return useStore;
        });
        function Provider() {
            return 'Provider';
        }
        function useDispatch() {
            return 'useDispatch';
        }
        function useStore() {
            return 'useStore';
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.a);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");