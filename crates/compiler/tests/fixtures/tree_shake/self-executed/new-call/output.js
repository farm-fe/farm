//index.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}function __commonJs(mod) {
  var module;
  return () => {
    if (module) {
      return module.exports;
    }
    module = {
      exports: {},
    };
    if(typeof mod === "function") {
      mod(module, module.exports);
    }else {
      mod[Object.keys(mod)[0]](module, module.exports);
    }
    return module.exports;
  };
}((function(){var index_js_cjs = __commonJs((module, exports)=>{
    "use strict";
    console.log('runtime/index.js');
    window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_f1d9.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"4312d062":function  (module, exports, farmRequire, farmDynamicRequire) {
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