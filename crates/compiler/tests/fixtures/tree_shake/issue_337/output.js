//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"d2214aaa":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"d2214aaa");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_a93b.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"066a321b":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return resolve;
        }
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
        return input.startsWith("//");
    }
    function isAbsolutePath(input) {
        return input.startsWith("/");
    }
    function isFileUrl(input) {
        return input.startsWith("file:");
    }
    function isRelative(input) {
        return /^[.?#]/.test(input);
    }
    function parseAbsoluteUrl(input) {
        const match = urlRegex.exec(input);
        return makeUrl(match[1], match[2] || "", match[3], match[4] || "", match[5] || "/", match[6] || "", match[7] || "");
    }
    function parseFileUrl(input) {
        const match = fileRegex.exec(input);
        const path = match[2];
        return makeUrl("file:", "", match[1] || "", "", isAbsolutePath(path) ? path : "/" + path, match[3] || "", match[4] || "");
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
            const url = parseAbsoluteUrl("http:" + input);
            url.scheme = "";
            url.type = UrlType.SchemeRelative;
            return url;
        }
        if (isAbsolutePath(input)) {
            const url = parseAbsoluteUrl("http://foo.com" + input);
            url.scheme = "";
            url.host = "";
            url.type = UrlType.AbsolutePath;
            return url;
        }
        if (isFileUrl(input)) return parseFileUrl(input);
        if (isAbsoluteUrl(input)) return parseAbsoluteUrl(input);
        const url = parseAbsoluteUrl("http://foo.com/" + input);
        url.scheme = "";
        url.host = "";
        url.type = input ? input.startsWith("?") ? UrlType.Query : input.startsWith("#") ? UrlType.Hash : UrlType.RelativePath : UrlType.Empty;
        return url;
    }
    function stripPathFilename(path) {
        if (path.endsWith("/..")) return path;
        const index = path.lastIndexOf("/");
        return path.slice(0, index + 1);
    }
    function mergePaths(url, base) {
        normalizePath(base, base.type);
        if (url.path === "/") {
            url.path = base.path;
        } else {
            url.path = stripPathFilename(base.path) + url.path;
        }
    }
    function normalizePath(url, type) {
        const rel = type <= UrlType.RelativePath;
        const pieces = url.path.split("/");
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
            if (piece === ".") continue;
            if (piece === "..") {
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
        let path = "";
        for(let i = 1; i < pointer; i++){
            path += "/" + pieces[i];
        }
        if (!path || addTrailingSlash && !path.endsWith("/..")) {
            path += "/";
        }
        url.path = path;
    }
    function resolve(input, base) {
        if (!input && !base) return "";
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
                    if (!path) return queryHash || ".";
                    if (isRelative(base || input) && !isRelative(path)) {
                        return "./" + path + queryHash;
                    }
                    return path + queryHash;
                }
            case UrlType.AbsolutePath:
                return url.path + queryHash;
            default:
                return url.scheme + "//" + url.user + url.host + url.port + url.path + queryHash;
        }
    }
}
,
"7cd09bc5":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _root = _interop_require_default._(farmRequire("b5147996"));
    var freeExports = typeof exports == "object" && exports && !exports.nodeType && exports;
    var freeModule = freeExports && typeof module == "object" && module && !module.nodeType && module;
    var moduleExports = freeModule && freeModule.exports === freeExports;
    var Buffer = moduleExports ? _root.default.Buffer : undefined, allocUnsafe = Buffer ? Buffer.allocUnsafe : undefined;
    function cloneBuffer(buffer, isDeep) {
        if (isDeep) {
            return buffer.slice();
        }
        var length = buffer.length, result = allocUnsafe ? allocUnsafe(length) : new buffer.constructor(length);
        buffer.copy(result);
        return result;
    }
    var _default = cloneBuffer;
}
,
"b5147996":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    var _default = "/home";
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _cloneBuffer = _interop_require_default._(farmRequire("7cd09bc5"));
    var _resolveuri = _interop_require_default._(farmRequire("066a321b"));
    console.log((0, _cloneBuffer.default)(Buffer.from("test")));
    console.log((0, _resolveuri.default)("test"));
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");