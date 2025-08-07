//index.js:
 function foo$1() {
    console.log('foo');
}
var dynamic_ts_namespace_farm_internal_ = {
    default: foo$1,
    __esModule: true
};
function sameNameDynamic() {
    console.log('sameNameDynamic');
}
var dynamic_ts_namespace_farm_internal_$1 = {
    default: sameNameDynamic,
    __esModule: true
};
Promise.resolve(dynamic_ts_namespace_farm_internal_).then((res)=>res.default());
const foo = ()=>Promise.resolve(dynamic_ts_namespace_farm_internal_);
function loader(m) {}
loader(Promise.resolve(dynamic_ts_namespace_farm_internal_));
Promise.resolve(dynamic_ts_namespace_farm_internal_);
const data = {
    foo: Promise.resolve(dynamic_ts_namespace_farm_internal_)
};
{
    Promise.resolve(dynamic_ts_namespace_farm_internal_$1);
}