(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = '404_index_2817.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"95649cd7": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "menu.exception.404": "404",
        "exception.result.404.description": "Whoops, this page is gone.",
        "exception.result.404.retry": "Retry",
        "exception.result.404.back": "Back"
    },
    "zh-CN": {
        "menu.exception": "异常页",
        "menu.exception.404": "404",
        "exception.result.404.description": "抱歉，页面不见了～",
        "exception.result.404.retry": "重试",
        "exception.result.404.back": "返回"
    }
};
const _default = i18n;

},
"9e276ce5": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("95649cd7"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("a0d30f53"));
function Exception404() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement(_webreact.Result, {
        className: _indexmoduleless.default.result,
        status: "404",
        subTitle: t['exception.result.404.description'],
        extra: [
            _react.default.createElement(_webreact.Button, {
                key: "again",
                style: {
                    marginRight: 16
                }
            }, t['exception.result.404.retry']),
            _react.default.createElement(_webreact.Button, {
                key: "back",
                type: "primary"
            }, t['exception.result.404.back'])
        ]
    }));
}
const _default = Exception404;

},
"a0d30f53": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "result": `result-6350f714`,
    "wrapper": `wrapper-6350f714`
};

},});