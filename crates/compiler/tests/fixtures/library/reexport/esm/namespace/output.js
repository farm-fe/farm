//index.js:
 var foo_ts_default = 'foo';
const foo$1 = 'foo';
const bar = 'bar';
var foo_ts_namespace_farm_internal_ = {
    get bar () {
        return bar;
    },
    get default () {
        return foo_ts_default;
    },
    get foo () {
        return foo$1;
    },
    __esModule: true
};
const foo = 123;
console.log(foo_ts_namespace_farm_internal_.default, foo_ts_namespace_farm_internal_.foo, foo);
export { foo_ts_namespace_farm_internal_ as ns };
