//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: foo.ts
var foo_default = 'foo';
const foo = 'foo';
const bar = 'bar';
var foo_ns = {
    bar: bar,
    foo: foo,
    "default": foo_default,
    __esModule: true
};

// module_id: index.ts
const foo$1 = 123;
console.log(foo_ns.default, foo_ns.foo, foo$1);
export { foo_ns as ns };
