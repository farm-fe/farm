//index.js:
 ; // module_id: foo.ts
const foo$1 = 'foo';
; // module_id: bundle2-foo.ts
const foo = {
    Provider: 'bundle2-foo'
};
; // module_id: index.ts
var foo$2 = foo;
function loadFoo(foo = foo$1) {
    return;
}
const loadFooArrowExpr = ()=>{
    return (foo)=>{
        console.log(foo, foo$2.Provider);
    };
};
class LoadFoo {
    foo;
    constructor(foo = foo$1){
        this.foo = foo;
        loadFooArrowExpr()(this.foo);
    }
    getFoo() {
        return this.foo;
    }
}
loadFoo();
new LoadFoo();
function computed(bar = 1) {
    return {
        [bar]: 123,
        [foo$1]: 234
    };
}
computed();
