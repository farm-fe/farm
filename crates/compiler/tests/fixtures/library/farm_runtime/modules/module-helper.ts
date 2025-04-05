export function initModuleSystem(ms) {
  ms._m = function() {
    console.log('module-system');
  }
}