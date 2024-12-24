//index.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};;((function(){function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        if (typeof mod === "function") {
            mod(module, module.exports);
        } else {
            mod[Object.keys(mod)[0]](module, module.exports);
        }
        return module.exports;
    };
}
var index_js_cjs = __commonJs((module, exports)=>{
    "use strict";
    console.log('runtime/index.js');
    window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
index_js_cjs();
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_ecb7.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"569704c1":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
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
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_a = module.i(farmRequire("569704c1"));
    console.log(module.f(_f_a));
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");