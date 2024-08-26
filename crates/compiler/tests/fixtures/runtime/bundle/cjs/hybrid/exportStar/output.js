//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function _interop_require_default(obj) {
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
}function _mergeNamespaces(n, m) {
    m.forEach(function (e) {
        e && typeof e !== 'string' && !Array.isArray(e) && Object.keys(e).forEach(function (k) {
            if (k !== 'default' && !(k in n)) {
                var d = Object.getOwnPropertyDescriptor(e, k);
                Object.defineProperty(n, k, d.get ? d : {
                    enumerable: true,
                    get: function () { return e[k]; }
                });
            }
        });
    });
    return Object.freeze(n);
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
}// module_id: cjsExport.ts.farm-runtime
import * as node_fs_ns from "node:fs.farm-runtime";
var cjsExport_ts_cjs = __commonJs({
    "cjsExport.ts.farm-runtime": (module, exports)=>{
        module.exports.name = 'shulan';
        module.exports.age = 18;
        module.exports.default = 'default';
    }
});

// module_id: esmExport.ts.farm-runtime
const esmName = 'esmName';
const esmAge = 18;
var esmExport_ts_ns = {
    esmAge: esmAge,
    esmName: esmName,
    __esModule: true
};

// module_id: esmExport2.ts.farm-runtime
const esmName2 = "esmName";
const esmAge2 = 18;
function foo() {}
var esmExport2_ts_ns = {
    esmAge2: esmAge2,
    esmName2: esmName2,
    "default": foo,
    __esModule: true
};

// module_id: cjsExportEsm.ts.farm-runtime
var cjsExportEsm_ts_cjs = __commonJs({
    "cjsExportEsm.ts.farm-runtime": (module, exports)=>{
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
            esmName2: function() {
                return _esmExport2.esmName2;
            },
            foo: function() {
                return _esmExport2.default;
            }
        });
        _export_star(esmExport_ts_ns, exports);
        var _esmExport2 = _interop_require_wildcard(esmExport2_ts_ns);
        module.exports.cjs_export_esm = 'shulan';
    }
});

// module_id: export.ts.farm-runtime
var export_ts_ns = _mergeNamespaces({
    __esModule: true
}, [
    cjsExportEsm_ts_cjs(),
    cjsExport_ts_cjs(),
    node_fs_ns
]);

// module_id: runtime.ts.farm-runtime
console.log(export_ts_ns);
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("index.ts");