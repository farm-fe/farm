(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'welcome_index_b08d.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"89272fed": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return Welcome;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _codeblock = /*#__PURE__*/ _interop_require_default._(farmRequire("3ef33d8a"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("8f54af56"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("1123b6ff"));
function Welcome() {
    const t = (0, _useLocale.default)(_locale.default);
    const userInfo = (0, _reactredux.useSelector)((state)=>state.userInfo) || {};
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.container
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default.header
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 5,
        style: {
            marginTop: 0
        }
    }, t['welcome.title.welcome']), _react.default.createElement(_webreact.Typography.Text, {
        type: "secondary"
    }, userInfo.name, ", ", userInfo.email)), _react.default.createElement("div", null, _react.default.createElement(_webreact.Alert, {
        type: "success",
        content: t['welcome.invite']
    }), _react.default.createElement(_webreact.Card, {
        style: {
            marginTop: 20
        },
        title: t['welcome.usage']
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6,
        style: {
            marginTop: 0
        }
    }, "1. ", t['welcome.step.title.pickup']), _react.default.createElement(_webreact.Typography.Text, null, t['welcome.step.content.pickup'], _react.default.createElement(_webreact.Tag, {
        style: {
            marginLeft: 8
        }
    }, "@arco-design/pro-pages-workplace")), _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, "2. ", t['welcome.step.title.install']), _react.default.createElement(_webreact.Typography.Text, null, t['welcome.step.content.install']), _react.default.createElement(_codeblock.default, {
        code: "arco block use @arco-design/pro-pages-workplace"
    }), _react.default.createElement(_webreact.Typography.Title, {
        heading: 6,
        style: {
            marginTop: 0
        }
    }, "3. ", t['welcome.step.title.result']), _react.default.createElement(_webreact.Typography.Text, null, t['welcome.step.content.result'])), _react.default.createElement(_webreact.Card, {
        style: {
            marginTop: 20
        }
    }, _react.default.createElement(_webreact.Typography.Text, null, t['welcome.title.material']), _react.default.createElement("div", {
        style: {
            marginTop: 8
        }
    }, _react.default.createElement(_webreact.Link, {
        target: "_blank",
        href: "https://arco.design/material?category=arco-design-pro"
    }, t['welcome.link.material-pro'], " ", _react.default.createElement(_icon.IconDoubleRight, null))), _react.default.createElement("div", {
        style: {
            marginTop: 8
        }
    }, _react.default.createElement(_webreact.Link, {
        target: "_blank",
        href: "https://arco.design/material"
    }, t['welcome.link.material-all'], " ", _react.default.createElement(_icon.IconDoubleRight, null))))));
}

},});