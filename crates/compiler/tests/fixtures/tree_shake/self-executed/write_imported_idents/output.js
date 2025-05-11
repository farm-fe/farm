//__farm_runtime.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}function __commonJs(mod) {
  var module;
  return () => {
    if (module) {
      return module.exports;
    }
    module = {
      exports: {},
    };
    if(typeof mod === "function") {
      mod(module, module.exports);
    }else {
      mod[Object.keys(mod)[0]](module, module.exports);
    }
    return module.exports;
  };
}((function(){var index_js_cjs = __commonJs((module, exports)=>{
    "use strict";
    console.log('runtime/index.js');
    window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
index_js_cjs();
})());

//index.js:
 import "./__farm_runtime.js";import "./index_f595.js";(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_f385.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "a", function() {
        return a;
    });
    module.o(exports, "b", function() {
        return b;
    });
    var _f_dep3 = farmRequire("1111770e");
    var a = {
        field: '1',
        Camera: undefined
    };
    const nativeLocation = _f_dep3.nativeWindow.location;
    var b = nativeLocation;
}
,
"1111770e":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "nativeWindow", function() {
        return nativeWindow;
    });
    var nativeWindow = document.defaultView || window || self;
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_g = farmRequire("7700579b");
    var _f_dep = farmRequire("05ee5ec7");
    var _f_dep1 = module.w(farmRequire("05ee5ec7"));
    var ns = _f_dep1;
    var _f_dep2 = module.w(farmRequire("05ee5ec7"));
    var ns_copy = _f_dep2;
    var _f_dep21 = module.w(farmRequire("cf66b21c"));
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
}
,
"cf66b21c":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "a", function() {
        return a;
    });
    module.o(exports, "b", function() {
        return b;
    });
    module.o(exports, "Animation", function() {
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
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources(['index_f595.js']);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");

//index_f595.js:
 (function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_f595.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"5650438d":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_g_lite = farmRequire("7927f9cf");
    class AnimationTimeline {
    }
    const runtime = _f_g_lite.runtime;
    runtime.AnimationTimeline = AnimationTimeline;
    _f_g_lite.runtime_should_be_preserved.AnimationTimeline = AnimationTimeline;
}
,
"7700579b":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_g_lite = farmRequire("7927f9cf");
    module._e(exports, _f_g_lite);
    var _f_g_web_animations_api = farmRequire("5650438d");
    module._e(exports, _f_g_web_animations_api);
}
,
"7927f9cf":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "runtime", function() {
        return runtime;
    });
    module.o(exports, "runtime_should_be_preserved", function() {
        return runtime_should_be_preserved;
    });
    module.o(exports, "Camera", function() {
        return Camera;
    });
    var runtime = {};
    var runtime_should_be_preserved = window;
    function Camera() {}
}
,});