//ac35b2ca.js:
 (function(modules) {
    for(var key in modules){
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "7c4a34c2": function(module, exports, farmRequire, dynamicRequire) {
        console.log("1111");
    }
});


//index.html:
 <!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Document</title>
<script data-farm-entry-script="true">
window.process = {
  env: {
    NODE_ENV: 'production',
  },
};
window.__FARM_TARGET_ENV__ = 'browser';
</script><script data-farm-entry-script="true">(function(modules, entryModule) {
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
})({
    "ec853507": function(module, exports, farmRequire, dynamicRequire) {
        "use strict";
        console.log("runtime/index.js");
        __farm_global_this__.__farm_module_system__.setPlugins([]);
    }
}, "ec853507");
</script></head>
<body>
  

<script src="/ac35b2ca.js"></script><script data-farm-entry-script="true">__farm_module_system__.setInitialLoadedResources(['ac35b2ca.js']);</script><script data-farm-entry-script="true">__farm_module_system__.setDynamicModuleResourcesMap({  });</script><script data-farm-entry-script="true">__farm_module_system__.setPublicPaths(['/']);</script><script data-farm-entry-script="true">__farm_module_system__.bootstrap();</script><script data-farm-entry-script="true">__farm_module_system__.require("7c4a34c2")</script></body></html>