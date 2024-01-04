({"index.module.css": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return _default;
        }
    });
    farmRequire("index.module.css.FARM_CSS_MODULES?f1d5b6cc");
    var _default = {
        "className": `className-47c35c9b`
    };
},
"index.module.css.FARM_CSS_MODULES?f1d5b6cc": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    const cssCode = `.className-47c35c9b {
  color: red;
}
`;
    const farmId = "index.module.css.FARM_CSS_MODULES?f1d5b6cc";
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
},
"index.ts": function(module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    farmRequire("index.css");
    var _indexmodulecss = _interop_require_default._(farmRequire("index.module.css"));
    console.log(_indexmodulecss.default);
},})
{}