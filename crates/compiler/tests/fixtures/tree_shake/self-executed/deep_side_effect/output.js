//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};function _interop_require_default(obj) {
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
    console.log('runtime/index.js')(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_d7f6.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"446ec84b":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "result", function() {
        return module.f(_f_use);
    });
    farmRequire("bebcbd1b");
    var _f_use = module.i(farmRequire("e0004d19"));
}
,
"8fb552f8":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "set", function() {
        return set;
    });
    module.o(exports, "get", function() {
        return get;
    });
    let cache = {};
    function set(key, obj) {
        cache[key] = obj;
    }
    function get(key) {
        return cache[key];
    }
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_export = farmRequire("446ec84b");
    console.log(_f_export.result);
}
,
"bebcbd1b":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_cache = farmRequire("8fb552f8");
    _f_cache.set('1', {
        a: 1
    });
    _f_cache.set('2', {
        a: 2
    });
    _f_cache.set('3', {
        a: 3
    });
    _f_cache.set('4', {
        a: 4
    });
    _f_cache.set('5', {
        a: 5
    });
    _f_cache.set('6', {
        a: 6
    });
}
,
"e0004d19":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_cache = farmRequire("8fb552f8");
    console.log(_f_cache.get('1').a);
    const r = _f_cache.get('1').a;
    exports.default = r;
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");