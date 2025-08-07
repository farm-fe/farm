window['__farm_default_namespace__'].m._rg=true;(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "index.css": function(module, exports, farmRequire, farmDynamicRequire) {
        const cssCode = `body {
  color: red;
}
`;
        const farmId = 'index.css';
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
    }
});
window['__farm_default_namespace__'].m._rg=false;
