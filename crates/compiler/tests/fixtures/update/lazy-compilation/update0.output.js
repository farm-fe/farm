({"dep.ts":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = 'dep';
}
,
"dep.ts.farm_dynamic_import_virtual_module":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_dep = module.w(farmRequire("dep.ts"));
    var ns = _f_dep;
    module.exports = ns;
}
,})
{}