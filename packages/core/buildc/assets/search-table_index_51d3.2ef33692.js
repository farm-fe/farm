(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'search-table_index_51d3.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"542b0a82": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    ContentType: function() {
        return ContentType;
    },
    FilterType: function() {
        return FilterType;
    },
    Status: function() {
        return Status;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _PermissionWrapper = /*#__PURE__*/ _interop_require_default._(farmRequire("60420157"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _constants = farmRequire("7ede7dda");
const _form = /*#__PURE__*/ _interop_require_default._(farmRequire("1e7b1dfb"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("4a7b2bdc"));
farmRequire("f03587c7");
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("1564d60e"));
const { Title } = _webreact.Typography;
const ContentType = [
    '图文',
    '横版短视频',
    '竖版短视频'
];
const FilterType = [
    '规则筛选',
    '人工'
];
const Status = [
    '已上线',
    '未上线'
];
function SearchTable() {
    const t = (0, _useLocale.default)(_locale.default);
    const tableCallback = async (record, type)=>{
        console.log(record, type);
    };
    const columns = (0, _react.useMemo)(()=>(0, _constants.getColumns)(t, tableCallback), [
        t
    ]);
    const [data, setData] = (0, _react.useState)([]);
    const [pagination, setPatination] = (0, _react.useState)({
        sizeCanChange: true,
        showTotal: true,
        pageSize: 10,
        current: 1,
        pageSizeChangeResetCurrent: true
    });
    const [loading, setLoading] = (0, _react.useState)(true);
    const [formParams, setFormParams] = (0, _react.useState)({});
    (0, _react.useEffect)(()=>{
        fetchData();
    }, [
        pagination.current,
        pagination.pageSize,
        JSON.stringify(formParams)
    ]);
    function fetchData() {
        const { current, pageSize } = pagination;
        setLoading(true);
        _axios.default.get('/api/list', {
            params: {
                page: current,
                pageSize,
                ...formParams
            }
        }).then((res)=>{
            setData(res.data.list);
            setPatination({
                ...pagination,
                current,
                pageSize,
                total: res.data.total
            });
            setLoading(false);
        });
    }
    function onChangeTable(pagination) {
        setPatination(pagination);
    }
    function handleSearch(params) {
        setFormParams(params);
    }
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(Title, {
        heading: 6
    }, t['menu.list.searchTable']), _react.default.createElement(_form.default, {
        onSearch: handleSearch
    }), _react.default.createElement(_PermissionWrapper.default, {
        requiredPermissions: [
            {
                resource: 'menu.list.searchTable',
                actions: [
                    'write'
                ]
            }
        ]
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['button-group']
    }, _react.default.createElement(_webreact.Space, null, _react.default.createElement(_webreact.Button, {
        type: "primary",
        icon: _react.default.createElement(_icon.IconPlus, null)
    }, t['searchTable.operations.add']), _react.default.createElement(_webreact.Button, null, t['searchTable.operations.upload'])), _react.default.createElement(_webreact.Space, null, _react.default.createElement(_webreact.Button, {
        icon: _react.default.createElement(_icon.IconDownload, null)
    }, t['searchTable.operation.download'])))), _react.default.createElement(_webreact.Table, {
        rowKey: "id",
        loading: loading,
        onChange: onChangeTable,
        pagination: pagination,
        columns: columns,
        data: data
    }));
}
const _default = SearchTable;

},
"7ede7dda": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    ContentType: function() {
        return ContentType;
    },
    FilterType: function() {
        return FilterType;
    },
    Status: function() {
        return Status;
    },
    getColumns: function() {
        return getColumns;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _webreact = farmRequire("050d455e");
const _dayjs = /*#__PURE__*/ _interop_require_default._(farmRequire("d0dc4dad"));
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _horizontalsvg = /*#__PURE__*/ _interop_require_default._(farmRequire("7284b3c9"));
const _textsvg = /*#__PURE__*/ _interop_require_default._(farmRequire("2d871be2"));
const _verticalsvg = /*#__PURE__*/ _interop_require_default._(farmRequire("2b1be6bc"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("1564d60e"));
const { Text } = _webreact.Typography;
const ContentType = [
    '图文',
    '横版短视频',
    '竖版短视频'
];
const FilterType = [
    '规则筛选',
    '人工'
];
const Status = [
    '未上线',
    '已上线'
];
const ContentIcon = [
    _react.default.createElement(_textsvg.default, {
        key: 0
    }),
    _react.default.createElement(_horizontalsvg.default, {
        key: 1
    }),
    _react.default.createElement(_verticalsvg.default, {
        key: 2
    })
];
function getColumns(t, callback) {
    return [
        {
            title: t['searchTable.columns.id'],
            dataIndex: 'id',
            render: (value)=>_react.default.createElement(Text, {
                    copyable: true
                }, value)
        },
        {
            title: t['searchTable.columns.name'],
            dataIndex: 'name'
        },
        {
            title: t['searchTable.columns.contentType'],
            dataIndex: 'contentType',
            render: (value)=>_react.default.createElement("div", {
                    className: _indexmoduleless.default['content-type']
                }, ContentIcon[value], ContentType[value])
        },
        {
            title: t['searchTable.columns.filterType'],
            dataIndex: 'filterType',
            render: (value)=>FilterType[value]
        },
        {
            title: t['searchTable.columns.contentNum'],
            dataIndex: 'count',
            sorter: (a, b)=>a.count - b.count,
            render (x) {
                return Number(x).toLocaleString();
            }
        },
        {
            title: t['searchTable.columns.createdTime'],
            dataIndex: 'createdTime',
            render: (x)=>(0, _dayjs.default)().subtract(x, 'days').format('YYYY-MM-DD HH:mm:ss'),
            sorter: (a, b)=>b.createdTime - a.createdTime
        },
        {
            title: t['searchTable.columns.status'],
            dataIndex: 'status',
            render: (x)=>{
                if (x === 0) {
                    return _react.default.createElement(_webreact.Badge, {
                        status: "error",
                        text: Status[x]
                    });
                }
                return _react.default.createElement(_webreact.Badge, {
                    status: "success",
                    text: Status[x]
                });
            }
        },
        {
            title: t['searchTable.columns.operations'],
            dataIndex: 'operations',
            headerCellStyle: {
                paddingLeft: '15px'
            },
            render: (_, record)=>_react.default.createElement(_webreact.Button, {
                    type: "text",
                    size: "small",
                    onClick: ()=>callback(record, 'view')
                }, t['searchTable.columns.operations.view'])
        }
    ];
}

},});