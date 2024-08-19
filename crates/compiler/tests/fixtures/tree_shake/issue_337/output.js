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
    console.log('runtime/index.js')(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_a93b.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"066a321b":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "default", function() {
        return resolve;
    });
    const schemeRegex = /^[\w+.-]+:\/\//;
    const urlRegex = /^([\w+.-]+:)\/\/([^@/#?]*@)?([^:/#?]*)(:\d+)?(\/[^#?]*)?(\?[^#]*)?(#.*)?/;
    const fileRegex = /^file:(?:\/\/((?![a-z]:)[^/#?]*)?)?(\/?[^#?]*)(\?[^#]*)?(#.*)?/i;
    var UrlType;
    (function(UrlType) {
        UrlType[UrlType["Empty"] = 1] = "Empty";
        UrlType[UrlType["Hash"] = 2] = "Hash";
        UrlType[UrlType["Query"] = 3] = "Query";
        UrlType[UrlType["RelativePath"] = 4] = "RelativePath";
        UrlType[UrlType["AbsolutePath"] = 5] = "AbsolutePath";
        UrlType[UrlType["SchemeRelative"] = 6] = "SchemeRelative";
        UrlType[UrlType["Absolute"] = 7] = "Absolute";
    })(UrlType || (UrlType = {}));
    function isAbsoluteUrl(input) {
        return schemeRegex.test(input);
    }
    function isSchemeRelativeUrl(input) {
        return input.startsWith('//');
    }
    function isAbsolutePath(input) {
        return input.startsWith('/');
    }
    function isFileUrl(input) {
        return input.startsWith('file:');
    }
    function isRelative(input) {
        return /^[.?#]/.test(input);
    }
    function parseAbsoluteUrl(input) {
        const match = urlRegex.exec(input);
        return makeUrl(match[1], match[2] || '', match[3], match[4] || '', match[5] || '/', match[6] || '', match[7] || '');
    }
    function parseFileUrl(input) {
        const match = fileRegex.exec(input);
        const path = match[2];
        return makeUrl('file:', '', match[1] || '', '', isAbsolutePath(path) ? path : '/' + path, match[3] || '', match[4] || '');
    }
    function makeUrl(scheme, user, host, port, path, query, hash) {
        return {
            scheme,
            user,
            host,
            port,
            path,
            query,
            hash,
            type: UrlType.Absolute
        };
    }
    function parseUrl(input) {
        if (isSchemeRelativeUrl(input)) {
            const url = parseAbsoluteUrl('http:' + input);
            url.scheme = '';
            url.type = UrlType.SchemeRelative;
            return url;
        }
        if (isAbsolutePath(input)) {
            const url = parseAbsoluteUrl('http://foo.com' + input);
            url.scheme = '';
            url.host = '';
            url.type = UrlType.AbsolutePath;
            return url;
        }
        if (isFileUrl(input)) return parseFileUrl(input);
        if (isAbsoluteUrl(input)) return parseAbsoluteUrl(input);
        const url = parseAbsoluteUrl('http://foo.com/' + input);
        url.scheme = '';
        url.host = '';
        url.type = input ? input.startsWith('?') ? UrlType.Query : input.startsWith('#') ? UrlType.Hash : UrlType.RelativePath : UrlType.Empty;
        return url;
    }
    function stripPathFilename(path) {
        if (path.endsWith('/..')) return path;
        const index = path.lastIndexOf('/');
        return path.slice(0, index + 1);
    }
    function mergePaths(url, base) {
        normalizePath(base, base.type);
        if (url.path === '/') {
            url.path = base.path;
        } else {
            url.path = stripPathFilename(base.path) + url.path;
        }
    }
    function normalizePath(url, type) {
        const rel = type <= UrlType.RelativePath;
        const pieces = url.path.split('/');
        let pointer = 1;
        let positive = 0;
        let addTrailingSlash = false;
        for(let i = 1; i < pieces.length; i++){
            const piece = pieces[i];
            if (!piece) {
                addTrailingSlash = true;
                continue;
            }
            addTrailingSlash = false;
            if (piece === '.') continue;
            if (piece === '..') {
                if (positive) {
                    addTrailingSlash = true;
                    positive--;
                    pointer--;
                } else if (rel) {
                    pieces[pointer++] = piece;
                }
                continue;
            }
            pieces[pointer++] = piece;
            positive++;
        }
        let path = '';
        for(let i = 1; i < pointer; i++){
            path += '/' + pieces[i];
        }
        if (!path || addTrailingSlash && !path.endsWith('/..')) {
            path += '/';
        }
        url.path = path;
    }
    function resolve(input, base) {
        if (!input && !base) return '';
        const url = parseUrl(input);
        let inputType = url.type;
        if (base && inputType !== UrlType.Absolute) {
            const baseUrl = parseUrl(base);
            const baseType = baseUrl.type;
            switch(inputType){
                case UrlType.Empty:
                    url.hash = baseUrl.hash;
                case UrlType.Hash:
                    url.query = baseUrl.query;
                case UrlType.Query:
                case UrlType.RelativePath:
                    mergePaths(url, baseUrl);
                case UrlType.AbsolutePath:
                    url.user = baseUrl.user;
                    url.host = baseUrl.host;
                    url.port = baseUrl.port;
                case UrlType.SchemeRelative:
                    url.scheme = baseUrl.scheme;
            }
            if (baseType > inputType) inputType = baseType;
        }
        normalizePath(url, inputType);
        const queryHash = url.query + url.hash;
        switch(inputType){
            case UrlType.Hash:
            case UrlType.Query:
                return queryHash;
            case UrlType.RelativePath:
                {
                    const path = url.path.slice(1);
                    if (!path) return queryHash || '.';
                    if (isRelative(base || input) && !isRelative(path)) {
                        return './' + path + queryHash;
                    }
                    return path + queryHash;
                }
            case UrlType.AbsolutePath:
                return url.path + queryHash;
            default:
                return url.scheme + '//' + url.user + url.host + url.port + url.path + queryHash;
        }
    }
}
,
"7cd09bc5":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f__root = module.i(farmRequire("b5147996"));
    var freeExports = typeof exports == 'object' && exports && !exports.nodeType && exports;
    var freeModule = freeExports && typeof module == 'object' && module && !module.nodeType && module;
    var moduleExports = freeModule && freeModule.exports === freeExports;
    var Buffer = moduleExports ? module.f(_f__root).Buffer : undefined, allocUnsafe = Buffer ? Buffer.allocUnsafe : undefined;
    function cloneBuffer(buffer, isDeep) {
        if (isDeep) {
            return buffer.slice();
        }
        var length = buffer.length, result = allocUnsafe ? allocUnsafe(length) : new buffer.constructor(length);
        buffer.copy(result);
        return result;
    }
    exports.default = cloneBuffer;
}
,
"b5147996":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = '/home';
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f__cloneBuffer = module.i(farmRequire("7cd09bc5"));
    var _f_resolve_uri = module.i(farmRequire("066a321b"));
    console.log(module.f(_f__cloneBuffer)(Buffer.from("test")));
    console.log(module.f(_f_resolve_uri)("test"));
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");