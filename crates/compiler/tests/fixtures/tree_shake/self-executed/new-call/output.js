//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_f1d9855084218d7c878eeacf757c9a8c_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "4312d062": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "default", function() {
            return F;
        });
        function $e() {
            return {
                seed: (e)=>{
                    return e;
                }
            };
        }
        var F = class {
            constructor(e = {}){
                let { randomizer: r = $e() } = e;
                this._randomizer = r;
            }
            get defaultRefDate() {
                return this._defaultRefDate;
            }
            setDefaultRefDate(e = ()=>new Date) {
                typeof e == "function" ? this._defaultRefDate = e : this._defaultRefDate = ()=>new Date(e);
            }
            seed(e = Math.ceil(Math.random() * Number.MAX_SAFE_INTEGER)) {
                return this._randomizer.seed(e), e;
            }
        }, Yt = new F;
    },
    "81077a1f": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "useFullscreen", function() {
            return useFullscreen;
        });
        farmRequire.o(exports, "document1", function() {
            return document1;
        });
        const defaultDocument = globalThis.isClient ? window.document : undefined;
        const defaultWindow = globalThis.isClient ? window : undefined;
        const F = {};
        function useFullscreen(target, options = {}) {
            const { document = defaultDocument, autoExit = false } = options;
            return document;
        }
        const { document: { document1 } = defaultWindow.document } = F;
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire.i(farmRequire("4312d062"));
        var _f_object_assign = farmRequire("81077a1f");
        console.log(farmRequire.f(_f_dep), _f_object_assign.useFullscreen, _f_object_assign.document1);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");