(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'monitor_index_3ab1.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"9f47c3da": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("84a52c12"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("34db861b"));
function QuickOperation() {
    const t = (0, _useLocale.default)(_locale.default);
    const columns = [
        {
            title: t['monitor.list.title.order'],
            render: (_col, _record, index)=>_react.default.createElement("span", null, index + 1)
        },
        {
            title: t['monitor.list.title.cover'],
            dataIndex: 'cover',
            render: (_col, record)=>_react.default.createElement("div", {
                    className: _indexmoduleless.default['data-statistic-list-cover-wrapper']
                }, _react.default.createElement("img", {
                    src: record.cover
                }), record.status === -1 && _react.default.createElement(_webreact.Tag, {
                    color: "red",
                    className: _indexmoduleless.default['data-statistic-list-cover-tag']
                }, t['monitor.list.tag.auditFailed']))
        },
        {
            title: t['monitor.list.title.name'],
            dataIndex: 'name'
        },
        {
            dataIndex: 'duration',
            title: t['monitor.list.title.duration']
        },
        {
            dataIndex: 'id',
            title: t['monitor.list.title.id']
        }
    ];
    const data = [
        {
            cover: 'http://p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/c788fc704d32cf3b1136c7d45afc2669.png~tplv-uwbnlip3yd-webp.webp',
            name: '视频直播',
            duration: '00:05:19',
            id: '54e23ade',
            status: -1
        }
    ];
    return _react.default.createElement("div", {
        className: _indexmoduleless.default['']
    }, _react.default.createElement(_webreact.Table, {
        columns: columns,
        data: data,
        rowKey: "id",
        rowSelection: {
            type: 'checkbox'
        },
        border: false,
        pagination: false
    }), _react.default.createElement(_webreact.Typography.Text, {
        type: "secondary",
        className: _indexmoduleless.default['data-statistic-list-tip']
    }, t['monitor.list.tip.rotations'], data.length, t['monitor.list.tip.rest']));
}

},
"b5b21312": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return ChatPanel;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("84a52c12"));
const _messagelist = /*#__PURE__*/ _interop_require_default._(farmRequire("fb7cb8fb"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("34db861b"));
function ChatPanel() {
    const t = (0, _useLocale.default)(_locale.default);
    const [messageList, setMessageList] = (0, _react.useState)([]);
    const [loading, setLoading] = (0, _react.useState)(false);
    function fetchMessageList() {
        setLoading(true);
        _axios.default.get('/api/chatList').then((res)=>{
            setMessageList(res.data || []);
        }).finally(()=>{
            setLoading(false);
        });
    }
    (0, _react.useEffect)(()=>{
        fetchMessageList();
    }, []);
    return _react.default.createElement("div", {
        className: _indexmoduleless.default['chat-panel']
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['chat-panel-header']
    }, _react.default.createElement(_webreact.Typography.Title, {
        style: {
            marginTop: 0,
            marginBottom: 16
        },
        heading: 6
    }, t['monitor.title.chatPanel']), _react.default.createElement(_webreact.Space, {
        size: 8
    }, _react.default.createElement(_webreact.Select, {
        style: {
            width: 80
        },
        defaultValue: "all"
    }, _react.default.createElement(_webreact.Select.Option, {
        value: "all"
    }, t['monitor.chat.options.all'])), _react.default.createElement(_webreact.Input.Search, {
        placeholder: t['monitor.chat.placeholder.searchCategory']
    }), _react.default.createElement(_webreact.Button, {
        type: "text",
        iconOnly: true
    }, _react.default.createElement(_icon.IconDownload, null)))), _react.default.createElement("div", {
        className: _indexmoduleless.default['chat-panel-content']
    }, _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_messagelist.default, {
        data: messageList
    }))), _react.default.createElement("div", {
        className: _indexmoduleless.default['chat-panel-footer']
    }, _react.default.createElement(_webreact.Space, {
        size: 8
    }, _react.default.createElement(_webreact.Input, {
        suffix: _react.default.createElement(_icon.IconFaceSmileFill, null)
    }), _react.default.createElement(_webreact.Button, {
        type: "primary"
    }, t['monitor.chat.update']))));
}

},
"d90a36b4": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return Studio;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("84a52c12"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("34db861b"));
function Studio(props) {
    const t = (0, _useLocale.default)(_locale.default);
    const { userInfo } = props;
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Grid.Row, null, _react.default.createElement(_webreact.Grid.Col, {
        span: 16
    }, _react.default.createElement(_webreact.Typography.Title, {
        style: {
            marginTop: 0,
            marginBottom: 16
        },
        heading: 6
    }, t['monitor.title.studioPreview'])), _react.default.createElement(_webreact.Grid.Col, {
        span: 8,
        style: {
            textAlign: 'right'
        }
    }, _react.default.createElement(_icon.IconMore, null))), _react.default.createElement("div", {
        className: _indexmoduleless.default['studio-wrapper']
    }, _react.default.createElement("img", {
        src: "http://p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/c788fc704d32cf3b1136c7d45afc2669.png~tplv-uwbnlip3yd-webp.webp",
        className: _indexmoduleless.default['studio-preview']
    }), _react.default.createElement("div", {
        className: _indexmoduleless.default['studio-bar']
    }, userInfo && _react.default.createElement("div", null, _react.default.createElement(_webreact.Space, {
        size: 12
    }, _react.default.createElement(_webreact.Avatar, {
        size: 24
    }, _react.default.createElement("img", {
        src: userInfo.avatar
    })), _react.default.createElement(_webreact.Typography.Text, null, userInfo.name, t['monitor.studioPreview.studio']))), _react.default.createElement(_webreact.Typography.Text, {
        type: "secondary"
    }, "3,6000 ", t['monitor.studioPreview.watching']))));
}

},});