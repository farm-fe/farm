(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_4b86.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"aad0a7a8": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
const _default = (config)=>{
    // const { mock = "production" === "development", setup } = config;
    // if (mock === false) return;
    // setup();
    const { setup } = config;
    setup();
};

},});