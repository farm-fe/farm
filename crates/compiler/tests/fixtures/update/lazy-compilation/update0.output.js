({"dep.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    var _default = "dep";
}
,
"dep.ts.farm_dynamic_import_virtual_module":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    var _depts = _interop_require_default._(_export_star._(farmRequire("dep.ts"), exports));
    var _default = _depts.default;
}
,})
{}