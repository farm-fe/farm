//index.js:
 const moduleSystem = {};
function initModuleSystem(ms) {
    ms.p = function() {
        console.log('plugin');
    };
}
function initModuleSystem$1(ms) {
    ms._m = function() {
        console.log('module-system');
    };
}
initModuleSystem(moduleSystem);
initModuleSystem$1(moduleSystem);
