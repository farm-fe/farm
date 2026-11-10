//index.js:
 (function(){if(typeof window==='undefined'||!window['__farm_default_namespace__']||!window['__farm_default_namespace__'].m){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_d7d494f3f58482765af33c12046b0df8_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "3e3af5b6": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "x", function() {
            return x;
        });
        farmRequire.o(exports, "Exported", function() {
            return Exported;
        });
        var x = 1;
        class StaticBlock {
            static{
                console.log('static block');
            }
        }
        class StaticField {
            static field = console.log('static field');
        }
        class ComputedMethod {
            [console.log('computed method')]() {}
        }
        class ComputedField {
            [console.log('computed field')] = console.log('deferred instance field');
        }
        const Expression = class {
            static{
                console.log('class expression');
            }
        };
        class Exported {
            static field = console.log('exported class');
        }
        exports.default = class {
            static #field = console.log('default class');
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("3e3af5b6");
        console.log('done', _f_dep.x);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;