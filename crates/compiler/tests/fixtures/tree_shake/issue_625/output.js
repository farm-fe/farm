//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_2c69066cb6d742fcfdd77bb73baaade5_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "10c43cb2": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "registerTickMethod", function() {
            return registerTickMethod;
        });
        const cache = {};
        function registerTickMethod(id, method) {
            cache[id] = method;
        }
    },
    "11ecb1ee": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "scaleFunc", function() {
            return scaleFunc;
        });
        farmRequire("3e3af5b6");
        function scaleFunc() {
            return 'tick';
        }
    },
    "3e3af5b6": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_register = farmRequire("10c43cb2");
        _f_register.registerTickMethod('xxx', ()=>console.log('xxx'));
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "tick", function() {
            return tick;
        });
        var _f_dep_index = farmRequire("11ecb1ee");
        function tick() {
            _f_dep_index.scaleFunc();
        }
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_tick__=__farm_entry__.tick;export {__farm_entry_tick__ as tick};