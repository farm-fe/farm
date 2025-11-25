//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('dynamic-import.ts');
}
function initModuleSystem$1() {
    console.log('module-system-helper.ts');
}
function initModuleSystem$2() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
initModuleSystem$1(__farm_internal_module_system__);
initModuleSystem$2(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_ddf1ac1dd74c1a8307895afa712b1c49_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "dep.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return a;
        });
        farmRequire.o(exports, "invalidate", function() {
            return invalidate;
        });
        if (module.meta.hot) {
            module.meta.hot.accept(()=>{
                module.meta.hot.invalidate('parent module should accept this');
            });
        }
        var a = '1';
        function invalidate() {
            return `invalidate data`;
        }
    },
    "index.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "InvalidateParent", function() {
            return InvalidateParent;
        });
        var _f_dep = farmRequire("dep.ts");
        console.log(_f_dep.a);
        const id = 'InvalidateParent';
        function InvalidateParent() {
            return {
                render: ()=>{
                    const renderData = _f_dep.invalidate();
                    const div = document.createElement('div', {});
                    div.id = id;
                    div.innerText = renderData;
                    div.className = 'box';
                    return div;
                }
            };
        }
        if (module.meta.hot) {
            module.meta.hot.accept();
            const div = document.getElementById(id);
            if (div) {
                const comp = InvalidateParent().render();
                console.log(div, comp);
                div.replaceWith(comp);
            }
        }
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("index.ts");var __farm_entry_InvalidateParent__=__farm_entry__.InvalidateParent;export {__farm_entry_InvalidateParent__ as InvalidateParent};