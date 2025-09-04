//index.js:
 ; // module_id: foo.ts
const foo$2 = 'foo';
; // module_id: bundle2-foo.ts
const foo$1 = {
    Provider: 'bundle2-foo'
};
; // module_id: index.ts
function loadFoo(foo = foo$2) {
    return;
}
const loadFooArrowExpr = ()=>{
    return (foo)=>{
        console.log(foo, foo$1.Provider);
    };
};
class LoadFoo {
    foo;
    constructor(foo = foo$2){
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
        [foo$2]: 234
    };
}
computed();
