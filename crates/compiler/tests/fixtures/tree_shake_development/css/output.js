//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-system-helper.ts');
}
function initModuleSystem$1() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
initModuleSystem$1(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "index.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "c", function() {
            return c;
        });
        farmRequire("style/a.css");
        var _f_logo1 = farmRequire.i(farmRequire("style/logo1.png"));
        var c = farmRequire.f(_f_logo1);
    },
    "style/a.css": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire("style/b.css");
        const cssCode = `
`;
        const farmId = 'style/a.css';
        const previousStyle = document.querySelector(`style[data-farm-id="${farmId}"]`);
        const style = document.createElement('style');
        style.setAttribute('data-farm-id', farmId);
        style.innerHTML = cssCode;
        if (previousStyle) {
            previousStyle.replaceWith(style);
        } else {
            document.head.appendChild(style);
        }
        if (module.meta.hot) {
            module.meta.hot.accept();
            module.meta.hot.prune(()=>{
                style.remove();
            });
        }
    },
    "style/b.css": function(module, exports, farmRequire, farmDynamicRequire) {
        const cssCode = `* {
  margin: 0;
  padding: 0;
  background: url("/logo-73d4a8.png");
}
`;
        const farmId = 'style/b.css';
        const previousStyle = document.querySelector(`style[data-farm-id="${farmId}"]`);
        const style = document.createElement('style');
        style.setAttribute('data-farm-id', farmId);
        style.innerHTML = cssCode;
        if (previousStyle) {
            previousStyle.replaceWith(style);
        } else {
            document.head.appendChild(style);
        }
        if (module.meta.hot) {
            module.meta.hot.accept();
            module.meta.hot.prune(()=>{
                style.remove();
            });
        }
    },
    "style/logo.png": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = "/logo-73d4a8.png";
    },
    "style/logo1.png": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = "/logo1-cbaed8.png";
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("index.ts");var __farm_entry_c__=__farm_entry__.c;export {__farm_entry_c__ as c};

//logo-73d4a8.png:
 

//logo1-cbaed8.png:
 