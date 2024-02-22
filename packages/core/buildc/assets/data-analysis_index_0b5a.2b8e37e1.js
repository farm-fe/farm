(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'data-analysis_index_0b5a.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"5591c91e": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
const _querystring = /*#__PURE__*/ _interop_require_default._(farmRequire("dfa4e0d3"));
const mockLine = (name)=>{
    const result = new Array(12).fill(0).map(()=>({
            y: _mockjs.default.Random.natural(20, 100)
        }));
    return result.map((item, index)=>({
            ...item,
            x: index,
            name
        }));
};
const mockPie = ()=>{
    return new Array(3).fill(0).map((_, index)=>({
            name: [
                "纯文本",
                "图文类",
                "视频类"
            ][index],
            count: _mockjs.default.Random.natural(20, 100)
        }));
};
(0, _setupMock.default)({
    setup: ()=>{
        _mockjs.default.mock(new RegExp("/api/data-analysis/overview"), (params)=>{
            const { type } = _querystring.default.parseUrl(params.url).query;
            return _mockjs.default.mock({
                count: ()=>_mockjs.default.Random.natural(1000, 10000),
                increment: ()=>_mockjs.default.Random.boolean(),
                diff: ()=>_mockjs.default.Random.natural(100, 1000),
                chartType: type,
                chartData: ()=>{
                    if (type === "pie") {
                        return mockPie();
                    } else if (type === "line") {
                        return [
                            ...mockLine("类目1"),
                            ...mockLine("类目2")
                        ];
                    }
                    return mockLine("类目1");
                }
            });
        });
        const getTimeLine = (name)=>{
            const timeArr = new Array(12).fill(0).map((_, index)=>{
                const time = index * 2;
                return time < 9 ? `0${time}:00` : `${time}:00`;
            });
            return new Array(12).fill(0).map((_, index)=>({
                    name,
                    time: timeArr[index],
                    count: _mockjs.default.Random.natural(1000, 5000),
                    rate: _mockjs.default.Random.natural(0, 100)
                }));
        };
        _mockjs.default.mock(new RegExp("/api/data-analysis/content-publishing"), ()=>{
            return [
                ...getTimeLine("纯文本"),
                ...getTimeLine("视频类"),
                ...getTimeLine("图文类")
            ];
        });
        _mockjs.default.mock(new RegExp("/api/data-analysis/author-list"), ()=>{
            return _mockjs.default.mock({
                "list|8": [
                    {
                        "id|+1": 1,
                        author: ()=>_mockjs.default.Random.pick([
                                "用魔法打败魔法",
                                "王多鱼",
                                "Christopher",
                                "叫我小李好了",
                                "陈皮话梅糖",
                                "碳烤小肥羊"
                            ]),
                        time: function() {
                            return new Array(12).fill(0).map((_, index)=>{
                                const time = index * 2;
                                return time < 9 ? `0${time}:00` : `${time}:00`;
                            })[this.id % 12];
                        },
                        contentCount: ()=>_mockjs.default.Random.natural(1000, 5000),
                        clickCount: ()=>_mockjs.default.Random.natural(5000, 30000)
                    }
                ]
            });
        });
    }
});

},
"69c6321e": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _multistackinterval = /*#__PURE__*/ _interop_require_default._(farmRequire("b79838ac"));
const _periodlegendline = /*#__PURE__*/ _interop_require_default._(farmRequire("9ecadaa2"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("93aaeec9"));
farmRequire("5591c91e");
const _publicopinion = /*#__PURE__*/ _interop_require_default._(farmRequire("f42f5e13"));
const { Row, Col } = _webreact.Grid;
function DataAnalysis() {
    const t = (0, _useLocale.default)(_locale.default);
    const [loading, setLoading] = (0, _react.useState)(false);
    const [tableLoading, setTableLoading] = (0, _react.useState)(false);
    const [chartData, setChartData] = (0, _react.useState)([]);
    const [tableData, setTableData] = (0, _react.useState)([]);
    const getChartData = async ()=>{
        setLoading(true);
        const { data } = await _axios.default.get('/api/data-analysis/content-publishing').finally(()=>setLoading(false));
        setChartData(data);
    };
    const getTableData = async ()=>{
        setTableLoading(true);
        const { data } = await _axios.default.get('/api/data-analysis/author-list').finally(()=>setTableLoading(false));
        setTableData(data.list);
    };
    (0, _react.useEffect)(()=>{
        getChartData();
        getTableData();
    }, []);
    const columns = (0, _react.useMemo)(()=>{
        return [
            {
                title: t['dataAnalysis.authorTable.rank'],
                dataIndex: 'id'
            },
            {
                title: t['dataAnalysis.authorTable.author'],
                dataIndex: 'author'
            },
            {
                title: t['dataAnalysis.authorTable.content'],
                dataIndex: 'contentCount',
                sorter: (a, b)=>a.contentCount - b.contentCount,
                render (x) {
                    return Number(x).toLocaleString();
                }
            },
            {
                title: t['dataAnalysis.authorTable.click'],
                dataIndex: 'clickCount',
                sorter: (a, b)=>a.clickCount - b.clickCount,
                render (x) {
                    return Number(x).toLocaleString();
                }
            }
        ];
    }, [
        t
    ]);
    return _react.default.createElement(_webreact.Space, {
        size: 16,
        direction: "vertical",
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['dataAnalysis.title.publicOpinion']), _react.default.createElement(_publicopinion.default, null)), _react.default.createElement(Row, {
        gutter: 16
    }, _react.default.createElement(Col, {
        span: 14
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['dataAnalysis.title.publishingRate']), _react.default.createElement(_multistackinterval.default, {
        data: chartData,
        loading: loading
    }))), _react.default.createElement(Col, {
        span: 10
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['dataAnalysis.title.authorsList']), _react.default.createElement("div", {
        style: {
            height: '370px'
        }
    }, _react.default.createElement(_webreact.Table, {
        rowKey: "id",
        loading: tableLoading,
        pagination: false,
        data: tableData,
        columns: columns
    }))))), _react.default.createElement(Row, null, _react.default.createElement(Col, {
        span: 24
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['dataAnalysis.title.publishingTiming']), _react.default.createElement(_periodlegendline.default, {
        data: chartData,
        loading: loading
    })))));
}
const _default = DataAnalysis;

},});