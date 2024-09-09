//index.html:
 <!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Document</title>
<script>
window['__farm_default_namespace__'] = {};
window['__farm_default_namespace__'] = {
  __FARM_TARGET_ENV__: 'browser',
};</script><script>function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}function __commonJs(mod) {
  var module;
  return () => {
    if (module) {
      return module.exports;
    }
    module = {
      exports: {},
    };
    if(typeof mod === "function") {
      mod(module, module.exports);
    }else {
      mod[Object.keys(mod)[0]](module, module.exports);
    }
    return module.exports;
  };
}((function(){var index_js_cjs = __commonJs((module, exports)=>{
    "use strict";
    console.log('runtime/index.js');
    window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
})());</script></head>
<body>
  

<script src="/index_564c.js" data-farm-resource="true"></script><script>window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources(['index_564c.js']);</script><script>window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });</script><script>window['__farm_default_namespace__'].__farm_module_system__.setPublicPaths(['/']);</script><script>window['__farm_default_namespace__'].__farm_module_system__.bootstrap();</script><script>window['__farm_default_namespace__'].__farm_module_system__.require("7c4a34c2")</script></body></html>

//index_564c.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='index_564c.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"7c4a34c2":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log('1111');
}
,});