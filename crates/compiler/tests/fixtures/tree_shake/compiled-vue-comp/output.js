//index.js:
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
})());window['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"vue": (window['vue']||{}).default && !(window['vue']||{}).__esModule ? {...(window['vue']||{}),__esModule:true} : window['vue']||{}});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_236f.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"8cea7e1d":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = (a)=>a;
}
,
"ae8e2392":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_vue = farmRequire('vue');
    var _f_vue1 = farmRequire("vue");
    const _imports_0 = "URL_ADDRESS";
    const _hoisted_1 = {
        class: "container"
    };
    const _hoisted_2 = _f_vue1.createStaticVNode("<a href=\"https://farmfe.org/\" target=\"_blank\" data-v-370e85d7><div class=\"logo1\" data-v-370e85d7></div><div class=\"logo2\" data-v-370e85d7></div></a><a href=\"https://farmfe.org/\" target=\"_blank\" data-v-370e85d7><img src=\"" + _imports_0 + "\" class=\"logo\" alt=\"Farm logo\" data-v-370e85d7></a>", 2);
    const _hoisted_4 = {
        href: "https://vuejs.org/",
        target: "_blank"
    };
    const HelloWorld = _f_vue.defineComponent({});
    const Formatter = _f_vue.defineComponent({});
    exports.default = _f_vue.defineComponent({
        __name: 'index',
        setup (__props) {
            return (_ctx, _cache)=>{
                const _component_el_button = _f_vue1.resolveComponent("el-button");
                const _component_my_svg_icon = _f_vue1.resolveComponent("my-svg-icon");
                const _component_el_config_provider = _f_vue1.resolveComponent("el-config-provider");
                return _f_vue1.openBlock(), _f_vue1.createElementBlock(_f_vue1.Fragment, null, [
                    _f_vue1.createVNode(_component_el_button, {
                        type: "primary",
                        onClick: _cache[0] || (_cache[0] = ($event)=>_ctx.$router.push('/about'))
                    }, {
                        default: _f_vue1.withCtx(()=>[
                                _f_vue1.createTextVNode("to about page")
                            ]),
                        _: 1
                    }),
                    _f_vue1.createElementVNode("div", _hoisted_1, [
                        _hoisted_2,
                        _f_vue1.createElementVNode("a", _hoisted_4, [
                            _f_vue1.createVNode(_component_my_svg_icon, {
                                name: "icon-vue",
                                class: "logo",
                                style: {
                                    "height": "6.25rem",
                                    "width": "6.25rem"
                                }
                            })
                        ])
                    ]),
                    _f_vue1.createVNode(_component_el_config_provider, {
                        size: 'large',
                        "z-index": 3000
                    }, {
                        default: _f_vue1.withCtx(()=>[
                                _f_vue1.createVNode(HelloWorld, {
                                    msg: "Farm + Vue"
                                }),
                                _f_vue1.createVNode(Formatter)
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
    module._m(exports);
    var _f_comp = module.i(farmRequire("d1fd5279"));
    var _f_comp1 = farmRequire("d1fd5279");
    module._e(exports, _f_comp1);
    console.log(module.f(_f_comp));
}
,
"d1fd5279":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_comp_ts = module.i(farmRequire("ae8e2392"));
    var _f_helper = module.i(farmRequire("8cea7e1d"));
    var _f_comp_ts1 = farmRequire("ae8e2392");
    module._e(exports, _f_comp_ts1);
    exports.default = module.f(_f_helper)(module.f(_f_comp_ts));
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");