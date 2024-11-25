//__farm_runtime.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};;((function(){function __commonJs(mod) {
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
var index_js_cjs = __commonJs((module, exports)=>{
    "use strict";
    console.log('runtime/index.js');
    window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
index_js_cjs();
})());

//index.js:
 import "./__farm_runtime.js";import "./index_7ecc.js";(function(_){for(var r in _){_[r].__farm_resource_pot__='index_7d8a.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"363fc137":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log('utils.js');
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    const fs = farmRequire("e4b1dea3", true);
    const utils = farmRequire("363fc137", true);
    console.log(fs, utils);
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources(['index_7ecc.js']);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry;

//index_7ecc.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='index_7ecc.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"e4b1dea3":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log('fs-extra');
}
,});