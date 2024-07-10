//index.js:
 import __farmNodeModule from 'node:module';globalThis.nodeRequire = __farmNodeModule.createRequire(import.meta.url);(globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function _interop_require_default(obj) {
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
}// module_id: esmExport.ts.farm-runtime
const name = 'name';
const age = 18;
var esmExport_ts_default = {
    name: name,
    age: age
};
var esmExport_ts_ns = {
    name: name,
    "default": esmExport_ts_default,
    __esModule: true
};

// module_id: cjsExport.ts.farm-runtime
var cjsExport_ts_cjs = __commonJs({
    "cjsExport.ts.farm-runtime": (module, exports)=>{
        module.exports.age = 18;
        module.exports.default = function() {
            return 'default';
        };
    }
});
var cjsDefault = _interop_require_default(cjsExport_ts_cjs()).default, cjsNs = _interop_require_wildcard(cjsExport_ts_cjs()), cjsNamed = cjsExport_ts_cjs()["age"];

// module_id: esmImprot.ts.farm-runtime
console.log({
    cjsNamed: cjsNamed,
    cjsDefault: cjsDefault,
    cjsNs: cjsNs
});
console.log({
    esmNamed: name,
    esmNs: esmExport_ts_ns,
    esmDefault: esmExport_ts_default
});

// module_id: cjsRequire.ts.farm-runtime
var cjsRequire_ts_cjs = __commonJs({
    "cjsRequire.ts.farm-runtime": (module, exports)=>{
        const esmExport = esmExport_ts_ns;
        const { age } = cjsExport_ts_cjs();
        console.log(esmExport, age);
    }
});

// module_id: runtime.ts.farm-runtime
cjsRequire_ts_cjs();
(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("index.ts");