//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_d1739fe657d7d37f1ef7779c03d1fec5_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "027594c8": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return Alpha;
        });
        function Alpha() {
            return 'Alpha';
        }
        var _f_exports2 = farmRequire("405aeea7");
        farmRequire._(exports, "b", _f_exports2, "a");
    },
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return MyAlpha;
        });
        var _f_exports1 = farmRequire("027594c8");
        var _f_exports11 = farmRequire("027594c8");
        farmRequire._(exports, "b", _f_exports11, "a");
        farmRequire._(exports, "c", _f_exports11, "b");
        class MyAlpha {
            constructor(){
                return _f_exports1.a;
            }
        }
    },
    "405aeea7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return Checkboard;
        });
        function Checkboard() {
            return 'Checkboard';
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.b, _f_dep.c, _f_dep.a);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;