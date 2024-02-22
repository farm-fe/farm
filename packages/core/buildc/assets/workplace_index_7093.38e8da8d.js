(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'workplace_index_7093.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"218f3f24": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _icon = farmRequire("f988cd7d");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("3eca280b"));
const _popularcontentsmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("982ae7d7"));
function PopularContent() {
    const t = (0, _useLocale.default)(_locale.default);
    const [type, setType] = (0, _react.useState)(0);
    const [data, setData] = (0, _react.useState)([]);
    const [loading, setLoading] = (0, _react.useState)(true);
    const [page, setPage] = (0, _react.useState)(1);
    const [total, setTotal] = (0, _react.useState)(0);
    const fetchData = (0, _react.useCallback)(()=>{
        setLoading(true);
        _axios.default.get(`/api/workplace/popular-contents?page=${page}&pageSize=5&category=${type}`).then((res)=>{
            setData(res.data.list);
            setTotal(res.data.total);
        }).finally(()=>{
            setLoading(false);
        });
    }, [
        page,
        type
    ]);
    (0, _react.useEffect)(()=>{
        fetchData();
    }, [
        page,
        fetchData
    ]);
    const columns = [
        {
            title: t['workplace.column.rank'],
            dataIndex: 'rank',
            width: 65
        },
        {
            title: t['workplace.column.title'],
            dataIndex: 'title',
            render: (x)=>_react.default.createElement(_webreact.Typography.Paragraph, {
                    style: {
                        margin: 0
                    },
                    ellipsis: true
                }, x)
        },
        {
            title: t['workplace.column.pv'],
            dataIndex: 'pv',
            width: 100,
            render: (text)=>{
                return `${text / 1000}k`;
            }
        },
        {
            title: t['workplace.column.increase'],
            dataIndex: 'increase',
            sorter: (a, b)=>a.increase - b.increase,
            width: 110,
            render: (text)=>{
                return _react.default.createElement("span", null, `${(text * 100).toFixed(2)}%`, _react.default.createElement("span", {
                    className: _popularcontentsmoduleless.default['symbol']
                }, text < 0 ? _react.default.createElement(_icon.IconCaretUp, {
                    style: {
                        color: 'rgb(var(--green-6))'
                    }
                }) : _react.default.createElement(_icon.IconCaretDown, {
                    style: {
                        color: 'rgb(var(--red-6))'
                    }
                })));
            }
        }
    ];
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement("div", {
        style: {
            display: 'flex',
            justifyContent: 'space-between'
        }
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['workplace.popularContents']), _react.default.createElement(_webreact.Link, null, t['workplace.seeMore'])), _react.default.createElement(_webreact.Radio.Group, {
        type: "button",
        value: type,
        onChange: setType,
        options: [
            {
                label: t['workplace.text'],
                value: 0
            },
            {
                label: t['workplace.image'],
                value: 1
            },
            {
                label: t['workplace.video'],
                value: 2
            }
        ],
        style: {
            marginBottom: 16
        }
    }), _react.default.createElement(_webreact.Table, {
        rowKey: "rank",
        columns: columns,
        data: data,
        loading: loading,
        tableLayoutFixed: true,
        onChange: (pagination)=>{
            setPage(pagination.current);
        },
        pagination: {
            total,
            current: page,
            pageSize: 5,
            simple: true
        }
    }));
}
const _default = PopularContent;

},
"59f5be78": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const SvgIncrease = (props)=>_react.createElement("svg", {
        width: 24,
        height: 27,
        viewBox: "0 0 24 27",
        fill: "none",
        xmlns: "http://www.w3.org/2000/svg",
        ...props
    }, _react.createElement("mask", {
        id: "mask0_178_29628",
        style: {
            maskType: "alpha"
        },
        maskUnits: "userSpaceOnUse",
        x: 1,
        y: 4,
        width: 21,
        height: 23
    }, _react.createElement("path", {
        d: "M20.1248 4.00061H3.87483C2.83929 4.00061 1.99983 4.84008 1.99983 5.87561V24.6256C1.99983 25.6611 2.83929 26.5006 3.87483 26.5006H20.1248C21.1604 26.5006 21.9998 25.6611 21.9998 24.6256V5.87561C21.9998 4.84008 21.1604 4.00061 20.1248 4.00061Z",
        fill: "white"
    })), _react.createElement("g", {
        mask: "url(#mask0_178_29628)"
    }, _react.createElement("g", {
        filter: "url(#filter0_ii_178_29628)"
    }, _react.createElement("path", {
        d: "M20.1248 4.00061H3.87483C2.83929 4.00061 1.99983 4.84008 1.99983 5.87561V23.6256C1.99983 24.6611 2.83929 25.5006 3.87483 25.5006H20.1248C21.1604 25.5006 21.9998 24.6611 21.9998 23.6256V5.87561C21.9998 4.84008 21.1604 4.00061 20.1248 4.00061Z",
        fill: "url(#paint0_linear_178_29628)"
    }))), _react.createElement("g", {
        filter: "url(#filter1_di_178_29628)"
    }, _react.createElement("path", {
        fillRule: "evenodd",
        clipRule: "evenodd",
        d: "M13.8747 1.87549C14.6534 1.87549 15.3213 2.35019 15.6046 3.02601C15.6294 3.08496 15.6858 3.12552 15.7497 3.12549C16.7853 3.12549 17.6247 3.96495 17.6247 5.00049C17.6247 6.03602 16.7853 6.87549 15.7497 6.87549H8.24974C7.2142 6.87549 6.37474 6.03602 6.37474 5.00049C6.37474 3.96495 7.2142 3.12549 8.24974 3.12549C8.31366 3.12552 8.37011 3.08496 8.39483 3.02601C8.67819 2.35019 9.34603 1.87549 10.1247 1.87549H13.8747Z",
        fill: "#FFFEFE"
    })), _react.createElement("path", {
        d: "M17.9719 9H6.02754V20.9444H17.9719V9Z",
        fill: "white",
        fillOpacity: 0.01
    }), _react.createElement("g", {
        filter: "url(#filter2_dii_178_29628)"
    }, _react.createElement("path", {
        fillRule: "evenodd",
        clipRule: "evenodd",
        d: "M12.5335 11.6822C12.5335 11.1511 12.9641 10.7206 13.4951 10.7206H17.0843C17.6154 10.7206 18.0459 11.1511 18.0459 11.6822V15.2715C18.0459 15.8025 17.6154 16.233 17.0843 16.233C16.5533 16.233 16.1227 15.8025 16.1227 15.2715V12.6438H13.4951C12.9641 12.6438 12.5335 12.2133 12.5335 11.6822Z",
        fill: "white"
    }), _react.createElement("path", {
        fillRule: "evenodd",
        clipRule: "evenodd",
        d: "M17.7488 10.9869C18.1327 11.3539 18.1464 11.9626 17.7795 12.3465L13.3481 16.9826C13.036 17.3092 12.5385 17.3744 12.1526 17.1393L9.93329 16.233L7.35673 18.8752C7.01826 19.2844 6.41212 19.3418 6.00289 19.0033C5.59365 18.6648 5.53629 18.0587 5.87476 17.6495L8.98186 14.3659C9.28641 13.9977 9.81516 13.9089 10.2232 14.1576L12.4926 15.0943L16.3892 11.0176C16.7562 10.6337 17.3649 10.62 17.7488 10.9869Z",
        fill: "white"
    })), _react.createElement("defs", null, _react.createElement("filter", {
        id: "filter0_ii_178_29628",
        x: 1.99983,
        y: 1.85775,
        width: 20,
        height: 24.7143,
        filterUnits: "userSpaceOnUse",
        colorInterpolationFilters: "sRGB"
    }, _react.createElement("feFlood", {
        floodOpacity: 0,
        result: "BackgroundImageFix"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in: "SourceGraphic",
        in2: "BackgroundImageFix",
        result: "shape"
    }), _react.createElement("feColorMatrix", {
        in: "SourceAlpha",
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0",
        result: "hardAlpha"
    }), _react.createElement("feOffset", {
        dy: -2.14286
    }), _react.createElement("feGaussianBlur", {
        stdDeviation: 1.07143
    }), _react.createElement("feComposite", {
        in2: "hardAlpha",
        operator: "arithmetic",
        k2: -1,
        k3: 1
    }), _react.createElement("feColorMatrix", {
        type: "matrix",
        values: "0 0 0 0 0.0788195 0 0 0 0 0.633708 0 0 0 0 0.945833 0 0 0 0.7 0"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in2: "shape",
        result: "effect1_innerShadow_178_29628"
    }), _react.createElement("feColorMatrix", {
        in: "SourceAlpha",
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0",
        result: "hardAlpha"
    }), _react.createElement("feOffset", {
        dy: 1.07143
    }), _react.createElement("feGaussianBlur", {
        stdDeviation: 1.07143
    }), _react.createElement("feComposite", {
        in2: "hardAlpha",
        operator: "arithmetic",
        k2: -1,
        k3: 1
    }), _react.createElement("feColorMatrix", {
        type: "matrix",
        values: "0 0 0 0 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0.7 0"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in2: "effect1_innerShadow_178_29628",
        result: "effect2_innerShadow_178_29628"
    })), _react.createElement("filter", {
        id: "filter1_di_178_29628",
        x: 4.23188,
        y: 0.452631,
        width: 15.5357,
        height: 9.28571,
        filterUnits: "userSpaceOnUse",
        colorInterpolationFilters: "sRGB"
    }, _react.createElement("feFlood", {
        floodOpacity: 0,
        result: "BackgroundImageFix"
    }), _react.createElement("feColorMatrix", {
        in: "SourceAlpha",
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0",
        result: "hardAlpha"
    }), _react.createElement("feOffset", {
        dy: 0.72
    }), _react.createElement("feGaussianBlur", {
        stdDeviation: 1.07143
    }), _react.createElement("feColorMatrix", {
        type: "matrix",
        values: "0 0 0 0 0.373715 0 0 0 0 0.67555 0 0 0 0 0.954167 0 0 0 0.6 0"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in2: "BackgroundImageFix",
        result: "effect1_dropShadow_178_29628"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in: "SourceGraphic",
        in2: "effect1_dropShadow_178_29628",
        result: "shape"
    }), _react.createElement("feColorMatrix", {
        in: "SourceAlpha",
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0",
        result: "hardAlpha"
    }), _react.createElement("feOffset", {
        dy: -0.72
    }), _react.createElement("feGaussianBlur", {
        stdDeviation: 1.07143
    }), _react.createElement("feComposite", {
        in2: "hardAlpha",
        operator: "arithmetic",
        k2: -1,
        k3: 1
    }), _react.createElement("feColorMatrix", {
        type: "matrix",
        values: "0 0 0 0 0.0178125 0 0 0 0 0.37905 0 0 0 0 0.7125 0 0 0 0.4 0"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in2: "shape",
        result: "effect2_innerShadow_178_29628"
    })), _react.createElement("filter", {
        id: "filter2_dii_178_29628",
        x: 1.32699,
        y: 9.27808,
        width: 21.0461,
        height: 17.1577,
        filterUnits: "userSpaceOnUse",
        colorInterpolationFilters: "sRGB"
    }, _react.createElement("feFlood", {
        floodOpacity: 0,
        result: "BackgroundImageFix"
    }), _react.createElement("feColorMatrix", {
        in: "SourceAlpha",
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0",
        result: "hardAlpha"
    }), _react.createElement("feOffset", {
        dy: 2.88476
    }), _react.createElement("feGaussianBlur", {
        stdDeviation: 2.16357
    }), _react.createElement("feColorMatrix", {
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0.355333 0 0 0 0 0.683333 0 0 0 0.5 0"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in2: "BackgroundImageFix",
        result: "effect1_dropShadow_178_29628"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in: "SourceGraphic",
        in2: "effect1_dropShadow_178_29628",
        result: "shape"
    }), _react.createElement("feColorMatrix", {
        in: "SourceAlpha",
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0",
        result: "hardAlpha"
    }), _react.createElement("feOffset", {
        dy: -1.1539
    }), _react.createElement("feGaussianBlur", {
        stdDeviation: 0.72119
    }), _react.createElement("feComposite", {
        in2: "hardAlpha",
        operator: "arithmetic",
        k2: -1,
        k3: 1
    }), _react.createElement("feColorMatrix", {
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0.58 0 0 0 0 1 0 0 0 0.4 0"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in2: "shape",
        result: "effect2_innerShadow_178_29628"
    }), _react.createElement("feColorMatrix", {
        in: "SourceAlpha",
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0",
        result: "hardAlpha"
    }), _react.createElement("feOffset", {
        dy: -0.582535
    }), _react.createElement("feGaussianBlur", {
        stdDeviation: 0.388357
    }), _react.createElement("feComposite", {
        in2: "hardAlpha",
        operator: "arithmetic",
        k2: -1,
        k3: 1
    }), _react.createElement("feColorMatrix", {
        type: "matrix",
        values: "0 0 0 0 0.879167 0 0 0 0 1 0 0 0 0 1 0 0 0 0.75 0"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in2: "effect2_innerShadow_178_29628",
        result: "effect3_innerShadow_178_29628"
    })), _react.createElement("linearGradient", {
        id: "paint0_linear_178_29628",
        x1: 31.2857,
        y1: -4.2138,
        x2: 0.7475,
        y2: 26.2897,
        gradientUnits: "userSpaceOnUse"
    }, _react.createElement("stop", {
        stopColor: "#24B5E3"
    }), _react.createElement("stop", {
        offset: 0.53305,
        stopColor: "#56CCFF"
    }), _react.createElement("stop", {
        offset: 1,
        stopColor: "#0BA7FF"
    }))));
