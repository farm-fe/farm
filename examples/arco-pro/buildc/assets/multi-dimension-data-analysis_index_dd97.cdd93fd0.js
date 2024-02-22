(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'multi-dimension-data-analysis_index_dd97.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"6570b5fa": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _areapolar = /*#__PURE__*/ _interop_require_default._(farmRequire("bc56371e"));
const _factmultipie = /*#__PURE__*/ _interop_require_default._(farmRequire("27b85efb"));
const _horizontalinterval = /*#__PURE__*/ _interop_require_default._(farmRequire("3887c6f3"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _cardlist = /*#__PURE__*/ _interop_require_default._(farmRequire("d2edbfcf"));
const _dataoverview = /*#__PURE__*/ _interop_require_default._(farmRequire("c2c12909"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("ac636b01"));
farmRequire("f1d225f2");
const { Row, Col } = _webreact.Grid;
const { Title } = _webreact.Typography;
function DataAnalysis() {
    const t = (0, _useLocale.default)(_locale.default);
    const [loading, setLoading] = (0, _react.useState)(false);
    const [interval, setInterval] = (0, _react.useState)([]);
    const [polarLoading, setPolarLoading] = (0, _react.useState)(false);
    const [polar, setPolar] = (0, _react.useState)({
        list: [],
        fields: []
    });
    const [multiPieLoading, setMultiPieLoading] = (0, _react.useState)(false);
    const [multiPie, setMultiPie] = (0, _react.useState)([]);
    const getInterval = async ()=>{
        setLoading(true);
        const { data } = await _axios.default.get('/api/multi-dimension/activity').finally(()=>{
            setLoading(false);
        });
        setInterval(data);
    };
    const getPolar = async ()=>{
        setPolarLoading(true);
        const { data } = await _axios.default.get('/api/multi-dimension/polar').finally(()=>setPolarLoading(false));
        setPolar(data);
    };
    const getMultiPie = async ()=>{
        setMultiPieLoading(true);
        const { data } = await _axios.default.get('/api/multi-dimension/content-source').finally(()=>{
            setMultiPieLoading(false);
        });
        setMultiPie(data);
    };
    (0, _react.useEffect)(()=>{
        getInterval();
        getPolar();
        getMultiPie();
    }, []);
    return _react.default.createElement(_webreact.Space, {
        size: 16,
        direction: "vertical",
        style: {
            width: '100%'
        }
    }, _react.default.createElement(Row, {
        gutter: 20
    }, _react.default.createElement(Col, {
        span: 16
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(Title, {
        heading: 6
    }, t['multiDAnalysis.card.title.dataOverview']), _react.default.createElement(_dataoverview.default, null))), _react.default.createElement(Col, {
        span: 8
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(Title, {
        heading: 6
    }, t['multiDAnalysis.card.title.todayActivity']), _react.default.createElement(_horizontalinterval.default, {
        data: interval,
        loading: loading,
        height: 160
    })), _react.default.createElement(_webreact.Card, null, _react.default.createElement(Title, {
        heading: 6
    }, t['multiDAnalysis.card.title.contentTheme']), _react.default.createElement(_areapolar.default, {
        data: polar.list,
        fields: polar.fields,
        height: 197,
        loading: polarLoading
    })))), _react.default.createElement(Row, null, _react.default.createElement(Col, {
        span: 24
    }, _react.default.createElement(_cardlist.default, null))), _react.default.createElement(Row, null, _react.default.createElement(Col, {
        span: 24
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(Title, {
        heading: 6
    }, t['multiDAnalysis.card.title.contentSource']), _react.default.createElement(_factmultipie.default, {
        loading: multiPieLoading,
        data: multiPie,
        height: 240
    })))));
}
const _default = DataAnalysis;

},
"d2edbfcf": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _bizcharts = farmRequire("f36f3472");
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("ac636b01"));
const _icon = farmRequire("f988cd7d");
const _cardblockmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("744cadd6"));
const { Row, Col } = _webreact.Grid;
const { Title, Text } = _webreact.Typography;
const basicChartProps = {
    pure: true,
    autoFit: true,
    height: 80,
    padding: [
        0,
        10,
        0,
        10
    ]
};
function CustomTooltip(props) {
    const { items } = props;
    return _react.default.createElement("div", {
        className: _cardblockmoduleless.default.tooltip
    }, items.map((item, index)=>_react.default.createElement("div", {
            key: index
        }, _react.default.createElement(Text, {
            bold: true
        }, Number(item.data.y).toLocaleString()))));
}
function SimpleLine(props) {
    const { chartData } = props;
    return _react.default.createElement(_bizcharts.Chart, {
        data: chartData,
        ...basicChartProps
    }, _react.default.createElement(_bizcharts.Line, {
        position: "x*y",
        shape: [
            'name',
            [
                'smooth',
                'dash'
            ]
        ],
        color: [
            'name',
            [
                '#165DFF',
                'rgba(106,161,255,0.3)'
            ]
        ]
    }), _react.default.createElement(_bizcharts.Tooltip, {
        shared: false,
        showCrosshairs: true
    }, (_, items)=>_react.default.createElement(CustomTooltip, {
            items: items
        })));
}
function SimpleInterval(props) {
    const { chartData } = props;
    return _react.default.createElement(_bizcharts.Chart, {
        data: chartData,
        ...basicChartProps
    }, _react.default.createElement(_bizcharts.Interval, {
        position: "x*y",
        color: [
            'x',
            (xVal)=>{
                if (Number(xVal) % 2 === 0) {
                    return '#86DF6C';
                }
                return '#468DFF';
            }
        ]
    }), _react.default.createElement(_bizcharts.Tooltip, {
        shared: false
    }, (_, items)=>_react.default.createElement(CustomTooltip, {
            items: items
        })), _react.default.createElement(_bizcharts.Interaction, {
        type: "active-region"
    }));
}
function CardBlock(props) {
    const { chartType, title, count, increment, diff, chartData, loading } = props;
    return _react.default.createElement(_webreact.Card, {
        className: _cardblockmoduleless.default.card
    }, _react.default.createElement("div", {
        className: _cardblockmoduleless.default.statistic
    }, _react.default.createElement(_webreact.Statistic, {
        title: _react.default.createElement(Title, {
            heading: 6,
            className: _cardblockmoduleless.default.title
        }, title),
        loading: loading,
        value: count,
        extra: _react.default.createElement("div", {
            className: _cardblockmoduleless.default['compare-yesterday']
        }, loading ? _react.default.createElement(_webreact.Skeleton, {
            text: {
                rows: 1
            },
            style: {
                width: '100px'
            },
            animation: true
        }) : _react.default.createElement("span", {
            className: (0, _classnames.default)(_cardblockmoduleless.default['diff'], {
                [_cardblockmoduleless.default['diff-increment']]: increment
            })
        }, diff, increment ? _react.default.createElement(_icon.IconArrowRise, null) : _react.default.createElement(_icon.IconArrowFall, null))),
        groupSeparator: true
    })), _react.default.createElement("div", {
        className: _cardblockmoduleless.default.chart
    }, _react.default.createElement(_webreact.Spin, {
        style: {
            width: '100%'
        },
        loading: loading
    }, chartType === 'interval' && _react.default.createElement(SimpleInterval, {
        chartData: chartData
    }), chartType === 'line' && _react.default.createElement(SimpleLine, {
        chartData: chartData
    }))));
}
const cardInfo = [
    {
        key: 'userRetentionTrend',
        type: 'line'
    },
    {
        key: 'userRetention',
        type: 'interval'
    },
    {
        key: 'contentConsumptionTrend',
        type: 'line'
    },
    {
        key: 'contentConsumption',
        type: 'interval'
    }
];
function CardList() {
    const t = (0, _useLocale.default)(_locale.default);
    const [loading, setLoading] = (0, _react.useState)(false);
    const [data, setData] = (0, _react.useState)(cardInfo.map((item)=>({
            ...item,
            chartType: item.type
        })));
    const getData = async ()=>{
        const requestList = cardInfo.map(async (info)=>{
            const { data } = await _axios.default.get(`/api/multi-dimension/card?type=${info.type}`).catch(()=>({
                    data: {}
                }));
            return {
                ...data,
                key: info.key,
                chartType: info.type
            };
        });
        setLoading(true);
        const result = await Promise.all(requestList).finally(()=>setLoading(false));
        setData(result);
    };
    (0, _react.useEffect)(()=>{
        getData();
    }, []);
    const formatData = (0, _react.useMemo)(()=>{
        return data.map((item)=>({
                ...item,
                title: t[`multiDAnalysis.cardList.${item.key}`]
            }));
    }, [
        t,
        data
    ]);
    return _react.default.createElement(Row, {
        gutter: 16
    }, formatData.map((item, index)=>_react.default.createElement(Col, {
            span: 6,
            key: index
        }, _react.default.createElement(CardBlock, {
            ...item,
            loading: loading
        }))));
}
const _default = CardList;

},});