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
    "569704c1": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        (function() {
            const iife_foo = 'iife-foo';
        })();
        var UrlType;
        (function(UrlType) {
            UrlType[UrlType['Empty'] = 1] = 'Empty';
            UrlType[UrlType['Hash'] = 2] = 'Hash';
            UrlType[UrlType['Query'] = 3] = 'Query';
            UrlType[UrlType['RelativePath'] = 4] = 'RelativePath';
            UrlType[UrlType['AbsolutePath'] = 5] = 'AbsolutePath';
            UrlType[UrlType['SchemeRelative'] = 6] = 'SchemeRelative';
            UrlType[UrlType['Absolute'] = 7] = 'Absolute';
        })(UrlType || (UrlType = {}));
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