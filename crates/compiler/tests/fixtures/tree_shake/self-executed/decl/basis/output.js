//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_ecb7bd149b01fc3bc5090d82beece659_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "569704c1": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var a1 = 11;
        console.log(a1);
        const aValue = 'a';
        var a = aValue;
        console.log(a);
        {
            let c = 1000;
            console.log(c);
        }
        function AAA() {
            console.log('aaa');
        }
        AAA();
        class Foo {
            constructor(){
                console.log('foo');
            }
        }
        new Foo();
        exports.default = function() {
            console.log('foo');
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_a = farmRequire.i(farmRequire("569704c1"));
        console.log(farmRequire.f(_f_a));
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");