(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'card_index_bd38.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"41bebf2f": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _dayjs = /*#__PURE__*/ _interop_require_default._(farmRequire("d0dc4dad"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
const qualityCategory = [
    "视频类",
    "图文类",
    "纯文本"
];
const qualityName = [
    "历史导入",
    "内容版权",
    "敏感内容",
    "商业品牌"
];
const serviceName = [
    "漏斗分析",
    "用户分布",
    "资源分发",
    "用户画像分析",
    "事件分析"
];
const serviceDescriptions = [
    "用户行为分析之漏斗分析模型是企业实现精细化运营、进行用户行为分析的重要数据分析模型。 ",
    "快速诊断用户人群，地域细分情况，了解数据分布的集中度，以及主要的数据分布的区间段是什么。",
    "移动端动态化资源分发解决方案。提供稳定大流量服务支持、灵活定制的分发圈选规则，通过离线化预加载。  ",
    "用户画像就是将典型用户信息标签化，根据用户特征、业务场景和用户行为等信息，构建一个标签化的用户模型。 ",
    "事件分析即可进行筛选、分组、聚合的灵活多维数据分析。详情请点击卡片。"
];
const rulesName = [
    "内容屏蔽规则",
    "内容置顶规则",
    "内容加权规则",
    "内容分发规则",
    "多语言文字符号识别"
];
const rulesDescription = [
    "用户在执行特定的内容分发任务时，可使用内容屏蔽规则根据特定标签，过滤内容集合。  ",
    "该规则支持用户在执行特定内容分发任务时，对固定的几条内容置顶。",
    "选定内容加权规则后可自定义从不同内容集合获取内容的概率。",
    "内容分发时，对某些内容需要固定在C端展示的位置。",
    "精准识别英语、维语、藏语、蒙古语、朝鲜语等多种语言以及emoji表情形态的语义识别。"
];
const getQualityCard = ()=>{
    const { list } = _mockjs.default.mock({
        "list|10": [
            {
                title: ()=>`${_mockjs.default.Random.pick(qualityCategory)}-${_mockjs.default.Random.pick(qualityName)}`,
                time: ()=>(0, _dayjs.default)().subtract(_mockjs.default.Random.natural(0, 30), "days").format("YYYY-MM-DD HH:mm:ss"),
                qualityCount: ()=>_mockjs.default.Random.natural(100, 500),
                randomCount: ()=>_mockjs.default.Random.natural(0, 100),
                duration: ()=>_mockjs.default.Random.natural(0, 200)
            }
        ]
    });
    return list;
};
const getServiceCard = ()=>{
    const { list } = _mockjs.default.mock({
        "list|10": [
            {
                icon: ()=>_mockjs.default.Random.natural(0, serviceName.length - 1),
                title: function() {
                    return serviceName[this.icon];
                },
                description: function() {
                    return serviceDescriptions[this.icon];
                },
                status: ()=>_mockjs.default.Random.natural(0, 2)
            }
        ]
    });
    return list;
};
const getRulesCard = ()=>{
    const { list } = _mockjs.default.mock({
        "list|10": [
            {
                index: ()=>_mockjs.default.Random.natural(0, rulesName.length - 1),
                title: function() {
                    return rulesName[this.index];
                },
                description: function() {
                    return rulesDescription[this.index];
                },
                status: ()=>_mockjs.default.Random.natural(0, 1)
            }
        ]
    });
    return list;
};
(0, _setupMock.default)({
    setup: ()=>{
        _mockjs.default.mock(new RegExp("/api/cardList"), ()=>{
            return {
                quality: getQualityCard(),
                service: getServiceCard(),
                rules: getRulesCard()
            };
        });
    }
});

},
"b5c07e81": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return ListCard;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _cardadd = /*#__PURE__*/ _interop_require_default._(farmRequire("e576be20"));
const _cardblock = /*#__PURE__*/ _interop_require_default._(farmRequire("72346ccd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("72edb2f7"));
farmRequire("41bebf2f");
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("35c3ed68"));
const { Title } = _webreact.Typography;
const { Row, Col } = _webreact.Grid;
const defaultList = new Array(10).fill({});
function ListCard() {
    const t = (0, _useLocale.default)(_locale.default);
    const [loading, setLoading] = (0, _react.useState)(true);
    const [data, setData] = (0, _react.useState)({
        quality: defaultList,
        service: defaultList,
        rules: defaultList
    });
    const [activeKey, setActiveKey] = (0, _react.useState)('all');
    const getData = ()=>{
        _axios.default.get('/api/cardList').then((res)=>{
            setData(res.data);
        }).finally(()=>setLoading(false));
    };
    (0, _react.useEffect)(()=>{
        getData();
    }, []);
    7;
    const getCardList = (list, type)=>{
        return _react.default.createElement(Row, {
            gutter: 24,
            className: _indexmoduleless.default['card-content']
        }, type === 'quality' && _react.default.createElement(Col, {
            xs: 24,
            sm: 12,
            md: 8,
            lg: 6,
            xl: 6,
            xxl: 6
        }, _react.default.createElement(_cardadd.default, {
            description: t['cardList.add.quality']
        })), list.map((item, index)=>_react.default.createElement(Col, {
                xs: 24,
                sm: 12,
                md: 8,
                lg: 6,
                xl: 6,
                xxl: 6,
                key: index
            }, _react.default.createElement(_cardblock.default, {
                card: item,
                type: type,
                loading: loading
            }))));
    };
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(Title, {
        heading: 6
    }, t['menu.list.card']), _react.default.createElement(_webreact.Tabs, {
        activeTab: activeKey,
        type: "rounded",
        onChange: setActiveKey,
        extra: _react.default.createElement(_webreact.Input.Search, {
            style: {
                width: '240px'
            },
            placeholder: t[`cardList.tab.${activeKey}.placeholder`]
        })
    }, _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "all",
        title: t['cardList.tab.title.all']
    }), _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "quality",
        title: t['cardList.tab.title.quality']
    }), _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "service",
        title: t['cardList.tab.title.service']
    }), _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "rules",
        title: t['cardList.tab.title.rules']
    })), _react.default.createElement("div", {
        className: _indexmoduleless.default.container
    }, activeKey === 'all' ? Object.entries(data).map(([key, list])=>_react.default.createElement("div", {
            key: key
        }, _react.default.createElement(Title, {
            heading: 6
        }, t[`cardList.tab.title.${key}`]), getCardList(list, key))) : _react.default.createElement("div", {
        className: _indexmoduleless.default['single-content']
    }, getCardList(data[activeKey], activeKey))));
}

},});