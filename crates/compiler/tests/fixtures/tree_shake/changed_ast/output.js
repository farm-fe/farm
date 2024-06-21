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
}((function(){// module_id: ../../_internal/runtime/index.js.farm-runtime
var index_js_cjs = __commonJs({
    "../../_internal/runtime/index.js.farm-runtime": (module, exports)=>{
        "use strict";
        console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
    }
});
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_4924.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"comp.tsx":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "Description", {
        enumerable: true,
        get: function() {
            return Description;
        }
    });
    var _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
    var _dep = _interop_require_wildcard._(farmRequire("dep.ts"));
    const LazyComp = _dep.default.lazy(()=>Promise.resolve({
            default: ()=>_dep.default.createElement("div", {
                    __source: {
                        fileName: "comp.tsx",
                        lineNumber: 3,
                        columnNumber: 73
                    },
                    __self: void 0
                }, "Lazy")
        }));
    _c = LazyComp;
    function Description() {
        console.trace("In Description, the sourcemap should be correct");
        return _dep.default.createElement(_dep.Suspense, {
            fallback: _dep.default.createElement("div", {
                __source: {
                    fileName: "comp.tsx",
                    lineNumber: 8,
                    columnNumber: 30
                }
            }, "Loading..."),
            __source: {
                fileName: "comp.tsx",
                lineNumber: 8,
                columnNumber: 10
            },
            __self: this
        }, _dep.default.createElement(LazyComp, {
            __source: {
                fileName: "comp.tsx",
                lineNumber: 8,
                columnNumber: 53
            },
            __self: this
        }));
    }
    _c1 = Description;
    var _c, _c1;
    $RefreshReg$(_c, "LazyComp");
    $RefreshReg$(_c1, "Description");
}
,
"dep.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
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
        Suspense: function() {
            return Suspense;
        },
        default: function() {
            return _default;
        }
    });
    const Suspense = function() {
        console.log("Suspense");
    };
    var _default = {
        createElement (comp, ...args) {
            console.log(comp(), args);
        },
        lazy: (promise)=>{
            console.log("lazy", promise);
        }
    };
}
,
"entry.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _comp = farmRequire("comp.tsx");
    console.log((0, _comp.Description)());
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("entry.ts");