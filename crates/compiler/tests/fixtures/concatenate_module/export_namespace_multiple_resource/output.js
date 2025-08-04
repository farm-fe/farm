//dep.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "3e3af5b6": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "watch", function() {
            return watch;
        });
        farmRequire.o(exports, "isRef", function() {
            return isRef;
        });
        function watch() {
            console.log("watch");
        }
        function isRef() {
            console.log("isRef");
        }
    }
});


//farm_internal_runtime_index.js:
 const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);


//index.js:
 import "./farm_internal_runtime_index.js";import "./dep.js";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "reexport2", function() {
            return reexport2_js_namespace_farm_internal_;
        });
        var _f_dep = farmRequire("3e3af5b6");
        var reexport_js_namespace_farm_internal_ = {
            isRef: _f_dep.isRef,
            watch: _f_dep.watch,
            __esModule: true
        };
        function localWatch() {
            console.log("local watch");
        }
        var reexport2_js_namespace_farm_internal_ = {
            localWatch: localWatch,
            watch: _f_dep.watch,
            __esModule: true
        };
        reexport_js_namespace_farm_internal_.watch();
        reexport_js_namespace_farm_internal_.isRef();
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_reexport2__=__farm_entry__.reexport2;export {__farm_entry_reexport2__ as reexport2};