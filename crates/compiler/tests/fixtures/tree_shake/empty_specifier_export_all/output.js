//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_b85a9939f8924ebb03d305e899557040_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep1 = farmRequire("ef0c4c9d");
        farmRequire._e(exports, _f_dep1);
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        farmRequire._e(exports, _f_dep);
        exports.default = 2;
    },
    "ef0c4c9d": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return a;
        });
        farmRequire.o(exports, "b", function() {
            return b;
        });
        var a = '1';
        var b = '2';
        console.log(a, b);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_a__=__farm_entry__.a;var __farm_entry_b__=__farm_entry__.b;var __farm_entry_default__=__farm_entry__.default;export {__farm_entry_a__ as a,__farm_entry_b__ as b,__farm_entry_default__ as default};