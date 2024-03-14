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
var _default = {};
},
"index.module.css?farm_css_modules":function(m,e,r,dr){"use strict";
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
if (m.meta.hot) {
    m.meta.hot.accept();
    m.meta.hot.prune(()=>{
        style.remove();
    });
}
},
"index.module.css?raw":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _default = "body {\n  color: red;\n}";
},})
{}