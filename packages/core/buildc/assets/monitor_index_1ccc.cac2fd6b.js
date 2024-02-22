(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'monitor_index_1ccc.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"2f452781": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return Monitor;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _chatpanel = /*#__PURE__*/ _interop_require_default._(farmRequire("b5b21312"));
const _datastatistic = /*#__PURE__*/ _interop_require_default._(farmRequire("fed60230"));
farmRequire("64a87592");
const _quickoperation = /*#__PURE__*/ _interop_require_default._(farmRequire("312b0baf"));
const _studio = /*#__PURE__*/ _interop_require_default._(farmRequire("d90a36b4"));
const _studioinformation = /*#__PURE__*/ _interop_require_default._(farmRequire("b2e65c1c"));
const _studiostatus = /*#__PURE__*/ _interop_require_default._(farmRequire("380fb846"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("34db861b"));
function Monitor() {
    const userInfo = (0, _reactredux.useSelector)((state)=>state.userInfo);
    return _react.default.createElement("div", null, _react.default.createElement("div", {
        className: _indexmoduleless.default.layout
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['layout-left-side']
    }, _react.default.createElement(_chatpanel.default, null)), _react.default.createElement("div", {
        className: _indexmoduleless.default['layout-content']
    }, _react.default.createElement(_webreact.Space, {
        size: 16,
        direction: "vertical",
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_studio.default, {
        userInfo: userInfo
    }), _react.default.createElement(_datastatistic.default, null))), _react.default.createElement("div", {
        className: _indexmoduleless.default['layout-right-side']
    }, _react.default.createElement(_webreact.Space, {
        size: 16,
        direction: "vertical",
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_studiostatus.default, null), _react.default.createElement(_quickoperation.default, null), _react.default.createElement(_studioinformation.default, null)))));
}

},
"5eff8891": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("73189a4e"));
function MessageItem(props) {
    const { data = {} } = props;
    const classNames = (0, _classnames.default)(_indexmoduleless.default['message-item'], {
        [_indexmoduleless.default['message-item-collected']]: data.isCollect
    });
    return _react.default.createElement("div", {
        className: classNames
    }, _react.default.createElement(_webreact.Space, {
        size: 4,
        direction: "vertical",
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_webreact.Typography.Text, {
        type: "warning"
    }, data.username), _react.default.createElement(_webreact.Typography.Text, null, data.content), _react.default.createElement("div", {
        className: _indexmoduleless.default['message-item-footer']
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['message-item-time']
    }, _react.default.createElement(_webreact.Typography.Text, {
        type: "secondary"
    }, data.time)), _react.default.createElement("div", {
        className: _indexmoduleless.default['message-item-actions']
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['message-item-actions-item']
    }, _react.default.createElement(_icon.IconCommand, null)), _react.default.createElement("div", {
        className: (0, _classnames.default)(_indexmoduleless.default['message-item-actions-item'], _indexmoduleless.default['message-item-actions-collect'])
    }, _react.default.createElement(_icon.IconStar, null))))));
}
const _default = MessageItem;

},
"b2e65c1c": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return StudioInformation;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("84a52c12"));
function StudioInformation() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        style: {
            marginTop: 0,
            marginBottom: 16
        },
        heading: 6
    }, t['monitor.title.studioInfo']), _react.default.createElement(_webreact.Form, {
        layout: "vertical"
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['monitor.studioInfo.label.studioTitle'],
        required: true
    }, _react.default.createElement(_webreact.Input, {
        placeholder: `王立群${t['monitor.studioInfo.placeholder.studioTitle']}`
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['monitor.studioInfo.label.onlineNotification'],
        required: true
    }, _react.default.createElement(_webreact.Input.TextArea, null)), _react.default.createElement(_webreact.Form.Item, {
        label: t['monitor.studioInfo.label.studioCategory'],
        required: true
    }, _react.default.createElement(_webreact.Input.Search, null)), _react.default.createElement(_webreact.Form.Item, {
        label: t['monitor.studioInfo.label.studioCategory'],
        required: true
    }, _react.default.createElement(_webreact.Input.Search, null))), _react.default.createElement(_webreact.Button, {
        type: "primary"
    }, "更新"));
}

},
"fed60230": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return DataStatistic;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _datastatisticlist = /*#__PURE__*/ _interop_require_default._(farmRequire("9f47c3da"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("84a52c12"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("34db861b"));
function DataStatistic() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Tabs, {
        defaultActiveTab: "liveMethod"
    }, _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "liveMethod",
        title: t['monitor.tab.title.liveMethod']
    }), _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "onlineUsers",
        title: t['monitor.tab.title.onlineUsers']
    })), _react.default.createElement("div", {
        className: _indexmoduleless.default['data-statistic-content']
    }, _react.default.createElement(_webreact.Radio.Group, {
        defaultValue: "3",
        type: "button"
    }, _react.default.createElement(_webreact.Radio, {
        value: "1"
    }, t['monitor.liveMethod.normal']), _react.default.createElement(_webreact.Radio, {
        value: "2"
    }, t['monitor.liveMethod.flowControl']), _react.default.createElement(_webreact.Radio, {
        value: "3"
    }, t['monitor.liveMethod.video']), _react.default.createElement(_webreact.Radio, {
        value: "4"
    }, t['monitor.liveMethod.web'])), _react.default.createElement("div", {
        className: _indexmoduleless.default['data-statistic-list-wrapper']
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['data-statistic-list-header']
    }, _react.default.createElement(_webreact.Button, {
        type: "text"
    }, t['monitor.editCarousel']), _react.default.createElement(_webreact.Button, {
        disabled: true
    }, t['monitor.startCarousel'])), _react.default.createElement("div", {
        className: _indexmoduleless.default['data-statistic-list-content']
    }, _react.default.createElement(_datastatisticlist.default, null)))));
}

},});