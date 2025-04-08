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
index_js_cjs();
})());window['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"fake-module": (window['fake-module']||{}).default && !(window['fake-module']||{}).__esModule ? {...(window['fake-module']||{}),__esModule:true} : window['fake-module']||{}});(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_dfc5.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    farmRequire("e3e06d35");
    farmRequire("fe72c3ec");
}
,
"e3e06d35":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_fake_module = farmRequire("fake-module");
    var _a1;
    [_a1] = [
        _f_fake_module.target
    ], _a1.namespace = 234;
    var arrayAssign = new Proxy(_f_fake_module.target.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS, {
        get (target22, prop, receiver) {
            return Reflect.get(target22, prop, receiver);
        }
    });
    console.log({
        arrayAssign
    });
}
,
"fe72c3ec":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_fake_module = farmRequire("fake-module");
    var _a1, _a2;
    (_a2 = (_a1 = _f_fake_module.target).__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS) != null ? _a2 : _a1.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS = [];
    var singleAssign = new Proxy(_f_fake_module.target.__VUE_DEVTOOLS_KIT_TIMELINE_LAYERS, {
        get (target22, prop, receiver) {
            return Reflect.get(target22, prop, receiver);
        }
    });
    console.log({
        singleAssign
    });
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");