(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'data-analysis_index_f891.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"f6cd7b46": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _icon = farmRequire("f988cd7d");
const _publicopinionmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("85bf8767"));
const { Title, Text } = _webreact.Typography;
const basicChartProps = {
    pure: true,
    autoFit: true,
    height: 80,
    padding: [
        10,
        10,
        0,
        10
    ]
};
function SimpleLine(props) {
    const { chartData } = props;
    return _react.default.createElement(_bizcharts.Chart, {
        data: chartData,
        ...basicChartProps
    }, _react.default.createElement(_bizcharts.Line, {
        position: "x*y",
        size: 3,
        shape: 'smooth',
        color: [
            'name',
            [
                '#165DFF',
                'rgba(106,161,255,0.3)'
            ]
        ],
        style: {
            fields: [
                'name'
            ],
            callback: (name)=>{
                if (name === '类目2') {
                    return {
                        lineDash: [
                            8,
                            10
                        ]
                    };
                }
                return {};
            }
        }
    }));
}
function SimpleInterval(props) {
    const { chartData } = props;
    _bizcharts.G2.registerShape('interval', 'border-radius', {
        draw (cfg, container) {
            const points = cfg.points;
            let path = [];
            path.push([
                'M',
                points[0].x,
                points[0].y
            ]);
            path.push([
                'L',
                points[1].x,
                points[1].y
            ]);
            path.push([
                'L',
                points[2].x,
                points[2].y
            ]);
            path.push([
                'L',
                points[3].x,
                points[3].y
            ]);
            path.push('Z');
            path = this.parsePath(path) // 将 0 - 1 转化为画布坐标
            ;
            const group = container.addGroup();
            group.addShape('rect', {
                attrs: {
                    x: path[1][1],
                    y: path[1][2],
                    width: path[2][1] - path[1][1],
                    height: path[0][2] - path[1][2],
                    fill: cfg.color,
                    radius: (path[2][1] - path[1][1]) / 2
                }
            });
            return group;
        }
    });
    return _react.default.createElement(_bizcharts.Chart, {
        data: chartData,
        ...basicChartProps
    }, _react.default.createElement(_bizcharts.Interval, {
        position: "x*y",
        color: [
            'x',
            (xVal)=>{
                if (Number(xVal) % 2 === 0) {
                    return '#2CAB40';
                }
                return '#86DF6C';
            }
        ],
        shape: "border-radius"
    }));
}
function SimplePie(props) {
    const { chartData } = props;
    return _react.default.createElement(_bizcharts.Chart, {
        data: chartData,
        ...basicChartProps,
        padding: [
            0,
            20,
            0,
            0
        ]
    }, _react.default.createElement(_bizcharts.Coordinate, {
        type: "theta",
        radius: 0.8,
        innerRadius: 0.7
    }), _react.default.createElement(_bizcharts.Interval, {
        adjust: "stack",
        position: "count",
        shape: "sliceShape",
        color: [
            'name',
            [
                '#8D4EDA',
                '#00B2FF',
                '#165DFF'
            ]
        ],
        label: false
    }), _react.default.createElement(_bizcharts.Tooltip, {
        visible: true
    }), _react.default.createElement(_bizcharts.Legend, {
        position: "right"
    }), _react.default.createElement(_bizcharts.Interaction, {
        type: "element-single-selected"
    }));
}
function PublicOpinionCard(props) {
    const { chartType, title, count, increment, diff, chartData, loading } = props;
    const className = (0, _classnames.default)(_publicopinionmoduleless.default.card, _publicopinionmoduleless.default[`card-${chartType}`]);
    return _react.default.createElement("div", {
        className: className
    }, _react.default.createElement("div", {
        className: _publicopinionmoduleless.default.statistic
    }, _react.default.createElement(_webreact.Statistic, {
        title: _react.default.createElement(Title, {
            heading: 6,
            className: _publicopinionmoduleless.default.title
        }, title),
        loading: loading,
        value: count,
        groupSeparator: true
    }), _react.default.createElement("div", {
        className: _publicopinionmoduleless.default['compare-yesterday']
    }, _react.default.createElement(Text, {
        type: "secondary",
        className: _publicopinionmoduleless.default['compare-yesterday-text']
    }, props.compareTime), _react.default.createElement("span", {
        className: (0, _classnames.default)(_publicopinionmoduleless.default['diff'], {
            [_publicopinionmoduleless.default['diff-increment']]: increment
        })
    }, loading ? _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 1
        },
        animation: true
    }) : _react.default.createElement(_react.default.Fragment, null, diff, increment ? _react.default.createElement(_icon.IconArrowRise, null) : _react.default.createElement(_icon.IconArrowFall, null))))), _react.default.createElement("div", {
        className: _publicopinionmoduleless.default.chart
    }, loading ? _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 3,
            width: Array(3).fill('100%')
        },
        animation: true
    }) : _react.default.createElement(_react.default.Fragment, null, chartType === 'interval' && _react.default.createElement(SimpleInterval, {
        chartData: chartData
    }), chartType === 'line' && _react.default.createElement(SimpleLine, {
        chartData: chartData
    }), chartType === 'pie' && _react.default.createElement(SimplePie, {
        chartData: chartData
    }))));
}
const _default = PublicOpinionCard;

},});