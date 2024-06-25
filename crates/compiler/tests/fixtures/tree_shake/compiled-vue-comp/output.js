//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};function _interop_require_default(obj) {
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
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
});
})());(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"vue": ((globalThis||window||{})['vue']||{}).default && !((globalThis||window||{})['vue']||{}).__esModule ? {...((globalThis||window||{})['vue']||{}),__esModule:true} : ((globalThis||window||{})['vue']||{})});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_236f.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"8cea7e1d":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return _default;
        }
    });
    var _default = (a)=>a;
}
,
"ae8e2392":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return _default;
        }
    });
    var _vue = farmRequire("vue");
    const _imports_0 = "URL_ADDRESS";
    const _hoisted_1 = {
        class: "container"
    };
    const _hoisted_2 = (0, _vue.createStaticVNode)('<a href="https://farmfe.org/" target="_blank" data-v-370e85d7><div class="logo1" data-v-370e85d7></div><div class="logo2" data-v-370e85d7></div></a><a href="https://farmfe.org/" target="_blank" data-v-370e85d7><img src="' + _imports_0 + '" class="logo" alt="Farm logo" data-v-370e85d7></a>', 2);
    const _hoisted_4 = {
        href: "https://vuejs.org/",
        target: "_blank"
    };
    const HelloWorld = (0, _vue.defineComponent)({});
    const Formatter = (0, _vue.defineComponent)({});
    var _default = (0, _vue.defineComponent)({
        __name: "index",
        setup (__props) {
            return (_ctx, _cache)=>{
                const _component_el_button = (0, _vue.resolveComponent)("el-button");
                const _component_my_svg_icon = (0, _vue.resolveComponent)("my-svg-icon");
                const _component_el_config_provider = (0, _vue.resolveComponent)("el-config-provider");
                return (0, _vue.openBlock)(), (0, _vue.createElementBlock)(_vue.Fragment, null, [
                    (0, _vue.createVNode)(_component_el_button, {
                        type: "primary",
                        onClick: _cache[0] || (_cache[0] = ($event)=>_ctx.$router.push("/about"))
                    }, {
                        default: (0, _vue.withCtx)(()=>[
                                (0, _vue.createTextVNode)("to about page")
                            ]),
                        _: 1
                    }),
                    (0, _vue.createElementVNode)("div", _hoisted_1, [
                        _hoisted_2,
                        (0, _vue.createElementVNode)("a", _hoisted_4, [
                            (0, _vue.createVNode)(_component_my_svg_icon, {
                                name: "icon-vue",
                                class: "logo",
                                style: {
                                    "height": "6.25rem",
                                    "width": "6.25rem"
                                }
                            })
                        ])
                    ]),
                    (0, _vue.createVNode)(_component_el_config_provider, {
                        size: "large",
                        "z-index": 3000
                    }, {
                        default: (0, _vue.withCtx)(()=>[
                                (0, _vue.createVNode)(HelloWorld, {
                                    msg: "Farm + Vue"
                                }),
                                (0, _vue.createVNode)(Formatter)
                            ]),
                        _: 1
                    })
                ], 64);
            };
        }
    });
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _export_star = farmRequire("@swc/helpers/_/_export_star");
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _comp = _interop_require_default._(_export_star._(farmRequire("d1fd5279"), exports));
    console.log(_comp.default);
}
,
"d1fd5279":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return _default;
        }
    });
    var _export_star = farmRequire("@swc/helpers/_/_export_star");
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _compts = _interop_require_default._(_export_star._(farmRequire("ae8e2392"), exports));
    var _helper = _interop_require_default._(farmRequire("8cea7e1d"));
    var _default = (0, _helper.default)(_compts.default);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");