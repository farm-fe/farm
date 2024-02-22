(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'monitor_index_6ab5.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"312b0baf": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return QuickOperation;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("84a52c12"));
function QuickOperation() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        style: {
            marginTop: 0,
            marginBottom: 16
        },
        heading: 6
    }, t['monitor.title.quickOperation']), _react.default.createElement(_webreact.Space, {
        direction: "vertical",
        style: {
            width: '100%'
        },
        size: 10
    }, _react.default.createElement(_webreact.Button, {
        long: true,
        icon: _react.default.createElement(_icon.IconTags, null)
    }, t['monitor.quickOperation.changeClarity']), _react.default.createElement(_webreact.Button, {
        long: true,
        icon: _react.default.createElement(_icon.IconSwap, null)
    }, t['monitor.quickOperation.switchStream']), _react.default.createElement(_webreact.Button, {
        long: true,
        icon: _react.default.createElement(_icon.IconStop, null)
    }, t['monitor.quickOperation.removeClarity']), _react.default.createElement(_webreact.Button, {
        long: true,
        icon: _react.default.createElement(_icon.IconArrowRight, null)
    }, t['monitor.quickOperation.pushFlowGasket'])));
}

},
"34db861b": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "chat-panel": `chat-panel-589231a3`,
    "chat-panel-content": `chat-panel-content-589231a3`,
    "data-statistic-content": `data-statistic-content-589231a3`,
    "data-statistic-list-content": `data-statistic-list-content-589231a3`,
    "data-statistic-list-cover-tag": `data-statistic-list-cover-tag-589231a3`,
    "data-statistic-list-cover-wrapper": `data-statistic-list-cover-wrapper-589231a3`,
    "data-statistic-list-header": `data-statistic-list-header-589231a3`,
    "data-statistic-list-tip": `data-statistic-list-tip-589231a3`,
    "layout": `layout-589231a3`,
    "layout-content": `layout-content-589231a3`,
    "layout-left-side": `layout-left-side-589231a3`,
    "layout-right-side": `layout-right-side-589231a3`,
    "studio-bar": `studio-bar-589231a3`,
    "studio-preview": `studio-preview-589231a3`,
    "studio-wrapper": `studio-wrapper-589231a3`
};

},
"64a87592": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
(0, _setupMock.default)({
    setup: ()=>{
        _mockjs.default.mock(new RegExp("/api/chatList"), ()=>{
            const data = _mockjs.default.mock({
                "data|4-6": [
                    {
                        "id|+1": 1,
                        username: "用户7352772",
                        content: "马上就开始了，好激动！",
                        time: "13:09:12",
                        "isCollect|2": true
                    }
                ]
            });
            return data.data;
        });
    }
});

},
"73189a4e": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "message-item": `message-item-8c867de7`,
    "message-item-actions": `message-item-actions-8c867de7`,
    "message-item-actions-collect": `message-item-actions-collect-8c867de7`,
    "message-item-actions-item": `message-item-actions-item-8c867de7`,
    "message-item-collected": `message-item-collected-8c867de7`,
    "message-item-footer": `message-item-footer-8c867de7`
};

},
"fb7cb8fb": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _item = /*#__PURE__*/ _interop_require_default._(farmRequire("5eff8891"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("73189a4e"));
function MessageList(props) {
    const { data = [] } = props;
    return _react.default.createElement("div", {
        className: _indexmoduleless.default['message-list']
    }, data.map((item)=>_react.default.createElement(_item.default, {
            key: item.id,
            data: item
        })), !data.length && _react.default.createElement(_webreact.Result, {
        status: "404"
    }));
}
const _default = MessageList;

},});