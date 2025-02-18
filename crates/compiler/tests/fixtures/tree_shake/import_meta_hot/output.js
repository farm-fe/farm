//index.js:
 (function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return a;
        });
        farmRequire.o(exports, "invalidate", function() {
            return invalidate;
        });
        ;
        var a = '1';
        function invalidate() {
            return `invalidate data`;
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "InvalidateParent", function() {
            return InvalidateParent;
        });
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.a);
        const id = "InvalidateParent";
        function InvalidateParent() {
            return {
                render: ()=>{
                    const renderData = _f_dep.invalidate();
                    const div = document.createElement("div", {});
                    div.id = id;
                    div.innerText = renderData;
                    div.className = "box";
                    return div;
                }
            };
        }
        ;
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_InvalidateParent__=__farm_entry__.InvalidateParent;export {__farm_entry_InvalidateParent__ as InvalidateParent};