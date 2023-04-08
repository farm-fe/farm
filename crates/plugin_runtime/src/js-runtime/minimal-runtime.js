// The smallest runtime that only has a really simple module system to load and execute modules synchronously.
// This minimal runtime is used for the Farm runtime and its plugins.

// should insert below var decl during compile
// var modules = { ... };
// var entryModule = 'xxx';

(function (modules, entryModule) {
  var cache = {};

  function require(id) {
    if (cache[id]) return cache[id].exports;

    var module = {
      id: id,
      exports: {}
    };

    modules[id](module, module.exports, require);
    cache[id] = module;
    return module.exports;
  }

  require(entryModule);
})(modules, entryModule)