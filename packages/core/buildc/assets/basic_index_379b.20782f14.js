(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'basic_index_379b.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"beb95129": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _item = /*#__PURE__*/ _interop_require_default._(farmRequire("3fec3bea"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("e9860c88"));
farmRequire("4e842562");
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("f0e1de71"));
function BasicProfile() {
    const t = (0, _useLocale.default)(_locale.default);
    const [loading, setLoading] = (0, _react.useState)(false);
    const [data, setData] = (0, _react.useState)({
        status: 1
    });
    const [preLoading, setPreLoading] = (0, _react.useState)(false);
    const [preData, setPreData] = (0, _react.useState)({});
    const [tableLoading, setTableLoading] = (0, _react.useState)(false);
    const [tableData, setTableData] = (0, _react.useState)([]);
    function fetchData() {
        setLoading(true);
        _axios.default.get('/api/basicProfile').then((res)=>{
            setData(res.data || {});
        }).finally(()=>{
            setLoading(false);
        });
    }
    function fetchPreData() {
        setPreLoading(true);
        _axios.default.get('/api/basicProfile').then((res)=>{
            setPreData(res.data || {});
        }).finally(()=>{
            setPreLoading(false);
        });
    }
    function fetchTableData() {
        setTableLoading(true);
        _axios.default.get('/api/adjustment').then((res)=>{
            setTableData(res.data);
        }).finally(()=>{
            setTableLoading(false);
        });
    }
    (0, _react.useEffect)(()=>{
        fetchData();
        fetchPreData();
        fetchTableData();
    }, []);
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.container
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Grid.Row, {
        justify: "space-between",
        align: "center"
    }, _react.default.createElement(_webreact.Grid.Col, {
        span: 16
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['basicProfile.title.form'])), _react.default.createElement(_webreact.Grid.Col, {
        span: 8,
        style: {
            textAlign: 'right'
        }
    }, _react.default.createElement(_webreact.Space, null, _react.default.createElement(_webreact.Button, null, t['basicProfile.cancel']), _react.default.createElement(_webreact.Button, {
        type: "primary"
    }, t['basicProfile.goBack'])))), _react.default.createElement(_webreact.Steps, {
        current: data.status,
        lineless: true,
        className: _indexmoduleless.default.steps
    }, _react.default.createElement(_webreact.Steps.Step, {
        title: t['basicProfile.steps.commit']
    }), _react.default.createElement(_webreact.Steps.Step, {
        title: t['basicProfile.steps.approval']
    }), _react.default.createElement(_webreact.Steps.Step, {
        title: t['basicProfile.steps.finish']
    }))), _react.default.createElement(_item.default, {
        title: t['basicProfile.title.currentParams'],
        data: data,
        type: "current",
        loading: loading
    }), _react.default.createElement(_item.default, {
        title: t['basicProfile.title.originParams'],
        data: preData,
        type: "origin",
        loading: preLoading
    }), _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['basicProfile.adjustment.record']), _react.default.createElement(_webreact.Table, {
        loading: tableLoading,
        data: tableData,
        columns: [
            {
                dataIndex: 'contentId',
                title: t['basicProfile.adjustment.contentId']
            },
            {
                dataIndex: 'content',
                title: t['basicProfile.adjustment.content']
            },
            {
                dataIndex: 'status',
                title: t['basicProfile.adjustment.status'],
                render: (status)=>{
                    if (status) {
                        return _react.default.createElement(_webreact.Badge, {
                            status: "success",
                            text: t['basicProfile.adjustment.success']
                        });
                    }
                    return _react.default.createElement(_webreact.Badge, {
                        status: "processing",
                        text: t['basicProfile.adjustment.waiting']
                    });
                }
            },
            {
                dataIndex: 'updatedTime',
                title: t['basicProfile.adjustment.updatedTime']
            },
            {
                title: t['basicProfile.adjustment.operation'],
                headerCellStyle: {
                    paddingLeft: '15px'
                },
                render () {
                    return _react.default.createElement(_webreact.Button, {
                        type: "text"
                    }, t['basicProfile.adjustment.operation.view']);
                }
            }
        ]
    })));
}
const _default = BasicProfile;

},});