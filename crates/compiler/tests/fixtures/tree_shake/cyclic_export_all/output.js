//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_dd58a7faaf33a8325980cddfccb56a6f_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "569704c1": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_b = farmRequire("f380ea31");
        farmRequire._e(exports, _f_b);
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_a = farmRequire("569704c1");
        console.log(_f_a.c1);
    },
    "c23f7b06": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "d1", function() {
            return d1;
        });
        var d1 = 3;
    },
    "f06623f5": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "c1", function() {
            return c1;
        });
        farmRequire("f380ea31");
        var c1 = 1;
    },
    "f380ea31": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_d = farmRequire("c23f7b06");
        var _f_c = farmRequire("f06623f5");
        farmRequire._e(exports, _f_c);
        console.log(_f_d.d1);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;