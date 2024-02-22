(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'data-analysis_index_3c9b.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"9ecadaa2": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _useChartTheme = /*#__PURE__*/ _interop_require_default._(farmRequire("7027c3d5"));
const _webreact = farmRequire("050d455e");
const _bizcharts = farmRequire("f36f3472");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _customertooltip = /*#__PURE__*/ _interop_require_default._(farmRequire("315f834c"));
const lineColor = [
    '#21CCFF',
    '#313CA9',
    '#249EFF'
];
function PeriodLine({ data, loading }) {
    return _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_bizcharts.Chart, {
        theme: (0, _useChartTheme.default)(),
        forceUpdate: true,
        height: 370,
        padding: [
            10,
            20,
            120,
            60
        ],
        data: data,
        autoFit: true,
        scale: {
            time: 'time'
        },
        className: 'chart-wrapper'
    }, _react.default.createElement(_bizcharts.Line, {
        shape: "smooth",
        position: "time*rate",
        color: [
            'name',
            lineColor
        ]
    }), _react.default.createElement(_bizcharts.Tooltip, {
        crosshairs: {
            type: 'x'
        },
        showCrosshairs: true,
        shared: true
    }, (title, items)=>{
        return _react.default.createElement(_customertooltip.default, {
            title: title,
            data: items
        });
    }), _react.default.createElement(_bizcharts.Axis, {
        name: "rate",
        label: {
            formatter (text) {
                return `${Number(text)} %`;
            }
        }
    }), _react.default.createElement(_bizcharts.Legend, {
        name: "name",
        marker: (_, index)=>{
            return {
                symbol: 'circle',
                style: {
                    fill: lineColor[index],
                    r: 4
                }
            };
        }
    }), _react.default.createElement(_bizcharts.Slider, {
        foregroundStyle: {
            borderRadius: ' 4px',
            fill: 'l (180) 0:rgba(206, 224, 255, 0.9) 1:rgba(146, 186, 255, 0.8)',
            opacity: 0.3
        },
        trendCfg: {
            data: data.map((item)=>item.rate),
            isArea: true,
            areaStyle: {
                fill: 'rgba(4, 135, 255, 0.15)',
                opacity: 1
            },
            backgroundStyle: {
                fill: '#F2F3F5'
            },
            lineStyle: {
                stroke: 'rgba(36, 158, 255, 0.3)',
                lineWidth: 2
            }
        },
        handlerStyle: {
            fill: '#ffffff',
            opacity: 1,
            width: 22,
            height: 22,
            stroke: '#165DFF'
        }
    })));
}
const _default = PeriodLine;

},
"f42f5e13": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("93aaeec9"));
const _card = /*#__PURE__*/ _interop_require_default._(farmRequire("f6cd7b46"));
const { Row, Col } = _webreact.Grid;
const cardInfo = [
    {
        key: 'visitor',
        type: 'line'
    },
    {
        key: 'content',
        type: 'interval'
    },
    {
        key: 'comment',
        type: 'line'
    },
    {
        key: 'share',
        type: 'pie'
    }
];
function PublicOpinion() {
    const t = (0, _useLocale.default)(_locale.default);
    const [loading, setLoading] = (0, _react.useState)(true);
    const [data, setData] = (0, _react.useState)(cardInfo.map((item)=>({
            ...item,
            chartType: item.type,
            title: t[`dataAnalysis.publicOpinion.${item.key}`]
        })));
    const getData = async ()=>{
        const requestList = cardInfo.map(async (info)=>{
            const { data } = await _axios.default.get(`/api/data-analysis/overview?type=${info.type}`).catch(()=>({
                    data: {}
                }));
            return {
                ...data,
                key: info.key,
                chartType: info.type
            };
        });
        const result = await Promise.all(requestList).finally(()=>setLoading(false));
        setData(result);
    };
    (0, _react.useEffect)(()=>{
        getData();
    }, []);
    const formatData = (0, _react.useMemo)(()=>{
        return data.map((item)=>({
                ...item,
                title: t[`dataAnalysis.publicOpinion.${item.key}`]
            }));
    }, [
        t,
        data
    ]);
    return _react.default.createElement("div", null, _react.default.createElement(Row, {
        gutter: 20
    }, formatData.map((item, index)=>_react.default.createElement(Col, {
            span: 6,
            key: index
        }, _react.default.createElement(_card.default, {
            ...item,
            compareTime: t['dataAnalysis.yesterday'],
            loading: loading
        })))));
}
const _default = PublicOpinion;

},});