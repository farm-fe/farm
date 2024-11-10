//index.js:
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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_5de5.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"6f462555":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    "";
    exports.default = 'comp';
}
,
"b3d9bc98":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log('resolved.ts');
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    farmRequire("b3d9bc98");
    var _f_comp = module.i(farmRequire("6f462555"));
    console.log(module.f(_f_comp));
    exports.default = 2;
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default || entry;

//index_337c.css:
 .body {
  color: red;
}