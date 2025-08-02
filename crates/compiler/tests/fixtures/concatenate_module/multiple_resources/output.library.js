//index.js:
 const __farm_internal_modules__ = {};
const __farm_internal_cache__ = {};
function farmRequire$2(id) {
    if (__farm_internal_cache__[id]) {
        var cachedModuleResult = __farm_internal_cache__[id].exports;
        return cachedModuleResult;
    }
    const initializer = __farm_internal_modules__[id];
    if (!initializer) {
        console.debug(`[Farm] Module "${id}" is not registered`);
        return {};
    }
    const module = __farm_internal_cache__[id] = {
        id,
        meta: {
            env: {}
        },
        exports: {},
        require: (moduleId)=>farmRequire$2(moduleId)
    };
    __farm_internal_cache__[id] = module;
    initializer(module, module.exports);
    return module.exports;
}
function farmRegister(id, module) {
    __farm_internal_modules__[id] = module;
    return ()=>farmRequire$2(id);
}
var farmRequire = farmRegister("01800dfe", function(module, exports) {
    Object.defineProperty(exports, "registerAction", {
        enumerable: true,
        get: function() {
            return (str)=>"interaction_2.registerAction";
        }
    });
    Object.defineProperty(exports, "registerEngine", {
        enumerable: true,
        get: function() {
            return (str)=>"interaction_2.registerEngine";
        }
    });
});
var __farm_cjs_exports__$2 = farmRequire();
var registerEngine = __farm_cjs_exports__$2.registerEngine, registerAction = __farm_cjs_exports__$2.registerAction;
function Route1Comp() {
    return registerAction('Route1Comp');
}
function Common2() {
    return 'Common2';
}
function Common1() {
    return 'Common1' + Common2();
}
function Route1() {
    return Route1Comp() + Common1() + registerEngine('route1');
}
var route1_ts_namespace_farm_internal_ = {
    get Route1 () {
        return Route1;
    },
    get registerAction () {
        return registerAction;
    },
    __esModule: true
};
defineExportStar(route1_ts_namespace_farm_internal_, __farm_cjs_exports__$2);
var farmRequire$1 = farmRegister("2dc19cb0", function(module, exports) {
    exports.isCjs = true;
    exports.dep2 = function(str) {
        return "dep2" + str;
    };
});
var __farm_cjs_exports__$3 = farmRequire$1();
function Common3() {
    return 'Common3';
}
function Route2Comp() {
    return route2_ts_namespace_farm_internal_.dep2('Route2Comp');
}
function Route2() {
    return "Route2" + Common1() + Common2() + Common3() + Route2Comp();
}
var route2_ts_namespace_farm_internal_ = {
    get Route2 () {
        return Route2;
    },
    __esModule: true
};
defineExportStar(route2_ts_namespace_farm_internal_, __farm_cjs_exports__$3);
Promise.resolve(route1_ts_namespace_farm_internal_).then((mod)=>console.log(mod));
Promise.resolve(route2_ts_namespace_farm_internal_).then((mod)=>console.log(mod));
