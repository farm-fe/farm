//index.js:
 function addClass() {
    console.log('addClass');
}
var exportFn_ts_default = addClass;
class AddClass {
    constructor(){
        console.log('addClass');
    }
}
var exportClass_ts_default = AddClass;
var _addClass = function addClass() {
    console.log('addClass');
    exportFn_ts_default();
};
var _AddClass = class AddClass {
    constructor(){
        console.log('addClass');
        new exportClass_ts_default();
    }
};
