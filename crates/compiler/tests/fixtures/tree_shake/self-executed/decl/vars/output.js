//index.js:
 (function(){if(typeof window==='undefined'||!window['__farm_default_namespace__']||!window['__farm_default_namespace__'].m){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_ddf1ac1dd74c1a8307895afa712b1c49_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "foo1", function() {
            return foo1;
        });
        farmRequire.o(exports, "foo2", function() {
            return foo2;
        });
        farmRequire.o(exports, "foo3", function() {
            return foo3;
        });
        farmRequire.o(exports, "foo4", function() {
            return foo4;
        });
        function __farm_live_foo1() {
            return __farm_live_foo1_cache || (__farm_live_foo1_cache = farmRequire.entryBindings.bind("[\"dep.ts\",\"foo1\"]", module, function() {
                return foo1;
            }, function(__FARM_NEXT__) {
                foo1 = __FARM_NEXT__;
            }));
        }
        var __farm_live_foo1_cache;
        function foo() {
            console.log('hello world');
        }
        let foo1 = 1, foo2 = 2;
        __farm_live_foo1().value = 2;
        var foo3 = foo;
        var foo4 = foo;
        foo3.create = foo;
        __farm_live_foo1();
        farmRequire.entryBindings.complete(module, [
            "[\"dep.ts\",\"foo1\"]"
        ]);
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.foo1, _f_dep.foo2, _f_dep.foo3, _f_dep.foo4);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;