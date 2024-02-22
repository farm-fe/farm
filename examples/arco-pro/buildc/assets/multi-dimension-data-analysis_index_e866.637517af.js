(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'multi-dimension-data-analysis_index_e866.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"19c2911b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _bizcharts = farmRequire("f36f3472");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _customertooltip = /*#__PURE__*/ _interop_require_default._(farmRequire("315f834c"));
const areaColorMap = [
    'l (90) 0:rgba(131, 100, 255, 0.5) 1:rgba(80, 52, 255, 0.001)',
    'l (90) 0:rgba(100, 255, 236, 0.5) 1:rgba(52, 255, 243, 0.001)',
    'l (90) 0:rgba(255, 211, 100, 0.5) 1:rgba(255, 235, 52, 0.001)',
    'l (90) 0:rgba(100, 162, 255, 0.5) 1:rgba(52, 105, 255, 0.001)'
];
const lineColorMap = [
    '#722ED1',
    '#33D1C9',
    '#F77234',
    '#165DFF'
];
function MultiAreaLine({ data, loading }) {
    return _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_bizcharts.Chart, {
        height: 352,
        data: data,
        padding: [
            10,
            0,
            30,
            30
        ],
        autoFit: true,
        scale: {
            time: 'time'
        },
        className: 'chart-wrapper'
    }, _react.default.createElement(_bizcharts.Line, {
        shape: "smooth",
        position: "time*count",
        color: [
            'name',
            lineColorMap
        ]
    }), _react.default.createElement(_bizcharts.Area, {
        position: "time*count",
        shape: "smooth",
        color: [
            'name',
            areaColorMap
        ],
        tooltip: false
    }), _react.default.createElement(_bizcharts.Tooltip, {
        crosshairs: {
            type: 'x'
        },
        showCrosshairs: true,
        shared: true,
        showMarkers: true
    }, (title, items)=>{
        return _react.default.createElement(_customertooltip.default, {
            title: title,
            data: items.sort((a, b)=>b.value - a.value),
            formatter: (value)=>Number(value).toLocaleString()
        });
    }), _react.default.createElement(_bizcharts.Axis, {
        name: "count",
        label: {
            formatter: (value)=>`${Number(value) / 100} k`
        }
    }), _react.default.createElement(_bizcharts.Legend, {
        visible: false
    })));
}
const _default = MultiAreaLine;

},
"27b85efb": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _bizcharts = farmRequire("f36f3472");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
function FactMultiPie(props) {
    return _react.default.createElement(_bizcharts.Chart, {
        theme: (0, _useChartTheme.default)(),
        forceUpdate: true,
        autoFit: true,
        data: props.data,
        height: props.height || 400,
        padding: [
            0,
            0,
            10,
            0
        ]
    }, _react.default.createElement(_bizcharts.Legend, {
        visible: true
    }), _react.default.createElement(_bizcharts.Facet, {
        fields: [
            'category'
        ],
        type: "rect",
        showTitle: false,
        eachView: (view, facet)=>{
            const data = facet.data;
            view.coordinate({
                type: 'theta',
                cfg: {
                    radius: 0.8,
                    innerRadius: 0.7
                }
            });
            view.interval().adjust('stack').position('value').color('type', [
                '#249eff',
                '#846BCE',
                '#21CCFF',
                ' #86DF6C',
                '#0E42D2'
            ]).label('value', {
                content: (content)=>{
                    return `${(content.value * 100).toFixed(2)} %`;
                }
            }), view.annotation().text({
                position: [
                    '50%',
                    '46%'
                ],
                content: data[0].category,
                style: {
                    fontSize: 14,
                    fontWeight: 500,
                    textAlign: 'center'
                },
                offsetY: 10
            });
            view.interaction('element-single-selected');
        }
    }));
}
const _default = FactMultiPie;

},
"67d39136": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "card": `card-5131f84a`,
    "content": `content-5131f84a`,
    "content-icon": `content-icon-5131f84a`,
    "skeleton": `skeleton-5131f84a`
};

},
"744cadd6": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "card": `card-faf72d71`,
    "diff": `diff-faf72d71`,
    "diff-increment": `diff-increment-faf72d71`,
    "statistic": `statistic-faf72d71`,
    "title": `title-faf72d71`,
    "tooltip": `tooltip-faf72d71`
};

},});