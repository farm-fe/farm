//index.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};;((function(){// module_id: ../../../_internal/runtime/index.js.farm-runtime
function __commonJs(mod) {
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
var index_js_cjs = __commonJs({
    "../../../_internal/runtime/index.js.farm-runtime": (module, exports)=>{
        "use strict";
        console.log('runtime/index.js');
        window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
    }
});
index_js_cjs();
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_830e.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"foo.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "foo", function() {
        return foo;
    });
    var _f_reexport = farmRequire("reexport.ts");
    var foo = _f_reexport.reexport + 'foo';
}
,
"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_reexport = module.w(farmRequire("reexport.ts"));
    var ns = _f_reexport;
    console.log(ns);
}
,
"reexport.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "reexport", function() {
        return reexport;
    });
    var reexport = 'reexport';
    var _f_foo = farmRequire("foo.ts");
    module._e(exports, _f_foo);
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("index.ts");