const _default = SvgIncrease;

},
"82aa9da7": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _overviewarealine = /*#__PURE__*/ _interop_require_default._(farmRequire("deb5811d"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _calendarsvg = /*#__PURE__*/ _interop_require_default._(farmRequire("3bb81100"));
const _commentssvg = /*#__PURE__*/ _interop_require_default._(farmRequire("059746dc"));
const _contentsvg = /*#__PURE__*/ _interop_require_default._(farmRequire("cae3d379"));
const _increasesvg = /*#__PURE__*/ _interop_require_default._(farmRequire("59f5be78"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("3eca280b"));
const _overviewmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("0e2c31dd"));
const { Row, Col } = _webreact.Grid;
function StatisticItem(props) {
    const { icon, title, count, loading, unit } = props;
    return _react.default.createElement("div", {
        className: _overviewmoduleless.default.item
    }, _react.default.createElement("div", {
        className: _overviewmoduleless.default.icon
    }, icon), _react.default.createElement("div", null, _react.default.createElement(_webreact.Skeleton, {
        loading: loading,
        text: {
            rows: 2,
            width: 60
        },
        animation: true
    }, _react.default.createElement("div", {
        className: _overviewmoduleless.default.title
    }, title), _react.default.createElement("div", {
        className: _overviewmoduleless.default.count
    }, count, _react.default.createElement("span", {
        className: _overviewmoduleless.default.unit
    }, unit)))));
}
function Overview() {
    const [data, setData] = (0, _react.useState)({});
    const [loading, setLoading] = (0, _react.useState)(true);
    const t = (0, _useLocale.default)(_locale.default);
    const userInfo = (0, _reactredux.useSelector)((state)=>state.userInfo || {});
    const fetchData = ()=>{
        setLoading(true);
        _axios.default.get('/api/workplace/overview-content').then((res)=>{
            setData(res.data);
        }).finally(()=>{
            setLoading(false);
        });
    };
    (0, _react.useEffect)(()=>{
        fetchData();
    }, []);
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 5
    }, t['workplace.welcomeBack'], userInfo.name), _react.default.createElement(_webreact.Divider, null), _react.default.createElement(Row, null, _react.default.createElement(Col, {
        flex: 1
    }, _react.default.createElement(StatisticItem, {
        icon: _react.default.createElement(_calendarsvg.default, null),
        title: t['workplace.totalOnlyData'],
        count: data.allContents,
        loading: loading,
        unit: t['workplace.pecs']
    })), _react.default.createElement(_webreact.Divider, {
        type: "vertical",
        className: _overviewmoduleless.default.divider
    }), _react.default.createElement(Col, {
        flex: 1
    }, _react.default.createElement(StatisticItem, {
        icon: _react.default.createElement(_contentsvg.default, null),
        title: t['workplace.contentInMarket'],
        count: data.liveContents,
        loading: loading,
        unit: t['workplace.pecs']
    })), _react.default.createElement(_webreact.Divider, {
        type: "vertical",
        className: _overviewmoduleless.default.divider
    }), _react.default.createElement(Col, {
        flex: 1
    }, _react.default.createElement(StatisticItem, {
        icon: _react.default.createElement(_commentssvg.default, null),
        title: t['workplace.comments'],
        count: data.increaseComments,
        loading: loading,
        unit: t['workplace.pecs']
    })), _react.default.createElement(_webreact.Divider, {
        type: "vertical",
        className: _overviewmoduleless.default.divider
    }), _react.default.createElement(Col, {
        flex: 1
    }, _react.default.createElement(StatisticItem, {
        icon: _react.default.createElement(_increasesvg.default, null),
        title: t['workplace.growth'],
        count: _react.default.createElement("span", null, data.growthRate, " ", _react.default.createElement(_icon.IconCaretUp, {
            style: {
                fontSize: 18,
                color: 'rgb(var(--green-6))'
            }
        })),
        loading: loading
    }))), _react.default.createElement(_webreact.Divider, null), _react.default.createElement("div", null, _react.default.createElement("div", {
        className: _overviewmoduleless.default.ctw
    }, _react.default.createElement(_webreact.Typography.Paragraph, {
        className: _overviewmoduleless.default['chart-title'],
        style: {
            marginBottom: 0
        }
    }, t['workplace.contentData'], _react.default.createElement("span", {
        className: _overviewmoduleless.default['chart-sub-title']
    }, "(", t['workplace.1year'], ")")), _react.default.createElement(_webreact.Link, null, t['workplace.seeMore'])), _react.default.createElement(_overviewarealine.default, {
        data: data.chartData,
        loading: loading
    })));
}
const _default = Overview;

},});