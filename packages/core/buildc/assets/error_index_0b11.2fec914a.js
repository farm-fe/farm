(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'error_index_0b11.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"31dd8c8e": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("775ee20d"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("9ae4dfff"));
function Success() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement("div", null, _react.default.createElement("div", {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement(_webreact.Result, {
        className: _indexmoduleless.default.result,
        status: "error",
        title: t['error.result.title'],
        subTitle: t['error.result.subTitle'],
        extra: [
            _react.default.createElement(_webreact.Button, {
                key: "again",
                type: "secondary",
                style: {
                    marginRight: 16
                }
            }, t['error.result.goBack']),
            _react.default.createElement(_webreact.Button, {
                key: "back",
                type: "primary"
            }, t['error.result.retry'])
        ]
    }), _react.default.createElement("div", {
        className: _indexmoduleless.default['details-wrapper']
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6,
        style: {
            marginTop: 0
        }
    }, t['error.detailTitle']), _react.default.createElement(_webreact.Typography.Paragraph, {
        style: {
            marginBottom: 0
        }
    }, _react.default.createElement("ol", null, _react.default.createElement("li", null, t['error.detailLine.record'], _react.default.createElement(_webreact.Link, null, _react.default.createElement(_icon.IconLink, null), t['error.detailLine.record.link'])), _react.default.createElement("li", null, t['error.detailLine.auth'], _react.default.createElement(_webreact.Link, null, t['error.detailLine.auth.link'])))))));
}
const _default = Success;

},
"775ee20d": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "menu.result": "Result",
        "menu.result.error": "Error",
        "error.result.title": "Submit Error",
        "error.result.subTitle": "Please check the modified information and try again",
        "error.result.goBack": "GoBack",
        "error.result.retry": "Retry",
        "error.detailTitle": "Details of Error",
        "error.detailLine.record": "The current domain name has not been registered, please check the registration process: ",
        "error.detailLine.record.link": "Registration Process",
        "error.detailLine.auth": "Your user group does not have the authority to perform this operation;",
        "error.detailLine.auth.link": "Request for access"
    },
    "zh-CN": {
        "menu.result": "结果页",
        "menu.result.error": "失败页",
        "error.result.title": "提交失败",
        "error.result.subTitle": "请核对修改信息后，再重试",
        "error.result.goBack": "回到首页",
        "error.result.retry": "返回修改",
        "error.detailTitle": "错误详情",
        "error.detailLine.record": "当前域名未备案，备案流程请查看：",
        "error.detailLine.record.link": "备案流程",
        "error.detailLine.auth": "你的用户组不具有进行此操作的权限；",
        "error.detailLine.auth.link": "申请权限"
    }
};
const _default = i18n;

},});