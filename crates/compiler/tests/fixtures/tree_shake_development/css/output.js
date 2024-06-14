//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"../../_internal/runtime/index.js.farm-runtime":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"../../_internal/runtime/index.js.farm-runtime");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_4246.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "c", {
        enumerable: true,
        get: function() {
            return c;
        }
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    farmRequire("style/a.css");
    var _logo1png = _interop_require_default._(farmRequire("style/logo1.png"));
    const c = _logo1png.default;
}
,
"style/a.css":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    farmRequire("style/b.css");
    const cssCode = `
`;
    const farmId = "style/a.css";
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
"style/b.css":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    const cssCode = `* {
  margin: 0;
  padding: 0;
  background: url("/logo-73d4a8.png");
}
`;
    const farmId = "style/b.css";
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
"style/logo.png":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    var _default = "/logo-73d4a8.png";
}
,
"style/logo1.png":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    var _default = "/logo1-cbaed8.png";
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("index.ts");var c=entry.c;export { c };

//logo-73d4a8.png:
 

//logo1-cbaed8.png:
 