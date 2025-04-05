//foo_bar-47689c.txt?url:
 foo_bar

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
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_dff7.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b334ec2f":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = "/foo_bar-47689c.txt?url";
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_foo_bar = module.i(farmRequire("b334ec2f"));
    var _f_foo_bar1 = module.i(farmRequire("b334ec2f"));
    var _f_foo_bar2 = module.i(farmRequire("b334ec2f"));
    const path1 = 'foo';
    const bar = 'bar';
    new URL({
        "./foo/bar/foo_bar.txt": module.f(_f_foo_bar2)
    }[`./foo/${path1}/${bar}`], module.meta.url);
    new URL({}[`./foo/${path1}-${bar}`], module.meta.url);
    new URL({
        "./foo/bar/foo_bar.txt": module.f(_f_foo_bar1)
    }[`./foo/${path1}/**/${bar}`], module.meta.url);
    new URL({
        "./foo/bar/foo_bar.txt": module.f(_f_foo_bar)
    }["./foo/**/*/**"], module.meta.url);
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");