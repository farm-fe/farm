//bar-a57e94.txt?url:
 bar

//foo-8bdf4c.txt?url:
 foo

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
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_5425.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"27eb6d1d":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = "./foo-8bdf4c.txt?url";
}
,
"8b8d3d28":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = "./bar-a57e94.txt?url";
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_foo = module.i(farmRequire("27eb6d1d"));
    var _f_bar = module.i(farmRequire("8b8d3d28"));
    var _f_foo1 = module.i(farmRequire("27eb6d1d"));
    const foo = 'bar';
    console.log(new URL({
        "./foo/foo.txt": module.f(_f_foo1)
    }[`./foo/${foo}.txt`], module.meta.url));
    console.log(new URL({
        "./foo/bar/bar.txt": module.f(_f_bar),
        "./foo/foo.txt": module.f(_f_foo)
    }[`./foo/${foo}`], module.meta.url));
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");