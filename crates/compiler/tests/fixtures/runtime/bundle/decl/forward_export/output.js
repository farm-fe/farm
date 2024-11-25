//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function __commonJs(mod) {
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
function Layout() {}
function wrap(fn) {
    return ()=>{
        return fn();
    };
}
var ForwardLayout = wrap(Layout);
var LayoutComponent = ForwardLayout;
LayoutComponent.Sider = ()=>{};
LayoutComponent.Row = ()=>{};


var cjs_ts_cjs = __commonJs((module, exports)=>{
    "use strict";
    const Sider = LayoutComponent.Sider;
    const Row = LayoutComponent.Row;
    console.log({
        Sider,
        Row,
        Layout: LayoutComponent
    });
    module.exports.Layout = LayoutComponent;
});

cjs_ts_cjs();
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");