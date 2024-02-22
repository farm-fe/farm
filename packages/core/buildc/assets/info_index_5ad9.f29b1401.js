(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'info_index_5ad9.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"0e07cb09": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _reactredux = farmRequire("e429bf23");
const _header = /*#__PURE__*/ _interop_require_default._(farmRequire("9e13e96b"));
const _latestnews = /*#__PURE__*/ _interop_require_default._(farmRequire("f9dd033a"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("c52355c6"));
farmRequire("726ab408");
const _myprojects = /*#__PURE__*/ _interop_require_default._(farmRequire("6fe47e30"));
const _myteam = /*#__PURE__*/ _interop_require_default._(farmRequire("8e5823c1"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("0a457a53"));
const { Title } = _webreact.Typography;
const { Row, Col } = _webreact.Grid;
function UserInfo() {
    const t = (0, _useLocale.default)(_locale.default);
    const userInfo = (0, _reactredux.useSelector)((state)=>state.userInfo);
    const loading = (0, _reactredux.useSelector)((state)=>state.userLoading);
    const [noticeLoading, setNoticeLoading] = (0, _react.useState)(false);
    const getNotice = async ()=>{
        setNoticeLoading(true);
        await _axios.default.get('/api/user/notice').finally(()=>setNoticeLoading(false));
    };
    (0, _react.useEffect)(()=>{
        getNotice();
    }, []);
    return _react.default.createElement("div", null, _react.default.createElement(_header.default, {
        userInfo: userInfo,
        loading: loading
    }), _react.default.createElement(Row, {
        gutter: 16
    }, _react.default.createElement(Col, {
        span: 16
    }, _react.default.createElement(_webreact.Card, {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['card-title-wrapper']
    }, _react.default.createElement(Title, {
        heading: 6,
        style: {
            marginBottom: '20px'
        }
    }, t['userInfo.title.project']), _react.default.createElement(_webreact.Link, null, t['userInfo.btn.more'])), _react.default.createElement(_myprojects.default, null))), _react.default.createElement(Col, {
        span: 8
    }, _react.default.createElement(_webreact.Card, {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['card-title-wrapper']
    }, _react.default.createElement(Title, {
        heading: 6,
        style: {
            marginBottom: '12px'
        }
    }, t['userInfo.title.team'])), _react.default.createElement(_myteam.default, null)))), _react.default.createElement(Row, {
        gutter: 16
    }, _react.default.createElement(Col, {
        span: 16
    }, _react.default.createElement(_webreact.Card, {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['card-title-wrapper']
    }, _react.default.createElement(Title, {
        heading: 6,
        style: {
            marginBottom: '8px'
        }
    }, t['userInfo.title.news']), _react.default.createElement(_webreact.Link, null, t['userInfo.btn.all'])), _react.default.createElement(_latestnews.default, null))), _react.default.createElement(Col, {
        span: 8
    }, _react.default.createElement(_webreact.Card, {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['card-title-wrapper']
    }, _react.default.createElement(Title, {
        heading: 6
    }, t['userInfo.title.notice'])), noticeLoading ? _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 10
        },
        animation: true
    }) : _react.default.createElement(_webreact.Result, {
        status: "404",
        subTitle: t['userInfo.notice.empty'],
        style: {
            paddingTop: '60px',
            paddingBottom: '130px'
        }
    })))));
}
const _default = UserInfo;

},
"726ab408": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
(0, _setupMock.default)({
    setup: ()=>{
        // 我的项目
        _mockjs.default.mock(new RegExp("/api/user/projectList"), ()=>{
            const contributors = [
                {
                    name: "秦臻宇",
                    email: "qingzhenyu@arco.design",
                    avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/a8c8cdb109cb051163646151a4a5083b.png~tplv-uwbnlip3yd-webp.webp"
                },
                {
                    name: "于涛",
                    email: "yuebao@arco.design",
                    avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/a8c8cdb109cb051163646151a4a5083b.png~tplv-uwbnlip3yd-webp.webp"
                },
                {
                    name: "宁波",
                    email: "ningbo@arco.design",
                    avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp"
                },
                {
                    name: "郑曦月",
                    email: "zhengxiyue@arco.design",
                    avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/8361eeb82904210b4f55fab888fe8416.png~tplv-uwbnlip3yd-webp.webp"
                },
                {
                    name: "宁波",
                    email: "ningbo@arco.design",
                    avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp"
                }
            ];
            return new Array(6).fill(null).map((_item, index)=>({
                    id: index,
                    enTitle: [
                        "Arco Design System",
                        "The Volcano Engine",
                        "OCR text recognition",
                        "Content resource management",
                        "Toutiao content management",
                        "Intelligent Robot Project"
                    ][index],
                    title: [
                        "企业级产品设计系统",
                        "火山引擎智能应用",
                        "OCR文本识别",
                        "内容资源管理",
                        "今日头条内容管理",
                        "智能机器人"
                    ][index],
                    contributors,
                    contributorsLength: _mockjs.default.Random.natural(5, 100)
                }));
        });
        // 我的团队
        _mockjs.default.mock(new RegExp("/api/users/teamList"), ()=>{
            return new Array(4).fill(null).map((_, index)=>({
                    name: [
                        "火山引擎智能应用团队",
                        "企业级产品设计团队",
                        "前端/UE小分队",
                        "内容识别插件小分队"
                    ][index],
                    avatar: [
                        "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/a8c8cdb109cb051163646151a4a5083b.png~tplv-uwbnlip3yd-webp.webp",
                        "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp",
                        "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp",
                        "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/8361eeb82904210b4f55fab888fe8416.png~tplv-uwbnlip3yd-webp.webp"
                    ][index],
                    members: _mockjs.default.Random.natural(1, 1000)
                }));
        });
        // 最新动态
        _mockjs.default.mock(new RegExp("/api/user/latestNews"), ()=>{
            return new Array(8).fill(null).map((_item, index)=>({
                    id: index,
                    title: "王多鱼审核了图文内容： 2021年，你过得怎么样？",
                    description: "新华网年终特别策划：《这一年，你过得怎么样？》回访那些你最熟悉的“陌生人”带你重温这难忘的2021年回顾我们共同记忆中的生动故事！",
                    avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/a8c8cdb109cb051163646151a4a5083b.png~tplv-uwbnlip3yd-webp.webp"
                }));
        });
        // 站内通知
        _mockjs.default.mock(new RegExp("/api/user/notice"), ()=>{
            return [];
        });
    }
});

},});