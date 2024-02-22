(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'data-analysis_index_8e4f.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"85bf8767": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "card": `card-e4dcdeb8`,
    "card-interval": `card-interval-e4dcdeb8`,
    "card-line": `card-line-e4dcdeb8`,
    "card-pie": `card-pie-e4dcdeb8`,
    "chart": `chart-e4dcdeb8`,
    "compare-yesterday-text": `compare-yesterday-text-e4dcdeb8`,
    "diff": `diff-e4dcdeb8`,
    "diff-increment": `diff-increment-e4dcdeb8`,
    "statistic": `statistic-e4dcdeb8`,
    "title": `title-e4dcdeb8`
};

},
"93aaeec9": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "menu.visualization.analysis": "Analysis",
        "dataAnalysis.yesterday": "Yesterday",
        "dataAnalysis.title.publicOpinion": "Public Opinion Analysis",
        "dataAnalysis.publicOpinion.visitor": "Total visitors",
        "dataAnalysis.publicOpinion.content": "Total content publishing",
        "dataAnalysis.publicOpinion.comment": "Total comments",
        "dataAnalysis.publicOpinion.share": "Total share",
        "dataAnalysis.title.publishingRate": "Content publishing rate",
        "dataAnalysis.title.publishingTiming": "Content period analysis",
        "dataAnalysis.title.authorsList": "Top authors list",
        "dataAnalysis.authorTable.rank": "Rank    ",
        "dataAnalysis.authorTable.author": "Author",
        "dataAnalysis.authorTable.content": "Interval volume",
        "dataAnalysis.authorTable.click": "Click volume"
    },
    "zh-CN": {
        "menu.visualization": "数据可视化",
        "menu.visualization.analysis": "分析页",
        "dataAnalysis.yesterday": "较昨日",
        "dataAnalysis.title.publicOpinion": "舆情分析",
        "dataAnalysis.publicOpinion.visitor": "访问总人数",
        "dataAnalysis.publicOpinion.content": "内容发布量",
        "dataAnalysis.publicOpinion.comment": "评论总量",
        "dataAnalysis.publicOpinion.share": "分享总量",
        "dataAnalysis.title.publishingRate": "内容发布比例",
        "dataAnalysis.title.publishingTiming": "内容时段分析",
        "dataAnalysis.title.authorsList": "热门作者榜单",
        "dataAnalysis.authorTable.rank": "排名",
        "dataAnalysis.authorTable.author": "作者",
        "dataAnalysis.authorTable.content": "内容量",
        "dataAnalysis.authorTable.click": "点击量"
    }
};
const _default = i18n;

},
"b79838ac": function(module, exports, farmRequire, farmDynamicRequire) {
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
function MultiInterval({ data, loading }) {
    return _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_bizcharts.Chart, {
        height: 370,
        padding: "auto",
        data: data,
        autoFit: true,
        className: 'chart-wrapper'
    }, _react.default.createElement(_bizcharts.Interval, {
        adjust: "stack",
        color: [
            'name',
            [
                '#81E2FF',
                '#00B2FF',
                '#246EFF'
            ]
        ],
        position: "time*count",
        size: 16,
        style: {
            radius: [
                2,
                2,
                0,
                0
            ]
        }
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
        name: "count",
        label: {
            formatter (text) {
                return `${Number(text) / 1000}k`;
            }
        }
    }), _react.default.createElement(_bizcharts.Legend, {
        name: "name",
        marker: {
            symbol: 'circle'
        }
    })));
}
const _default = MultiInterval;

},});