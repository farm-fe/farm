//dep_8b00.js:
 (function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("dep_8b00.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "dep", function() {
        return dep;
    });
    var _f_dep1 = module.i(farmRequire("ef0c4c9d"));
    var dep = 'dep';
    exports.default = function() {
        return module.f(_f_dep1)();
    };
    console.log('side effect in dep.ts');
}
,
"ef0c4c9d":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = function() {
        console.log('1111');
    };
}
,});

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
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_dcdc.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    farmDynamicRequire("05ee5ec7").then((dep)=>{
        console.log(dep);
    });
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([{ path: 'dep_8b00.js', type: 0 }],{ '05ee5ec7': [0] });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");