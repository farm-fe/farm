//index.js:
 const a = 3;
const b = 4;
const c = 5;
function BB() {
    const a = 5;
    const b = 6;
    console.log(a, b);
}
var dep_ts_default = {
    a,
    b,
    c
};
var dep_ts_namespace_farm_internal_ = {
    a: a,
    b: b,
    default: dep_ts_default,
    __esModule: true
};
var importNamespace_ts_default = dep_ts_namespace_farm_internal_;
var exportAll_ts_namespace_farm_internal_ = {
    a: a,
    b: b,
    __esModule: true
};
console.log({
    ExportNamespace: dep_ts_namespace_farm_internal_,
    A: exportAll_ts_namespace_farm_internal_,
    ImportNamespace: dep_ts_namespace_farm_internal_
});
