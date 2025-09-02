//index.js:
 const a$1 = 3;
const b$1 = 4;
const c = 5;
function BB() {
    const a = 5;
    const b = 6;
    console.log(a, b);
}
var dep_ts_default = {
    a: a$1,
    b: b$1,
    c
};
var dep_ts_namespace_farm_internal_ = {
    a: a$1,
    b: b$1,
    default: dep_ts_default,
    __esModule: true
};
var importNamespace_ts_default = dep_ts_namespace_farm_internal_;
var exportAll_ts_namespace_farm_internal_ = {
    a: a$1,
    b: b$1,
    __esModule: true
};
console.log({
    ExportNamespace: dep_ts_namespace_farm_internal_,
    A: exportAll_ts_namespace_farm_internal_,
    ImportNamespace: dep_ts_namespace_farm_internal_
});
