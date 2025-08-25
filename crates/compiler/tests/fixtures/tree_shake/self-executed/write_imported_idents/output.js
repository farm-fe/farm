//farm_internal_runtime_index.js:
 const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);


//index-f595b3de.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_f595b3de245434922124458e2dd6e758_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "5650438d": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_g_lite = farmRequire("7927f9cf");
        class AnimationTimeline {
        }
        const runtime = _f_g_lite.runtime;
        runtime.AnimationTimeline = AnimationTimeline;
        _f_g_lite.runtime_should_be_preserved.AnimationTimeline = AnimationTimeline;
    },
    "7700579b": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_g_lite = farmRequire("7927f9cf");
        farmRequire._e(exports, _f_g_lite);
        var _f_g_web_animations_api = farmRequire("5650438d");
        farmRequire._e(exports, _f_g_web_animations_api);
    },
    "7927f9cf": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "runtime", function() {
            return runtime;
        });
        farmRequire.o(exports, "runtime_should_be_preserved", function() {
            return runtime_should_be_preserved;
        });
        farmRequire.o(exports, "Camera", function() {
            return Camera;
        });
        var runtime = {};
        var runtime_should_be_preserved = window;
        function Camera() {}
    }
});


//index.js:
 import "./farm_internal_runtime_index.js";import "./index-f595b3de.js";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_f38548c7660e41b17c5ed47a6f7f7751_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return a;
        });
        farmRequire.o(exports, "b", function() {
            return b;
        });
        var _f_dep3 = farmRequire("1111770e");
        var a = {
            field: '1',
            Camera: undefined
        };
        const nativeLocation = _f_dep3.nativeWindow.location;
        var b = nativeLocation;
    },
    "1111770e": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "nativeWindow", function() {
            return nativeWindow;
        });
        var nativeWindow = document.defaultView || window || self;
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_g = farmRequire("7700579b");
        var _f_dep = farmRequire("05ee5ec7");
        var _f_dep1 = farmRequire.w(farmRequire("05ee5ec7"));
        var ns = _f_dep1;
        var _f_dep2 = farmRequire.w(farmRequire("05ee5ec7"));
        var ns_copy = _f_dep2;
        var _f_dep21 = farmRequire.w(farmRequire("cf66b21c"));
        var ns2 = _f_dep21;
        var AdvancedCamera = function(_Camera) {
            return _Camera;
        }(_f_g.Camera);
        ns.a.Camera = AdvancedCamera;
        const ns_a = ns_copy.a;
        ns_a.field = 'ns_a';
        ns.b.field = 'b';
        ns2[window.document.DOCUMENT_FRAGMENT_NODE].field = '2';
        console.log(_f_g.runtime.AnimationTimeline, _f_dep.a);
    },
    "cf66b21c": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return a;
        });
        farmRequire.o(exports, "b", function() {
            return b;
        });
        farmRequire.o(exports, "Animation", function() {
            return Animation;
        });
        var a = {
            field: '1'
        };
        var b = window;
        const Ani = window.Animation;
        var Animation = Ani;
        Animation.prototype.run = function() {
            console.log('run');
        };
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;