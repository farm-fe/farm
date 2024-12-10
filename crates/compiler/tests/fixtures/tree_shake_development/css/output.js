//index.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};;((function(){// module_id: ../../_internal/runtime/index.js.farm-runtime
function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        if (typeof mod === "function") {
            mod(module, module.exports);
        } else {
            mod[Object.keys(mod)[0]](module, module.exports);
        }
        return module.exports;
    };
}
var index_js_cjs = __commonJs({
    "../../_internal/runtime/index.js.farm-runtime": (module, exports)=>{
        "use strict";
        console.log('runtime/index.js');
        window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
    }
});
index_js_cjs();
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_4246.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "c", function() {
        return c;
    });
    farmRequire("style/a.css");
    var _f_logo1 = module.i(farmRequire("style/logo1.png"));
    var c = module.f(_f_logo1);
}
,
"style/a.css":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    farmRequire("style/b.css");
    const cssCode = `
`;
    const farmId = 'style/a.css';
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
,
"style/b.css":function  (module, exports, farmRequire, farmDynamicRequire) {
    const cssCode = `* {
  margin: 0;
  padding: 0;
  background: url("/logo-73d4a8.png");
}
`;
    const farmId = 'style/b.css';
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
,
"style/logo.png":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = "/logo-73d4a8.png";
}
,
"style/logo1.png":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = "/logo1-cbaed8.png";
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("index.ts");var c=entry.c;export { c };

//logo-73d4a8.png:
 

//logo1-cbaed8.png:
 