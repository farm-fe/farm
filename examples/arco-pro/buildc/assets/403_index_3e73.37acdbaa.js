(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = '403_index_3e73.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"41bf5fbc": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "menu.exception.403": "403",
        "exception.result.403.description": "Access to this resource on the server is denied.",
        "exception.result.403.back": "Back"
    },
    "zh-CN": {
        "menu.exception": "异常页",
        "menu.exception.403": "403",
        "exception.result.403.description": "对不起，您没有访问该资源的权限",
        "exception.result.403.back": "返回"
    }
};
const _default = i18n;

},
"aac3abd4": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "result": `result-6399e96b`,
    "wrapper": `wrapper-6399e96b`
};

},
"d064950f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("41bf5fbc"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("aac3abd4"));
function Exception403() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.container
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement(_webreact.Result, {
        className: _indexmoduleless.default.result,
        status: "403",
        subTitle: t['exception.result.403.description'],
        extra: _react.default.createElement(_webreact.Button, {
            key: "back",
            type: "primary"
        }, t['exception.result.403.back'])
    })));
}
const _default = Exception403;

},});