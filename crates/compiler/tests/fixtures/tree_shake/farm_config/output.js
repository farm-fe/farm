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
})());window['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"module": (window['module']||{}).default && !(window['module']||{}).__esModule ? {...(window['module']||{}),__esModule:true} : ({...window['module']||{}})});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_7f1c.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"052dab48":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = {
        main: './main.tsx'
    };
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_module = farmRequire('module');
    var _f_config = farmRequire("edceee38");
    var _f_util = module.i(farmRequire("052dab48"));
    exports.default = _f_config.defineFarmConfig({
        compilation: {
            input: module.f(_f_util),
            external: _f_module.builtinModules
        }
    });
}
,
"edceee38":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "defineFarmConfig", function() {
        return defineFarmConfig;
    });
    function defineFarmConfig(userConfig) {
        return userConfig;
    }
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default || entry;