//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"../../_internal/runtime/index.js.farm-runtime":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"../../_internal/runtime/index.js.farm-runtime");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_4924.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"comp.tsx":function  (module, exports, farmRequire, farmDynamicRequire) {
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