//bar-a57e94.txt?url:
 bar

//foo-8bdf4c.txt?url:
 foo

//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "27eb6d1d": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = "./foo-8bdf4c.txt?url";
    },
    "8b8d3d28": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = "./bar-a57e94.txt?url";
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_foo = farmRequire.i(farmRequire("27eb6d1d"));
        var _f_bar = farmRequire.i(farmRequire("8b8d3d28"));
        var _f_foo1 = farmRequire.i(farmRequire("27eb6d1d"));
        const foo = 'bar';
        console.log(new URL({
            "./foo/foo.txt": farmRequire.f(_f_foo1)
        }[`./foo/${foo}.txt`], module.meta.url));
        console.log(new URL({
            "./foo/bar/bar.txt": farmRequire.f(_f_bar),
            "./foo/foo.txt": farmRequire.f(_f_foo)
        }[`./foo/${foo}`], module.meta.url));
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");