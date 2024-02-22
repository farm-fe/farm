(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'success_index_9e4c.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"40b8d97c": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "menu.result.success": "Success",
        "success.result.title": "Submit Success",
        "success.result.subTitle": "Submit form success!",
        "success.result.printResult": "Print result",
        "success.result.projectList": "Project List",
        "success.result.progress": "Progress",
        "success.submitApplication": "Submit Application",
        "success.leaderReview": "Leader Review",
        "success.purchaseCertificate": "Purchase Certificate",
        "success.safetyTest": "Safety Test",
        "success.launched": "Officially launched",
        "success.waiting": "Waiting",
        "success.processing": "Processing"
    },
    "zh-CN": {
        "menu.result": "结果页",
        "menu.result.success": "成功页",
        "success.result.title": "提交成功",
        "success.result.subTitle": "表单提交成功！",
        "success.result.printResult": "打印结果",
        "success.result.projectList": "返回项目列表",
        "success.result.progress": "当前进度",
        "success.submitApplication": "提交申请",
        "success.leaderReview": "直属领导审核",
        "success.purchaseCertificate": "购买证书",
        "success.safetyTest": "安全测试",
        "success.launched": "正式上线",
        "success.waiting": "未开始",
        "success.processing": "进行中"
    }
};
const _default = i18n;

},
"87644080": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("40b8d97c"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("0c0c3088"));
const Step = _webreact.Steps.Step;
function Success() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement("div", null, _react.default.createElement("div", {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement(_webreact.Result, {
        className: _indexmoduleless.default.result,
        status: "success",
        title: t['success.result.title'],
        subTitle: t['success.result.subTitle'],
        extra: [
            _react.default.createElement(_webreact.Button, {
                key: "again",
                type: "secondary",
                style: {
                    marginRight: 16
                }
            }, t['success.result.printResult']),
            _react.default.createElement(_webreact.Button, {
                key: "back",
                type: "primary"
            }, t['success.result.projectList'])
        ]
    }), _react.default.createElement("div", {
        className: _indexmoduleless.default['steps-wrapper']
    }, _react.default.createElement(_webreact.Typography.Paragraph, {
        bold: true
    }, t['success.result.progress']), _react.default.createElement(_webreact.Steps, {
        type: "dot",
        current: 2
    }, _react.default.createElement(Step, {
        title: t['success.submitApplication'],
        description: "2020/10/10 14:00:39"
    }), _react.default.createElement(Step, {
        title: t['success.leaderReview'],
        description: t['success.processing']
    }), _react.default.createElement(Step, {
        title: t['success.purchaseCertificate'],
        description: t['success.waiting']
    }), _react.default.createElement(Step, {
        title: t['success.safetyTest'],
        description: t['success.waiting']
    }), _react.default.createElement(Step, {
        title: t['success.launched'],
        description: t['success.waiting']
    })))));
}
const _default = Success;

},});