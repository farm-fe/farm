//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_de21bc06289dd4f5d35b71c727a5725c_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "routes", function() {
            return routes;
        });
        var _f_dep = farmRequire.w(farmRequire("edfa0cee"));
        var __glob__0_0 = _f_dep;
        var routes = {
            "./modules/dep.ts": __glob__0_0
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.routes);
    },
    "edfa0cee": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
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
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");