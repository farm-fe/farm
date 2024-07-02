({"index.module.css":function  (module, exports, farmRequire, farmDynamicRequire) {
    farmRequire("index.module.css?farm_css_modules");
    exports.default = {
        "className": `className-ec324e46`
    };
    module._m(exports);
}
,
"index.module.css?farm_css_modules":function  (module, exports, farmRequire, farmDynamicRequire) {
    const cssCode = `.className-ec324e46 {
  color: red;
}
`;
    const farmId = "index.module.css?farm_css_modules";
    const previousStyle = document.querySelector(`style[data-farm-id="${farmId}"]`);
    const style = document.createElement("style");
    style.setAttribute("data-farm-id", farmId);
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
,
"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    farmRequire("index.css");
    var _f_index_module = module.i(farmRequire("index.module.css"));
    console.log(module.f(_f_index_module));
    module._m(exports);
}
,})
{}