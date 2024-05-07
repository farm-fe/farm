//__farm_runtime.bfbb01c9.mjs:
 import __farmNodeModule from 'node:module';globalThis.nodeRequire = __farmNodeModule.createRequire(import.meta.url);(globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function _export_star(from, to) {
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

const a = 3;
const b = 4;
const c = 5;
function BB() {
    const a = 5;
    const b = 6;
    console.log(a, b);
}
var dep_ts = {
    a: a,
    b: b,
    c: c
};
var dep_ts_ns = {
    "a": a,
    "b": b
};




const bundle2A = "bundle2A";
const bundle2B = "bundle2B";



(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);


//bundle2.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='bundle2.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"9488de80":function  (module, exports, farmRequire, farmDynamicRequire) {
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
        bundle2A: function() {
            return bundle2A;
        },
        bundle2B: function() {
            return bundle2B;
        }
    });
    const bundle2A = "bundle2A";
    const bundle2B = "bundle2B";
}
,
"d1a94858":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _export_star = farmRequire("@swc/helpers/_/_export_star");
    _export_star._(farmRequire("9488de80"), exports);
}
,});

//index.js:
 import "./__farm_runtime.bfbb01c9.mjs";import "./bundle2.js";(function(_){for(var r in _){_[r].__farm_resource_pot__='index_e001.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
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
        a: function() {
            return a;
        },
        b: function() {
            return b;
        },
        default: function() {
            return _default;
        }
    });
    const a = 3;
    const b = 4;
    const c = 5;
    function BB() {
        const a = 5;
        const b = 6;
        console.log(a, b);
    }
    var _default = {
        a,
        b,
        c
    };
}
,
"1e5f1cae":function  (module, exports, farmRequire, farmDynamicRequire) {
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
        ImportNamespace: function() {
            return _dep;
        },
        default: function() {
            return _default;
        }
    });
    var _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
    var _dep = _interop_require_wildcard._(farmRequire("05ee5ec7"));
    var _default = _dep;
}
,
"25593d80":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _export_star = farmRequire("@swc/helpers/_/_export_star");
    _export_star._(farmRequire("05ee5ec7"), exports);
}
,
"8c9fcf3b":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _export_star = farmRequire("@swc/helpers/_/_export_star");
    _export_star._(farmRequire("9488de80"), exports);
}
,
"b31fbbb1":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "ExportNamespace", {
        enumerable: true,
        get: function() {
            return _dep;
        }
    });
    var _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
    var _dep = _interop_require_wildcard._(farmRequire("05ee5ec7"));
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
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
        ExportNamespace: function() {
            return _exportNamespace.ExportNamespace;
        },
        ImportNamespace: function() {
            return _importNamespace.ImportNamespace;
        },
        bundle2A: function() {
            return _bundle2index.bundle2A;
        },
        bundle2B: function() {
            return _bundle2index.bundle2B;
        }
    });
    var _export_star = farmRequire("@swc/helpers/_/_export_star");
    var _importNamespace = farmRequire("1e5f1cae");
    var _exportNamespace = farmRequire("b31fbbb1");
    _export_star._(farmRequire("25593d80"), exports);
    _export_star._(farmRequire("8c9fcf3b"), exports);
    var _bundle2index = farmRequire("d1a94858");
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources(['bundle2.js']);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");var ImportNamespace=entry.ImportNamespace;export { ImportNamespace };var ExportNamespace=entry.ExportNamespace;export { ExportNamespace };var a=entry.a;export { a };var b=entry.b;export { b };var bundle2A=entry.bundle2A;export { bundle2A };var bundle2B=entry.bundle2B;export { bundle2B };var bundle2A=entry.bundle2A;export { bundle2A };var bundle2B=entry.bundle2B;export { bundle2B };