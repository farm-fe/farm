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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_daf2.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"27eb6d1d":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = "./foo-8bdf4c.txt?url";
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_foo = module.i(farmRequire("27eb6d1d"));
    console.log(new URL({
        "./foo/foo.txt": module.f(_f_foo)
    }[`./foo/*`], module.meta.url));
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");