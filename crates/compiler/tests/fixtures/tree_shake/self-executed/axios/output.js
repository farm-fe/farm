//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "AxiosURLSearchParams2", function() {
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
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire.w(farmRequire("05ee5ec7"));
        console.log(farmRequire.f(_f_dep), _f_dep.AxiosURLSearchParams2);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");