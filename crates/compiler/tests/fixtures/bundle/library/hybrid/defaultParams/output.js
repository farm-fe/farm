//bundle1.js:
 // module_id: bundle2-foo.ts
const foo$5 = {
    Provider: 'bundle2-foo'
};
export { foo$5 };


//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: foo.ts
import { foo$5 } from "./bundle1.js";
const foo$1 = 'foo';

// module_id: index.ts
function loadFoo(foo = foo$1) {
    return;
}
const loadFooArrowExpr = ()=>{
    return (foo)=>{
        console.log(foo, foo$5.Provider);
    };
};
class LoadFoo {
    foo;
    constructor(foo = foo$1){
        this.foo = foo;
    }
    getFoo() {
        return this.foo;
    }
}
loadFoo();
new LoadFoo();
const bar$1 = 2;
function computed(bar = 1) {
    return {
        [bar]: 123,
        [foo$1]: 234
    };
}
computed();
