//ac35b2ca.js:
 (function(modules) {
    for(var key in modules){
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "main.ts": function(module, exports, require, dynamicRequire) {
        "use strict";
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
    NODE_ENV: 'development',
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
    "../../../_internal/runtime/index.js.farm-runtime": function(module, exports, require, dynamicRequire) {
        "use strict";
        console.log("runtime/index.js");
        __farm_global_this__.__farm_module_system__.setPlugins([]);
    }
}, "../../../_internal/runtime/index.js.farm-runtime");
</script></head>
<body>
  

<script src="ac35b2ca.js"></script><script data-farm-entry-script="true">__farm_module_system__.setInitialLoadedResources(['ac35b2ca.js']);</script><script data-farm-entry-script="true">__farm_module_system__.setDynamicModuleResourcesMap({  });</script><script data-farm-entry-script="true">__farm_module_system__.setPublicPaths(['/']);</script><script data-farm-entry-script="true">__farm_module_system__.bootstrap();</script><script data-farm-entry-script="true">__farm_module_system__.require("main.ts")</script></body></html>