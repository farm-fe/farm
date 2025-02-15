

export function initModuleSystem(ms) {
  ms.p = function() {
    console.log('plugin');
  }
}