//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};const MIN_CHART_WIDTH = 1;
const MIN_CHART_HEIGHT = 1;
function getElementSize(ele) {
    var style = getComputedStyle(ele);
    return {
        width: (ele.clientWidth || parseInt(style.width, 10)) - parseInt(style.paddingLeft, 10) - parseInt(style.paddingRight, 10),
        height: (ele.clientHeight || parseInt(style.height, 10)) - parseInt(style.paddingTop, 10) - parseInt(style.paddingBottom, 10)
    };
}
function isNumber(v) {
    return typeof v === 'number' && !isNaN(v);
}
function getChartSize(ele, autoFit, width, height) {
    var w = width;
    var h = height;
    if (autoFit) {
        var size = getElementSize(ele);
        w = size.width ? size.width : w;
        h = size.height ? size.height : h;
    }
    return {
        width: Math.max(isNumber(w) ? w : MIN_CHART_WIDTH, MIN_CHART_WIDTH),
        height: Math.max(isNumber(h) ? h : MIN_CHART_HEIGHT, MIN_CHART_HEIGHT)
    };
}
function removeDom(dom) {
    var parent = dom.parentNode;
    if (parent) {
        parent.removeChild(dom);
    }
}
var dep_ts_default = {
    getChartSize: getChartSize,
    removeDom: removeDom
};
var dep_ts_ns = {
    getChartSize: getChartSize,
    removeDom: removeDom,
    "default": dep_ts_default,
    __esModule: true
};

console.log(getChartSize, removeDom, dep_ts_ns, dep_ts_default);
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");