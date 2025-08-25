//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_fcbd0b435115f6df44a35899c2737c57_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "h", function() {
            return re$3;
        });
        farmRequire.o(exports, "r", function() {
            return re$1;
        });
        function re$1() {
            var re = "internal re";
            console.log("re.dep2", re, index_ts_namespace_farm_internal_.h);
        }
        var re$2 = {
            value: "re.dep" + re$1
        };
        var dep_js_namespace_farm_internal_ = {
            get e () {
                return re$2;
            },
            __esModule: true
        };
        function re$3() {
            console.log("re.index");
        }
        console.log(dep_js_namespace_farm_internal_.e.value, re$3);
        var index_ts_namespace_farm_internal_ = {
            get h () {
                return re$3;
            },
            get r () {
                return re$1;
            },
            __esModule: true
        };
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_h__=__farm_entry__.h;var __farm_entry_r__=__farm_entry__.r;export {__farm_entry_h__ as h,__farm_entry_r__ as r};