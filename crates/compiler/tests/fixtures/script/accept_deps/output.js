//index.js:
 (function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_foo = farmRequire("59ebf907");
        var _f_bar = farmRequire("e185e932");
        _f_foo.foo();
        _f_bar.bar();
        module.meta.hot.accept([
            "foo.js",
            "bar.js"
        ], ([newFooModule, newBarModule])=>{});
    },
    "59ebf907": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "foo", function() {
            return foo;
        });
        function foo() {
            return 'foo';
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_foo = farmRequire("59ebf907");
        farmRequire("05ee5ec7");
        _f_foo.foo();
        ;
    },
    "e185e932": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "bar", function() {
            return bar;
        });
        function bar() {
            return 'bar';
        }
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");