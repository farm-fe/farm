(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'multi-dimension-data-analysis_index_b527.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"c2c12909": function(module, exports, farmRequire, farmDynamicRequire) {
// 数据总览
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
const _multiarealine = /*#__PURE__*/ _interop_require_default._(farmRequire("19c2911b"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("ac636b01"));
const _dataoverviewmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("67d39136"));
const { Title } = _webreact.Typography;
const _default = ()=>{
    const t = (0, _useLocale.default)(_locale.default);
    const [overview, setOverview] = (0, _react.useState)([]);
    const [lineData, setLineData] = (0, _react.useState)([]);
    const [loading, setLoading] = (0, _react.useState)(false);
    const getData = async ()=>{
        setLoading(true);
        const { data } = await _axios.default.get('/api/multi-dimension/overview').finally(()=>setLoading(false));
        const { overviewData, chartData } = data;
        setLineData(chartData);
        setOverview(overviewData);
    };
    (0, _react.useEffect)(()=>{
        getData();
    }, []);
    const formatedData = (0, _react.useMemo)(()=>{
        return [
            {
                title: t['multiDAnalysis.dataOverview.contentProduction'],
                icon: _react.default.createElement(_icon.IconEdit, null),
                value: overview[0],
                background: 'rgb(var(--orange-2))',
                color: 'rgb(var(--orange-6))'
            },
            {
                title: t['multiDAnalysis.dataOverview.contentClicks'],
                icon: _react.default.createElement(_icon.IconThumbUp, null),
                value: overview[1],
                background: 'rgb(var(--cyan-2))',
                color: 'rgb(var(--cyan-6))'
            },
            {
                title: t['multiDAnalysis.dataOverview.contextExposure'],
                value: overview[2],
                icon: _react.default.createElement(_icon.IconHeart, null),
                background: 'rgb(var(--arcoblue-1))',
                color: 'rgb(var(--arcoblue-6))'
            },
            {
                title: t['multiDAnalysis.dataOverview.activeUsers'],
                value: overview[3],
                icon: _react.default.createElement(_icon.IconUser, null),
                background: 'rgb(var(--purple-1))',
                color: 'rgb(var(--purple-6))'
            }
        ];
    }, [
        t,
        overview
    ]);
    return _react.default.createElement(_webreact.Grid.Row, {
        justify: "space-between"
    }, formatedData.map((item, index)=>_react.default.createElement(_webreact.Grid.Col, {
            span: 24 / formatedData.length,
            key: `${index}`
        }, _react.default.createElement(_webreact.Card, {
            className: _dataoverviewmoduleless.default.card,
            title: null
        }, _react.default.createElement(Title, {
            heading: 6
        }, item.title), _react.default.createElement("div", {
            className: _dataoverviewmoduleless.default.content
        }, _react.default.createElement("div", {
            style: {
                backgroundColor: item.background,
                color: item.color
            },
            className: _dataoverviewmoduleless.default['content-icon']
        }, item.icon), loading ? _react.default.createElement(_webreact.Skeleton, {
            animation: true,
            text: {
                rows: 1,
                className: _dataoverviewmoduleless.default['skeleton']
            },
            style: {
                width: '120px'
            }
        }) : _react.default.createElement(_webreact.Statistic, {
            value: item.value,
            groupSeparator: true
        }))))), _react.default.createElement(_webreact.Grid.Col, {
        span: 24
    }, _react.default.createElement(_multiarealine.default, {
        data: lineData,
        loading: loading
    })));
};

},
"f1d225f2": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _dayjs = /*#__PURE__*/ _interop_require_default._(farmRequire("d0dc4dad"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
const _querystring = /*#__PURE__*/ _interop_require_default._(farmRequire("dfa4e0d3"));
const legend = [
    "活跃用户数",
    "内容生产量",
    "内容点击量",
    "内容曝光量"
];
const count = [
    0,
    600,
    1000,
    2000,
    4000
];
const category = [
    "纯文本",
    "图文类",
    "视频类"
];
const getLineData = (name, index)=>{
    const { list } = _mockjs.default.mock({
        "list|10": [
            {
                "id|+1": 1,
                time: function() {
                    return (0, _dayjs.default)().subtract(this.id, "days").format("MM-DD");
                },
                count: ()=>_mockjs.default.Random.natural(count[index], count[index + 1]),
                name: name
            }
        ]
    });
    return list.map((item)=>{
        delete item.id;
        return item;
    });
};
const mockLine = (name)=>{
    const result = new Array(12).fill(0).map(()=>({
            y: _mockjs.default.Random.natural(1000, 10000)
        }));
    return result.sort((a, b)=>a.y - b.y).map((item, index)=>({
            ...item,
            x: index,
            name
        }));
};
const getContentSource = (name)=>{
    const typeList = [
        "UGC原创",
        "国外网站",
        "转载文章",
        "行业报告",
        "其他"
    ];
    const result = [];
    typeList.forEach((type)=>{
        result.push({
            type,
            value: _mockjs.default.Random.natural(100, 10000),
            name
        });
    });
    const total = result.reduce((a, b)=>a + b.value, 0);
    return result.map((item)=>({
            ...item,
            value: Number((item.value / total).toFixed(2))
        }));
};
(0, _setupMock.default)({
    setup: ()=>{
        _mockjs.default.mock(new RegExp("/api/multi-dimension/overview"), ()=>{
            const { array: overviewData } = _mockjs.default.mock({
                "array|4": [
                    function() {
                        return _mockjs.default.Random.natural(0, 10000);
                    }
                ]
            });
            let list = [];
            legend.forEach((name, index)=>list = list.concat(getLineData(name, index)));
            return {
                overviewData,
                chartData: list
            };
        });
        _mockjs.default.mock(new RegExp("/api/multi-dimension/activity"), ()=>{
            const { list } = _mockjs.default.mock({
                "list|3": [
                    {
                        "name|+1": [
                            "分享量",
                            "评论量",
                            "点赞量"
                        ],
                        count: ()=>_mockjs.default.Random.natural(1000, 10000)
                    }
                ]
            });
            return list;
        });
        _mockjs.default.mock(new RegExp("/api/multi-dimension/polar"), ()=>{
            const items = [
                "国际",
                "娱乐",
                "体育",
                "财经",
                "科技",
                "其他"
            ];
            const getCategoryCount = ()=>{
                const result = {};
                category.forEach((name)=>{
                    result[name] = _mockjs.default.Random.natural(0, 100);
                });
                return result;
            };
            return {
                list: items.map((item)=>({
                        item,
                        ...getCategoryCount()
                    })),
                fields: category
            };
        });
        _mockjs.default.mock(new RegExp("/api/multi-dimension/card"), (params)=>{
            const { type } = _querystring.default.parseUrl(params.url).query;
            return _mockjs.default.mock({
                count: ()=>_mockjs.default.Random.natural(1000, 10000),
                increment: ()=>_mockjs.default.Random.boolean(),
                diff: ()=>_mockjs.default.Random.natural(100, 1000),
                chartType: type,
                chartData: ()=>{
                    return mockLine("类目1");
                }
            });
        });
        _mockjs.default.mock(new RegExp("/api/multi-dimension/content-source"), ()=>{
            const allList = category.map((name)=>{
                return getContentSource(name).map((item)=>({
                        ...item,
                        category: name
                    }));
            });
            return allList.flat();
        });
    }
});

},});