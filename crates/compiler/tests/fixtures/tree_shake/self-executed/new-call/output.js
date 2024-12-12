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
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_f1d9.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"4312d062":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "default", function() {
        return F;
    });
    function $e() {
        return {
            seed: (e)=>{
                return e;
            }
        };
    }
    var F = class {
        constructor(e = {}){
            let { randomizer: r = $e() } = e;
            this._randomizer = r;
        }
        get defaultRefDate() {
            return this._defaultRefDate;
        }
        setDefaultRefDate(e = ()=>new Date) {
            typeof e == "function" ? this._defaultRefDate = e : this._defaultRefDate = ()=>new Date(e);
        }
        seed(e = Math.ceil(Math.random() * Number.MAX_SAFE_INTEGER)) {
            return this._randomizer.seed(e), e;
        }
    }, Yt = new F;
}
,
"81077a1f":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "useFullscreen", function() {
        return useFullscreen;
    });
    module.o(exports, "document1", function() {
        return document1;
    });
    const defaultDocument = globalThis.isClient ? window.document : undefined;
    const defaultWindow = globalThis.isClient ? window : undefined;
    const F = {};
    function useFullscreen(target, options = {}) {
        const { document = defaultDocument, autoExit = false } = options;
        return document;
    }
    const { document: { document1 } = defaultWindow.document } = F;
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_dep = module.i(farmRequire("4312d062"));
    var _f_object_assign = farmRequire("81077a1f");
    console.log(module.f(_f_dep), _f_object_assign.useFullscreen, _f_object_assign.document1);
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");