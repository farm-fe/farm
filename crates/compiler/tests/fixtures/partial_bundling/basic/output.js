//__farm_runtime.js:
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
})());

//debounce_6f74.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='debounce_6f74.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"01609b59":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "default", function() {
        return debounce;
    });
    var _f_utils = farmRequire("a5831d05");
    function debounce(fn) {
        _f_utils.debug("debounce");
        return fn;
    }
}
,});

//index.js:
 import "./__farm_runtime.js";import "./index_2faa.js";import "./index_64d2.js";window['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"module": (window['module']||{}).default && !(window['module']||{}).__esModule ? {...(window['module']||{}),__esModule:true} : window['module']||{}});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_module = module.i(farmRequire('module'));
    var _f_merge = module.i(farmRequire("726cd210"));
    function defineConfig(config) {
        module.f(_f_merge)(config, {
            compilation: {
                input: {
                    main: './main.tsx'
                },
                external: module.f(_f_module).builtinModules
            }
        });
        return config;
    }
    farmDynamicRequire("01609b59").then((debounce)=>{
        console.log(debounce);
    });
    exports.default = defineConfig({});
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources(['index_2faa.js','index_64d2.js']);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([{ path: 'index_2faa.js', type: 0 },{ path: 'debounce_6f74.js', type: 0 }],{ '01609b59': [0,1] });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default || entry;

//index_2faa.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='index_2faa.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"a5831d05":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "debug", function() {
        return debug;
    });
    function debug(msg) {
        console.log(msg);
    }
}
,});

//index_64d2.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='index_64d2.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"726cd210":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "default", function() {
        return merge;
    });
    var _f_utils = farmRequire("a5831d05");
    function merge(a, b) {
        _f_utils.debug("merge");
        return a + b;
    }
}
,});