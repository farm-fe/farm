//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"ec853507":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"ec853507");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_f1d9.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"4312d062":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return F;
        }
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
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    function _export(target, all) {
        for(var name in all)Object.defineProperty(target, name, {
            enumerable: true,
            get: all[name]
        });
    }
    _export(exports, {
        document1: function() {
            return document1;
        },
        useFullscreen: function() {
            return useFullscreen;
        }
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
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _dep = _interop_require_default._(farmRequire("4312d062"));
    var _objectassign = farmRequire("81077a1f");
    console.log(_dep.default, _objectassign.useFullscreen, _objectassign.document1);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");