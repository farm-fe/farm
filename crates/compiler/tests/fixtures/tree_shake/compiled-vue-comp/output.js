//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());window['__farm_default_namespace__'].m.se({
    "vue": window['vue'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "8cea7e1d": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = (a)=>a;
    },
    "ae8e2392": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
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
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_comp = farmRequire.i(farmRequire("d1fd5279"));
        var _f_comp1 = farmRequire("d1fd5279");
        farmRequire._e(exports, _f_comp1);
        console.log(farmRequire.f(_f_comp));
    },
    "d1fd5279": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_comp_ts = farmRequire.i(farmRequire("ae8e2392"));
        var _f_helper = farmRequire.i(farmRequire("8cea7e1d"));
        var _f_comp_ts1 = farmRequire("ae8e2392");
        farmRequire._e(exports, _f_comp_ts1);
        exports.default = farmRequire.f(_f_helper)(farmRequire.f(_f_comp_ts));
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");