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
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_ddf1.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"dep.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "a", function() {
        return a;
    });
    module.o(exports, "invalidate", function() {
        return invalidate;
    });
    if (module.meta.hot) {
        module.meta.hot.accept(()=>{
            module.meta.hot.invalidate('parent module should accept this');
        });
    }
    var a = '1';
    function invalidate() {
        return `invalidate data`;
    }
}
,
"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "InvalidateParent", function() {
        return InvalidateParent;
    });
    var _f_dep = farmRequire("dep.ts");
    console.log(_f_dep.a);
    const id = 'InvalidateParent';
    function InvalidateParent() {
        return {
            render: ()=>{
                const renderData = _f_dep.invalidate();
                const div = document.createElement('div', {});
                div.id = id;
                div.innerText = renderData;
                div.className = 'box';
                return div;
            }
        };
    }
    if (module.meta.hot) {
        module.meta.hot.accept();
        const div = document.getElementById(id);
        if (div) {
            const comp = InvalidateParent().render();
            console.log(div, comp);
            div.replaceWith(comp);
        }
    }
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("index.ts");var InvalidateParent=entry.InvalidateParent;export { InvalidateParent };