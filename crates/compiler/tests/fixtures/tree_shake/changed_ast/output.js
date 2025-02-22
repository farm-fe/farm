//index.js:
 (function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-system-helper.ts');
}
function initModuleSystem$1() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
initModuleSystem$1(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "comp.tsx": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Description", function() {
            return Description;
        });
        var _f_dep = farmRequire.w(farmRequire("dep.ts"));
        const LazyComp = farmRequire.f(_f_dep).lazy(()=>Promise.resolve({
                default: ()=>farmRequire.f(_f_dep).createElement("div", {
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
            return farmRequire.f(_f_dep).createElement(_f_dep.Suspense, {
                fallback: farmRequire.f(_f_dep).createElement("div", {
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
            }, farmRequire.f(_f_dep).createElement(LazyComp, {
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
    },
    "dep.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Suspense", function() {
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
    },
    "entry.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_comp = farmRequire("comp.tsx");
        console.log(_f_comp.Description());
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("entry.ts");