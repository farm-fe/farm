(function (modules) {
  for (var key in modules) {
    globalThis.__acquire_farm_module_system__().register(key, modules[key]);
  }

})(modules)