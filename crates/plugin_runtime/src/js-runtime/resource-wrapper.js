(function (modules) {
  for (var key in modules) {
    // var __farm_global_this__ = globalThis || window || global || self;
    __farm_global_this__.__farm_module_system__.register(key, modules[key]);
  }
})(modules);
