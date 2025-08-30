//index.js:
 const namedA = 10;
const namedB = 20;
const namedC = 30;
const sameNameWithFile_ts = 1;
const sameNameWithFile_ts_ns = 2;
var exportNamed_ts_default = {
    renamedA: namedA,
    renamedB: namedB,
    renamedC: namedC
};
var exportNamed_ts_namespace_farm_internal_ = {
    default: exportNamed_ts_default,
    namedA: namedA,
    namedB: namedB,
    namedC: namedC,
    renamedA: namedA,
    renamedB: namedB,
    renamedC: namedC,
    __esModule: true
};
console.log('export expr');
var exportExpr_ts_default$1 = 'export expr';
const sameNameWithFile_ts_ns$1 = 1;
const sameNameWithFile_ts$1 = 2;
const exportExpr_ts_default = 3;
function say() {
    console.log('hello');
}
var sameNameWithFile_ts_namespace_farm_internal_ = {
    say: say,
    __esModule: true
};
console.log({
    NamedNamespace: exportNamed_ts_namespace_farm_internal_,
    namedA: namedA,
    namedB: namedB,
    namedC: namedC,
    DefaultNamed: exportNamed_ts_default,
    SameNameWithFileNamespace: sameNameWithFile_ts_namespace_farm_internal_,
    Expr: exportExpr_ts_default$1
});
