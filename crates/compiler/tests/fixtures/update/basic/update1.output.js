({"index.css":function  (module, exports, farmRequire, farmDynamicRequire) {
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
,})
{}