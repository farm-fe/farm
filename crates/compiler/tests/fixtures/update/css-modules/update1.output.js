({"index.module.css":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
r("index.module.css?farm_css_modules");
var _default = {
    "className": `className-ec324e46`
};
},
"index.module.css?farm_css_modules":function(m,e,r,dr){"use strict";
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
if (m.meta.hot) {
    m.meta.hot.accept();
    m.meta.hot.prune(()=>{
        style.remove();
    });
}
},
"index.ts":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
var _interop_require_default = r("@swc/helpers/_/_interop_require_default");
r("index.css");
var _indexmodulecss = _interop_require_default._(r("index.module.css"));
console.log(_indexmodulecss.default);
},})
{}