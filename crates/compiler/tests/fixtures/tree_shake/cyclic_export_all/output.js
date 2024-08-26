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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dd58.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"569704c1":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_b = farmRequire("f380ea31");
    module._e(exports, _f_b);
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_a = farmRequire("569704c1");
    console.log(_f_a.c1);
}
,
"c23f7b06":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "d1", function() {
        return d1;
    });
    var d1 = 3;
}
,
"f06623f5":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "c1", function() {
        return c1;
    });
    farmRequire("f380ea31");
    var c1 = 1;
}
,
"f380ea31":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_d = farmRequire("c23f7b06");
    var _f_c = farmRequire("f06623f5");
    module._e(exports, _f_c);
    console.log(_f_d.d1);
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");