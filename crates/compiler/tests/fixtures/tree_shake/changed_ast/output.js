//index.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};;((function(){// module_id: ../../_internal/runtime/index.js.farm-runtime
function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        if (typeof mod === "function") {
            mod(module, module.exports);
        } else {
            mod[Object.keys(mod)[0]](module, module.exports);
        }
        return module.exports;
    };
}
var index_js_cjs = __commonJs({
    "../../_internal/runtime/index.js.farm-runtime": (module, exports)=>{
        "use strict";
        console.log('runtime/index.js');
        window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
    }
});
index_js_cjs();
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_4924.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"comp.tsx":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "Description", function() {
        return Description;
    });
    var _f_dep = module.w(farmRequire("dep.ts"));
    const LazyComp = module.f(_f_dep).lazy(()=>Promise.resolve({
            default: ()=>module.f(_f_dep).createElement("div", {
                    __source: {
                        fileName: "comp.tsx",
                        lineNumber: 3,
                        columnNumber: 73
                    },
                    __self: this
                }, "Lazy")
        }));
    _c = LazyComp;
    function Description() {
        console.trace('In Description, the sourcemap should be correct');
        return module.f(_f_dep).createElement(_f_dep.Suspense, {
            fallback: module.f(_f_dep).createElement("div", {
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
        }, module.f(_f_dep).createElement(LazyComp, {
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
    module._m(exports);
    module.o(exports, "Suspense", function() {
        return Suspense;
    });
    var Suspense = function() {
        console.log('Suspense');
    };
    exports.default = {
        createElement (comp, ...args) {
            console.log(comp(), args);
        },
        lazy: (promise)=>{
            console.log('lazy', promise);
        }
    };
}
,
"entry.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_comp = farmRequire("comp.tsx");
    console.log(_f_comp.Description());
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("entry.ts");