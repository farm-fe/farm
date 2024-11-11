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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_ddf1.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "AxiosURLSearchParams2", function() {
        return AxiosURLSearchParams2;
    });
    function encode(str) {
        const charMap = {
            '!': '%21',
            "'": '%27',
            '(': '%28',
            ')': '%29',
            '~': '%7E',
            '%20': '+',
            '%00': '\x00'
        };
        return encodeURIComponent(str).replace(/[!'()~]|%20|%00/g, function replacer(match) {
            return charMap[match];
        });
    }
    function AxiosURLSearchParams(params, options) {
        this._pairs = [];
        params;
    }
    const prototype = AxiosURLSearchParams.prototype;
    prototype.append = function append(name, value) {
        this._pairs.push([
            name,
            value
        ]);
    };
    prototype.toString = function toString(encoder) {
        const _encode = encoder ? function(value) {
            return encoder.call(this, value, encode);
        } : encode;
        return this._pairs.map(function each(pair) {
            return _encode(pair[0]) + '=' + _encode(pair[1]);
        }, '').join('&');
    };
    exports.default = AxiosURLSearchParams;
    function AxiosURLSearchParams2(params, options) {
        this._pairs = [];
        params;
    }
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_dep = module.w(farmRequire("05ee5ec7"));
    console.log(module.f(_f_dep), _f_dep.AxiosURLSearchParams2);
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");