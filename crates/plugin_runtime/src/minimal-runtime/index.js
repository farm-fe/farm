// The smallest runtime that only has a really simple module system to load and execute modules synchronously.
// And only works with a single resource.

// should insert below var decl during compile
// var module = { ... };
// var entryModule = 'xxx';

!(function (modules, entryModule) {
  var cache = {};

  function require(id) {
    if (cache[id]) return cache[id];

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