({"index.module.css":function  (module, exports, farmRequire, farmDynamicRequire) {
    farmRequire("index.module.css?farm_css_modules");
    exports.default = {};
    module._m(exports);
}
,
"index.module.css?farm_css_modules":function  (module, exports, farmRequire, farmDynamicRequire) {
    const cssCode = `body {
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
"index.module.css?raw":function  (module, exports, farmRequire, farmDynamicRequire) {
    exports.default = "body {\n  color: red;\n}";
    module._m(exports);
}
,})
{}