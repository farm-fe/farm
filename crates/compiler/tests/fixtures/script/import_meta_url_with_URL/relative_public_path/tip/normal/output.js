//bar-a57e94.txt?url:
 bar

//foo-8bdf4c.txt?url:
 foo

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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_5425.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"27eb6d1d":function  (module, exports, farmRequire, farmDynamicRequire) {
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