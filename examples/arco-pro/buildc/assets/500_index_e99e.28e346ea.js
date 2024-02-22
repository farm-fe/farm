(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = '500_index_e99e.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"0ec79c3f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const i18n = {
    "en-US": {
        "menu.exception": "Exception page",
        "menu.exception.500": "500",
        "exception.result.500.description": "Internal server error",
        "exception.result.500.back": "Back"
    },
    "zh-CN": {
        "menu.exception": "异常页",
        "menu.exception.500": "500",
        "exception.result.500.description": "抱歉，服务器出了点问题～",
        "exception.result.500.back": "返回"
    }
};
const _default = i18n;

},
"37a7b57f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("0ec79c3f"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("b4548c4a"));
function Exception500() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement(_webreact.Result, {
        className: _indexmoduleless.default.result,
        status: "500",
        subTitle: t['exception.result.500.description'],
        extra: _react.default.createElement(_webreact.Button, {
            key: "back",
            type: "primary"
        }, t['exception.result.500.back'])
    }));
}
const _default = Exception500;

},
"b4548c4a": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "result": `result-2ba8a165`,
    "wrapper": `wrapper-2ba8a165`
};

},});