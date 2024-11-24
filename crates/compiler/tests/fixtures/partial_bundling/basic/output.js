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

//debounce_6f74.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='debounce_6f74.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"01609b59":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "default", function() {
        return debounce;
    });
    var _f_utils = farmRequire("a5831d05");
    function debounce(fn) {
        _f_utils.debug("debounce");
        return fn;
    }
}
,});

//index.js:
 import "./__farm_runtime.js";import "./index_2faa.js";import "./index_64d2.js";window['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"module": (window['module']||{}).default && !(window['module']||{}).__esModule ? {...(window['module']||{}),__esModule:true} : window['module']||{}});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_module = module.i(farmRequire('module'));
    var _f_merge = module.i(farmRequire("726cd210"));
    function defineConfig(config) {
        module.f(_f_merge)(config, {
            compilation: {
                input: {
                    main: './main.tsx'
                },
                external: module.f(_f_module).builtinModules
            }
        });
        return config;
    }
    farmDynamicRequire("01609b59").then((debounce)=>{
        console.log(debounce);
    });
    exports.default = defineConfig({});
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources(['index_2faa.js','index_64d2.js']);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([{ path: 'index_2faa.js', type: 0 },{ path: 'debounce_6f74.js', type: 0 }],{ '01609b59': [0,1] });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default || entry;

//index_2faa.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='index_2faa.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"a5831d05":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "debug", function() {
        return debug;
    });
    function debug(msg) {
        console.log(msg);
    }
}
,});

//index_64d2.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='index_64d2.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"726cd210":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "default", function() {
        return merge;
    });
    var _f_utils = farmRequire("a5831d05");
    function merge(a, b) {
        _f_utils.debug("merge");
        return a + b;
    }
}
,});