(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'multi-dimension-data-analysis_index_d577.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"3887c6f3": function(module, exports, farmRequire, farmDynamicRequire) {
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
function HorizontalInterval({ data, loading, height }) {
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
            const radius = (path[1][2] - path[2][2]) / 2;
            group.addShape('rect', {
                attrs: {
                    x: path[0][1],
                    y: path[0][2] - radius * 2,
                    width: path[1][1] - path[0][1],
                    height: path[1][2] - path[2][2],
                    fill: cfg.color,
                    radius: radius
                }
            });
            return group;
        }
    });
    return _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_bizcharts.Chart, {
        height: height || 370,
        padding: "auto",
        data: data,
        autoFit: true,
        className: 'chart-wrapper'
    }, _react.default.createElement(_bizcharts.Coordinate, {
        transpose: true
    }), _react.default.createElement(_bizcharts.Interval, {
        color: "#4086FF",
        position: "name*count",
        size: 10,
        shape: "border-radius"
    }), _react.default.createElement(_bizcharts.Tooltip, null, (title, items)=>{
        return _react.default.createElement(_customertooltip.default, {
            title: title,
            data: items
        });
    }), _react.default.createElement(_bizcharts.Axis, {
        name: "count",
        label: {
            formatter (text) {
                return `${Number(text) / 1000}k`;
            }
        }
    })));
}
const _default = HorizontalInterval;

},
"ac636b01": function(module, exports, farmRequire, farmDynamicRequire) {
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
const i18n = {
    "en-US": {
        "menu.visualization": "Data Visualization",
        "menu.visualization.multiDimensionDataAnalysis": "Multi-D Analysis",
        "multiDAnalysis.card.title.activeContributors": "Active Contributors",
        "multiDAnalysis.unit": "times",
        "multiDAnalysis.card.title.officeVisitors": "Office Visitors",
        "multiDAnalysis.card.title.downloads": "Downloads",
        "multiDAnalysis.card.title.dataOverview": "Overview",
        "multiDAnalysis.card.title.todayActivity": "Today's Likes and Comments Statistics",
        "multiDAnalysis.card.title.contentTheme": "Content theme distribution",
        "multiDAnalysis.card.title.contentSource": "Content publishing source",
        "multiDAnalysis.dataOverview.contentProduction": "Content production",
        "multiDAnalysis.dataOverview.contentClicks": "Content clicks",
        "multiDAnalysis.dataOverview.contextExposure": "Content exposure",
        "multiDAnalysis.dataOverview.activeUsers": "Active users",
        "multiDAnalysis.cardList.userRetentionTrend": "User retention trends",
        "multiDAnalysis.cardList.userRetention": "User retention",
        "multiDAnalysis.cardList.contentConsumptionTrend": "Content consumption trends",
        "multiDAnalysis.cardList.contentConsumption": "Content consumption"
    },
    "zh-CN": {
        "menu.visualization": "数据可视化",
        "menu.visualization.multiDimensionDataAnalysis": "多维数据分析",
        "multiDAnalysis.card.title.officeVisitors": "官网访问量",
        "multiDAnalysis.card.title.downloads": "下载量",
        "multiDAnalysis.card.title.dataOverview": "数据总览",
        "multiDAnalysis.card.title.todayActivity": "今日转赞评统计",
        "multiDAnalysis.card.title.contentTheme": "内容题材分布",
        "multiDAnalysis.card.title.contentSource": "内容发布来源",
        "multiDAnalysis.dataOverview.contentProduction": "内容生产量",
        "multiDAnalysis.dataOverview.contentClicks": "内容点击量",
        "multiDAnalysis.dataOverview.contextExposure": "内容曝光量",
        "multiDAnalysis.dataOverview.activeUsers": "活跃用户数",
        "multiDAnalysis.cardList.userRetentionTrend": "用户留存趋势",
        "multiDAnalysis.cardList.userRetention": "用户留存量",
        "multiDAnalysis.cardList.contentConsumptionTrend": "内容消费趋势",
        "multiDAnalysis.cardList.contentConsumption": "内容消费量"
    }
};
const _default = i18n;

},
"bc56371e": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _dataset = /*#__PURE__*/ _interop_require_default._(farmRequire("c4ed6075"));
const _webreact = farmRequire("050d455e");
const _bizcharts = farmRequire("f36f3472");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _customertooltip = /*#__PURE__*/ _interop_require_default._(farmRequire("315f834c"));
function AreaPolar(props) {
    const { data, loading, fields, height } = props;
    const { DataView } = _dataset.default;
    const dv = new DataView().source(data);
    dv.transform({
        type: 'fold',
        fields: fields,
        key: 'category',
        value: 'score'
    });
    return _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_bizcharts.Chart, {
        height: height || 400,
        padding: 0,
        data: dv.rows,
        autoFit: true,
        scale: {
            score: {
                min: 0,
                max: 80
            }
        },
        interactions: [
            'legend-highlight'
        ],
        className: 'chart-wrapper'
    }, _react.default.createElement(_bizcharts.Coordinate, {
        type: "polar",
        radius: 0.8
    }), _react.default.createElement(_bizcharts.Tooltip, {
        shared: true
    }, (title, items)=>{
        return _react.default.createElement(_customertooltip.default, {
            title: title,
            data: items
        });
    }), _react.default.createElement(_bizcharts.Line, {
        position: "item*score",
        size: "2",
        color: [
            'category',
            [
                '#313CA9',
                '#21CCFF',
                '#249EFF'
            ]
        ]
    }), _react.default.createElement(_bizcharts.Area, {
        position: "item*score",
        tooltip: false,
        color: [
            'category',
            [
                'rgba(49, 60, 169, 0.4)',
                'rgba(33, 204, 255, 0.4)',
                'rgba(36, 158, 255, 0.4)'
            ]
        ]
    }), _react.default.createElement(_bizcharts.Axis, {
        name: "score",
        label: false
    }), _react.default.createElement(_bizcharts.Legend, {
        position: "right",
        marker: (_, index)=>{
            return {
                symbol: 'circle',
                style: {
                    r: 4,
                    lineWidth: 0,
                    fill: [
                        '#313CA9',
                        '#21CCFF',
                        '#249EFF'
                    ][index]
                }
            };
        },
        name: "category"
    })));
}
const _default = AreaPolar;

},});