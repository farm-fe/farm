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
farmRequire("index.module.css?farm_css_modules");
var _default = {
    "className": `className-ec324e46`
};

},
"index.module.css?farm_css_modules": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
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

},
"index.ts": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
farmRequire("index.css");
var _indexmodulecss = /*#__PURE__*/ _interop_require_default._(farmRequire("index.module.css"));
console.log(_indexmodulecss.default);

},})
{}