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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_de21.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "routes", function() {
        return routes;
    });
    var _f_dep = module.w(farmRequire("edfa0cee"));
    var __glob__0_0 = _f_dep;
    var routes = {
        "./modules/dep.ts": __glob__0_0
    };
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_dep = farmRequire("05ee5ec7");
    console.log(_f_dep.routes);
}
,
"edfa0cee":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = {
        path: "/vue-flow",
        redirect: "/vue-flow/index",
        meta: {
            icon: "ep:set-up",
            title: "vue-flow"
        },
        children: [
            {
                path: "/vue-flow/index",
                name: "VueFlow",
                meta: {
                    title: "vue-flow",
                    extraIcon: "IF-pure-iconfont-new svg"
                }
            }
        ]
    };
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");