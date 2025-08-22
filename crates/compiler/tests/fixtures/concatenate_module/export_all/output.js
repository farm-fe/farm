//farm_internal_runtime_index.js:
 const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);


//index.js:
 import "./farm_internal_runtime_index.js";import "./vue-core.js";import "./vue-pack.js";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_dcdc3e0b3362edb8fec2a51d3fa51f8f_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_vue = farmRequire("f4b74577");
        console.log(_f_vue.render(), _f_vue.createElementVNode);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");

//vue-core.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "vue-core__js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "02916e72": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "createElementVNode", function() {
            return createBaseVNode;
        });
        farmRequire.o(exports, "h", function() {
            return h;
        });
        function h(tag, props, children) {
            return {
                tag,
                props,
                children
            };
        }
        function createBaseVNode() {
            return 'base vnode';
        }
    }
});


//vue-pack.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "vue-pack__js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "f4b74577": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "createElementVNode", function() {
            return _f_vue_core.createElementVNode;
        });
        farmRequire.o(exports, "h", function() {
            return _f_vue_core.h;
        });
        farmRequire.o(exports, "render", function() {
            return render;
        });
        var _f_vue_core = farmRequire("02916e72");
        function render() {
            return _f_vue_core.h('div', {}, 'hello world');
        }
        function initDev() {
            console.log('init dev');
        }
        initDev();
    }
});
