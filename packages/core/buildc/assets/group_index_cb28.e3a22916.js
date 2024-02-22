(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'group_index_cb28.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"a84302f3": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
(0, _setupMock.default)({
    setup: ()=>{
        // 保存表单数据
        _mockjs.default.mock(new RegExp("/api/groupForm"), ()=>{
            return true;
        });
    }
});

},
"da4e9fba": function(module, exports, farmRequire, farmDynamicRequire) {
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
"";
const _default = {
    "actions": `actions-867a8a5a`,
    "container": `container-867a8a5a`
};

},});