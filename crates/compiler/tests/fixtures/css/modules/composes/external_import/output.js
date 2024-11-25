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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_0467.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"8b6840d6":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    "";
    exports.default = {
        "action": `farm-action`
    };
}
,
"95fe6ac5":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    "";
    var _f_action = module.i(farmRequire("8b6840d6"));
    exports.default = {
        "base": `farm-base ${module.f(_f_action)["action"]}`
    };
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_index = module.i(farmRequire("95fe6ac5"));
    console.log(module.f(_f_index).base);
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");

//index_c9a6.css:
 .farm-base {
  font-size: 18px;
}
.farm-action {
  color: red;
}