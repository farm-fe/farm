window['__farm_default_namespace__'].m._rg=true;(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "index.module.css": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire("index.module.css?farm_css_modules");
        exports.default = {
            "className": `className-ec324e46`
        };
    },
    "index.module.css?farm_css_modules": function(module, exports, farmRequire, farmDynamicRequire) {
        const cssCode = `.className-ec324e46 {
  color: red;
}
`;
        const farmId = 'index.module.css?farm_css_modules';
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
    "index.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire("index.css");
        var _f_index_module = farmRequire.i(farmRequire("index.module.css"));
        console.log(farmRequire.f(_f_index_module));
    }
});
window['__farm_default_namespace__'].m._rg=false;
