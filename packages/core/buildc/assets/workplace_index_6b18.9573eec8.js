(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'workplace_index_6b18.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"3eca280b": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "workplace.welcomeBack": "Welcome Back,",
        "workplace.totalOnlyData": "Total online data",
        "workplace.contentInMarket": "Content in market",
        "workplace.comments": "Comments",
        "workplace.growth": "Growth",
        "workplace.contentData": "Content Data",
        "workplace.1year": "Nearly 1 Year",
        "workplace.seeMore": "See More",
        "workplace.popularContents": "Popular Contents",
        "workplace.text": "Text",
        "workplace.image": "Image",
        "workplace.video": "Video",
        "workplace.column.rank": "Rank",
        "workplace.column.title": "Title",
        "workplace.column.pv": "PV",
        "workplace.column.increase": "Daily Increase",
        "workplace.contentPercentage": "Percentage of content categories",
        "workplace.shortcuts": "Shortcuts",
        "workplace.manage": "Manage",
        "workplace.contentMgmt": "Management",
        "workplace.contentStatistic": "Statistic",
        "workplace.advancedMgmt": "Advance",
        "workplace.onlinePromotion": "Promotion",
        "workplace.marketing": "Marketing",
        "workplace.recent": "Recent",
        "workplace.announcement": "Announcement",
        "workplace.activity": "Activity",
        "workplace.info": "Info",
        "workplace.notice": "Notice",
        "workplace.docs": "Document",
        "workplace.pecs": "pecs",
        "workplace.designLab": "DesignLab",
        "workplace.materialMarket": "MaterialMarket",
        "workplace.react": "React Quick Start",
        "workplace.vue": "Vue Quick Start"
    },
    "zh-CN": {
        "workplace.welcomeBack": "欢迎回来，",
        "workplace.totalOnlyData": "线上总数据",
        "workplace.contentInMarket": "投放中的内容",
        "workplace.comments": "日新增评论",
        "workplace.growth": "较昨日新增",
        "workplace.contentData": "内容数据",
        "workplace.1year": "近1年",
        "workplace.seeMore": "查看更多",
        "workplace.popularContents": "线上热门内容",
        "workplace.text": "文本",
        "workplace.image": "图文",
        "workplace.video": "视频",
        "workplace.column.rank": "排名",
        "workplace.column.title": "内容标题",
        "workplace.column.pv": "点击量",
        "workplace.column.increase": "日涨幅",
        "workplace.contentPercentage": "内容类别占比",
        "workplace.shortcuts": "快捷入口",
        "workplace.manage": "管理",
        "workplace.contentMgmt": "内容管理",
        "workplace.contentStatistic": "内容数据",
        "workplace.advancedMgmt": "高级管理",
        "workplace.onlinePromotion": "线上推广",
        "workplace.marketing": "内容投放",
        "workplace.recent": "最近访问",
        "workplace.announcement": "公告",
        "workplace.activity": "活动",
        "workplace.info": "消息",
        "workplace.notice": "通知",
        "workplace.docs": "文档中心",
        "workplace.pecs": "个",
        "workplace.designLab": "风格配置平台",
        "workplace.materialMarket": "物料市场",
        "workplace.react": "React 组件库",
        "workplace.vue": "Vue 组件库"
    }
};
const _default = i18n;

},
"5f303d81": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("3eca280b"));
function PopularContent() {
    const t = (0, _useLocale.default)(_locale.default);
    const [data, setData] = (0, _react.useState)([]);
    const [loading, setLoading] = (0, _react.useState)(true);
    const fetchData = ()=>{
        setLoading(true);
        _axios.default.get('/api/workplace/content-percentage').then((res)=>{
            setData(res.data);
        }).finally(()=>{
            setLoading(false);
        });
    };
    (0, _react.useEffect)(()=>{
        fetchData();
    }, []);
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['workplace.contentPercentage']), _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            display: 'block'
        }
    }, _react.default.createElement(_bizcharts.DonutChart, {
        autoFit: true,
        height: 340,
        data: data,
        radius: 0.7,
        innerRadius: 0.65,
        angleField: "count",
        colorField: "type",
        color: [
            '#21CCFF',
            '#313CA9',
            '#249EFF'
        ],
        interactions: [
            {
                type: 'element-single-selected'
            }
        ],
        tooltip: {
            showMarkers: false
        },
        label: {
            visible: true,
            type: 'spider',
            formatter: (v)=>`${(v.percent * 100).toFixed(0)}%`,
            style: {
                fill: '#86909C',
                fontSize: 14
            }
        },
        legend: {
            position: 'bottom'
        },
        statistic: {
            title: {
                style: {
                    fontSize: '14px',
                    lineHeight: 2,
                    color: 'rgb(--var(color-text-1))'
                },
                formatter: ()=>'内容量'
            },
            content: {
                style: {
                    fontSize: '16px',
                    color: 'rgb(--var(color-text-1))'
                },
                formatter: (_, data)=>{
                    const sum = data.reduce((a, b)=>a + b.count, 0);
                    return Number(sum).toLocaleString();
                }
            }
        }
    })));
}
const _default = PopularContent;

},
"956f3d7a": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
const _querystring = /*#__PURE__*/ _interop_require_default._(farmRequire("dfa4e0d3"));
(0, _setupMock.default)({
    setup: ()=>{
        _mockjs.default.mock(new RegExp("/api/workplace/overview-content"), ()=>{
            const year = new Date().getFullYear();
            const getLineData = ()=>{
                return new Array(12).fill(0).map((_item, index)=>({
                        date: `${year}-${index + 1}`,
                        count: _mockjs.default.Random.natural(20000, 75000)
                    }));
            };
            return {
                allContents: "373.5w+",
                liveContents: "368",
                increaseComments: "8874",
                growthRate: "2.8%",
                chartData: getLineData()
            };
        });
        const getList = ()=>{
            const { list } = _mockjs.default.mock({
                "list|100": [
                    {
                        "rank|+1": 1,
                        title: ()=>_mockjs.default.Random.pick([
                                "经济日报：财政政策要精准提升效能",
                                "“双12”遇冷消费者厌倦了电商平台的促销“套路”",
                                "致敬坚守战“疫”一线的社区工作者",
                                "普高还是职高？家长们陷入选校难题"
                            ]),
                        pv: function() {
                            return 500000 - 3200 * this.rank;
                        },
                        increase: "@float(-1, 1)"
                    }
                ]
            });
            return list;
        };
        const listText = getList();
        const listPic = getList();
        const listVideo = getList();
        _mockjs.default.mock(new RegExp("/api/workplace/popular-contents"), (params)=>{
            const { page = 1, pageSize = 5, category = 0 } = _querystring.default.parseUrl(params.url).query;
            const list = [
                listText,
                listPic,
                listVideo
            ][Number(category)];
            return {
                list: list.slice((page - 1) * pageSize, page * pageSize),
                total: 100
            };
        });
        _mockjs.default.mock(new RegExp("/api/workplace/content-percentage"), ()=>{
            return [
                {
                    type: "纯文本",
                    count: 148564,
                    percent: 0.16
                },
                {
                    type: "图文类",
                    count: 334271,
                    percent: 0.36
                },
                {
                    type: "视频类",
                    count: 445695,
                    percent: 0.48
                }
            ];
        });
        _mockjs.default.mock(new RegExp("/api/workplace/announcement"), ()=>{
            return [
                {
                    type: "activity",
                    key: "1",
                    content: "内容最新优惠活动"
                },
                {
                    type: "info",
                    key: "2",
                    content: "新增内容尚未通过审核，详情请点击查看。"
                },
                {
                    type: "notice",
                    key: "3",
                    content: "当前产品试用期即将结束，如需续费请点击查看。"
                },
                {
                    type: "notice",
                    key: "4",
                    content: "1 月新系统升级计划通知"
                },
                {
                    type: "info",
                    key: "5",
                    content: "新增内容已经通过审核，详情请点击查看。"
                }
            ];
        });
    }
});

},
"cc60be79": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("3eca280b"));
const _shortcutsmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("f3922c4a"));
function Shortcuts() {
    const t = (0, _useLocale.default)(_locale.default);
    const shortcuts = [
        {
            title: t['workplace.contentMgmt'],
            key: 'Content Management',
            icon: _react.default.createElement(_icon.IconFile, null)
        },
        {
            title: t['workplace.contentStatistic'],
            key: 'Content Statistic',
            icon: _react.default.createElement(_icon.IconStorage, null)
        },
        {
            title: t['workplace.advancedMgmt'],
            key: 'Advanced Management',
            icon: _react.default.createElement(_icon.IconSettings, null)
        },
        {
            title: t['workplace.onlinePromotion'],
            key: 'Online Promotion',
            icon: _react.default.createElement(_icon.IconMobile, null)
        },
        {
            title: t['workplace.marketing'],
            key: 'Marketing',
            icon: _react.default.createElement(_icon.IconFire, null)
        }
    ];
    const recentShortcuts = [
        {
            title: t['workplace.contentStatistic'],
            key: 'Content Statistic',
            icon: _react.default.createElement(_icon.IconStorage, null)
        },
        {
            title: t['workplace.contentMgmt'],
            key: 'Content Management',
            icon: _react.default.createElement(_icon.IconFile, null)
        },
        {
            title: t['workplace.advancedMgmt'],
            key: 'Advanced Management',
            icon: _react.default.createElement(_icon.IconSettings, null)
        }
    ];
    function onClickShortcut(key) {
        _webreact.Message.info({
            content: _react.default.createElement("span", null, "You clicked ", _react.default.createElement("b", null, key))
        });
    }
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement("div", {
        style: {
            display: 'flex',
            justifyContent: 'space-between'
        }
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['workplace.shortcuts']), _react.default.createElement(_webreact.Link, null, t['workplace.seeMore'])), _react.default.createElement("div", {
        className: _shortcutsmoduleless.default.shortcuts
    }, shortcuts.map((shortcut)=>_react.default.createElement("div", {
            className: _shortcutsmoduleless.default.item,
            key: shortcut.key,
            onClick: ()=>onClickShortcut(shortcut.key)
        }, _react.default.createElement("div", {
            className: _shortcutsmoduleless.default.icon
        }, shortcut.icon), _react.default.createElement("div", {
            className: _shortcutsmoduleless.default.title
        }, shortcut.title)))), _react.default.createElement(_webreact.Divider, null), _react.default.createElement("div", {
        className: _shortcutsmoduleless.default.recent
    }, t['workplace.recent']), _react.default.createElement("div", {
        className: _shortcutsmoduleless.default.shortcuts
    }, recentShortcuts.map((shortcut)=>_react.default.createElement("div", {
            className: _shortcutsmoduleless.default.item,
            key: shortcut.key,
            onClick: ()=>onClickShortcut(shortcut.key)
        }, _react.default.createElement("div", {
            className: _shortcutsmoduleless.default.icon
        }, shortcut.icon), _react.default.createElement("div", {
            className: _shortcutsmoduleless.default.title
        }, shortcut.title)))));
}
const _default = Shortcuts;

},
"deb5811d": function(module, exports, farmRequire, farmDynamicRequire) {
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
function OverviewAreaLine({ data, loading, name = '总内容量', color = '#4080FF' }) {
    return _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            width: '100%'
        }
    }, _react.default.createElement(_bizcharts.Chart, {
        scale: {
            value: {
                min: 0
            }
        },
        padding: [
            10,
            20,
            50,
            40
        ],
        autoFit: true,
        height: 300,
        data: data,
        className: 'chart-wrapper'
    }, _react.default.createElement(_bizcharts.Axis, {
        name: "count",
        title: true,
        grid: {
            line: {
                style: {
                    lineDash: [
                        4,
                        4
                    ]
                }
            }
        },
        label: {
            formatter (text) {
                return `${Number(text) / 1000}k`;
            }
        }
    }), _react.default.createElement(_bizcharts.Axis, {
        name: "date",
        grid: {
            line: {
                style: {
                    stroke: '#E5E8EF'
                }
            }
        }
    }), _react.default.createElement(_bizcharts.Line, {
        shape: "smooth",
        position: "date*count",
        size: 3,
        color: "l (0) 0:#1EE7FF .57:#249AFF .85:#6F42FB"
    }), _react.default.createElement(_bizcharts.Area, {
        position: "date*count",
        shape: "smooth",
        color: "l (90) 0:rgba(17, 126, 255, 0.5)  1:rgba(17, 128, 255, 0)"
    }), _react.default.createElement(_bizcharts.Tooltip, {
        showCrosshairs: true,
        showMarkers: true,
        marker: {
            lineWidth: 3,
            stroke: color,
            fill: '#ffffff',
            symbol: 'circle',
            r: 8
        }
    }, (title, items)=>{
        return _react.default.createElement(_customertooltip.default, {
            title: title,
            data: items,
            color: color,
            name: name,
            formatter: (value)=>Number(value).toLocaleString()
        });
    })));
}
const _default = OverviewAreaLine;

},});