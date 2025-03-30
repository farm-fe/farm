//index.js:
 var export_class_ts_default = class {
    constructor(){
        console.log('class foo');
    }
};
var export_fn_ts_default = function() {
    console.log('fn foo');
};
console.log({
    FooClass: export_class_ts_default,
    FooFn: export_fn_ts_default
});
