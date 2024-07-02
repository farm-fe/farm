({"dep.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    exports.default = "dep";
    module._m(exports);
}
,
"dep.ts.farm_dynamic_import_virtual_module":function  (module, exports, farmRequire, farmDynamicRequire) {
    var _f_dep = module.i(farmRequire("dep.ts"));
    exports.default = module.f(_f_dep);
    var _f_dep1 = farmRequire("dep.ts");
    module._e(exports, _f_dep1);
    module._m(exports);
}
,})
{}