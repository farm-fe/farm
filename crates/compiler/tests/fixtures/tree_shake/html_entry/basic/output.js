//index-564c.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "7c4a34c2": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log('1111');
    }
});


//index.html:
 <!doctype html><html lang=en><head>
  <meta charset=UTF-8>
  <meta http-equiv=X-UA-Compatible content="IE=edge">
  <meta name=viewport content="width=device-width, initial-scale=1.0">
  <title>Document</title>
<script>window['__farm_default_namespace__'] = {};window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};</script><script>(function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
})();</script></head>
<body>
  

<script src=/index-564c.js data-farm-resource=true></script><script></script><script>window['__farm_default_namespace__'].m.b();window['__farm_default_namespace__'].m.r("7c4a34c2");</script>