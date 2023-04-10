//dbadda5a.js:
 import module from "node:module";
global.__farmNodeRequire = module.createRequire(import.meta.url);
global.__farmNodeBuiltinModules = module.builtinModules;
(function(modules, entryModule) {
    var cache = {};
    function require(id) {
        if (cache[id]) return cache[id].exports;
        var module = {
            id: id,
            exports: {}
        };
        modules[id](module, module.exports, require);
        cache[id] = module;
        return module.exports;
    }
    require(entryModule);
})({
    "../../_internal/runtime/index.js.farm-runtime": function(module, exports, require, dynamicRequire) {
        "use strict";
        console.log("runtime/index.js");
        __farm_global_this__.__farm_module_system__.setPlugins([]);
    }
}, "../../_internal/runtime/index.js.farm-runtime");
(function(modules) {
    for(var key in modules){
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "comp.tsx": function(module, exports, require, dynamicRequire) {
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
        var _interopRequireWildcard = require("@swc/helpers/lib/_interop_require_wildcard.js").default;
        var _dep = _interopRequireWildcard(require("dep.ts"));
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
                fallback: _dep.default.createElement("div", null, "Loading..."),
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
        _c = Description;
        var _c, _c;
        $RefreshReg$(_c, "LazyComp");
        $RefreshReg$(_c, "Description");
    },
    "dep.ts": function(module, exports, require, dynamicRequire) {
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
    },
    "entry.ts": function(module, exports, require, dynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        var _comp = require("comp.tsx");
        console.log((0, _comp.Description)());
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("entry.ts").default;
export default entry;
