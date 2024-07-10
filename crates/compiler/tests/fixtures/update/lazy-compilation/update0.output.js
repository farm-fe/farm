({"dep.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = 'dep';
}
,
"dep.ts.farm_dynamic_import_virtual_module":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_dep = module.i(farmRequire("dep.ts"));
    exports.default = module.f(_f_dep);
    var _f_dep1 = farmRequire("dep.ts");
    module._e(exports, _f_dep1);
}
,})
{}