//index.js:
 globalThis.nodeRequire = require;(globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};// module_id: ../../../node_modules/@swc/helpers/_/_export_star/index.js.farm-runtime
var node_fs_ns = _interop_require_wildcard(require("node:fs.farm-runtime"));
function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        mod[Object.keys(mod)[0]](module, module.exports);
        return module.exports;
    };
}
function _mergeNamespaces(n, m) {
    m.forEach(function(e) {
        e && typeof e !== "string" && !Array.isArray(e) && Object.keys(e).forEach(function(k) {
            if (k !== "default" && !(k in n)) {
                var d = Object.getOwnPropertyDescriptor(e, k);
                Object.defineProperty(n, k, d.get ? d : {
                    enumerable: true,
                    get: function() {
                        return e[k];
                    }
                });
            }
        });
    });
    return Object.freeze(n);
}
function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}
function _interop_require_wildcard(obj, nodeInterop) {
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
}
function _export_star(from, to) {
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
}
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
function _export_star$1(from, to) {
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
}

// module_id: cjsExport.ts.farm-runtime
var cjsExport_ts_cjs = __commonJs({
    "cjsExport.ts.farm-runtime": (module, exports)=>{
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "esmName", {
            enumerable: true,
            get: function() {
                return esmName;
            }
        });
        module.exports.name = "shulan";
        module.exports.age = 18;
        const esmName = "esm-shulan";
    }
});
var cjsExport_ts_ns = cjsExport_ts_cjs();

// module_id: esmExport.ts.farm-runtime
const foo = "foo";
var esmExport_ts_ns = {
    "foo": foo,
    __esModule: true
};

// module_id: runtime.ts.farm-runtime
const esmName = "repeat-esm-shulan";
(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='file://'+__filename;(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("index.ts");