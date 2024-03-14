({"index.css":function(m,e,r,dr){"use strict";
const cssCode = `body {
  color: red;
}
`;
const farmId = "index.css";
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
},})
{}