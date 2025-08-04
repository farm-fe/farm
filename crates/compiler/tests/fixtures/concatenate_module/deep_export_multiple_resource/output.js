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
        farmRequire.o(exports, "isRef", function() {
            return _f_dep.isRef;
        });
        farmRequire.o(exports, "localIsRef", function() {
            return localIsRef;
        });
        farmRequire.o(exports, "localWatch", function() {
            return localWatch;
        });
        farmRequire.o(exports, "watch", function() {
            return _f_dep.watch;
        });
        var _f_dep = farmRequire("3e3af5b6");
        function localWatch() {
            console.log("local watch");
        }
        function localIsRef() {
            console.log("local isRef");
        }
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_isRef__=__farm_entry__.isRef;var __farm_entry_localIsRef__=__farm_entry__.localIsRef;var __farm_entry_localWatch__=__farm_entry__.localWatch;var __farm_entry_watch__=__farm_entry__.watch;export {__farm_entry_isRef__ as isRef,__farm_entry_localIsRef__ as localIsRef,__farm_entry_localWatch__ as localWatch,__farm_entry_watch__ as watch};