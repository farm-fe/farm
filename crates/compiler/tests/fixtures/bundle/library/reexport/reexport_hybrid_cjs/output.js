//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: default/reexport.ts
class foo {
    constructor(){
        console.log(this.constructor === foo);
    }
}
const bar = 'foo';
var reexport_ns = {
    bar: bar,
    default: foo,
    __esModule: true
};

// module_id: default/index.ts
export { bar, foo as default };
