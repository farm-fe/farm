//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: foo.ts
const foo$2 = 'foo';

// module_id: index.ts
function loadFoo(foo = foo$2) {
    return {};
}
class LoadFoo {
    foo;
    constructor(foo = foo$2){
        this.foo = foo;
    }
    getFoo() {
        return this.foo;
    }
}
loadFoo();
new LoadFoo();
const bar = 2;
function computed(bar = 1) {
    return {
        [bar]: 123,
        [foo$2]: 234
    };
}
computed();
