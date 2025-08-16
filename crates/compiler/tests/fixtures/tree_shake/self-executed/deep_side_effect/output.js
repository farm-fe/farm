//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_d7f65b11ca619511c3f5c90c725d0678_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "446ec84b": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "result", function() {
            return farmRequire.f(_f_use);
        });
        farmRequire("bebcbd1b");
        var _f_use = farmRequire.i(farmRequire("e0004d19"));
    },
    "8fb552f8": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "set", function() {
            return set;
        });
        farmRequire.o(exports, "get", function() {
            return get;
        });
        let cache = {};
        function set(key, obj) {
            cache[key] = obj;
        }
        function get(key) {
            return cache[key];
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_export = farmRequire("446ec84b");
        console.log(_f_export.result);
    },
    "bebcbd1b": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_cache = farmRequire("8fb552f8");
        _f_cache.set('1', {
            a: 1
        });
        _f_cache.set('2', {
            a: 2
        });
        _f_cache.set('3', {
            a: 3
        });
        _f_cache.set('4', {
            a: 4
        });
        _f_cache.set('5', {
            a: 5
        });
        _f_cache.set('6', {
            a: 6
        });
    },
    "e0004d19": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_cache = farmRequire("8fb552f8");
        console.log(_f_cache.get('1').a);
        const r = _f_cache.get('1').a;
        exports.default = r;
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");