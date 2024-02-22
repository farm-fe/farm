(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_704c.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"0526fb72": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _is = farmRequire("1cb2548c");
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
farmRequire("11f60e04");
farmRequire("29b8f8b1");
if (!_is.isSSR) {
    _mockjs.default.setup({
        timeout: "500-1500"
    });
}

},
"06e27f5c": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "dropdown-icon": `dropdown-icon-f78761a4`,
    "fixed-settings": `fixed-settings-f78761a4`,
    "left": `left-f78761a4`,
    "logo": `logo-f78761a4`,
    "logo-name": `logo-name-f78761a4`,
    "navbar": `navbar-f78761a4`,
    "right": `right-f78761a4`,
    "round": `round-f78761a4`,
    "username": `username-f78761a4`
};

},
"0f9ad454": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _icon = farmRequire("f988cd7d");
const _copytoclipboard = /*#__PURE__*/ _interop_require_default._(farmRequire("7e415b87"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _IconButton = /*#__PURE__*/ _interop_require_default._(farmRequire("b04795b8"));
const _block = /*#__PURE__*/ _interop_require_default._(farmRequire("1e670b54"));
const _color = /*#__PURE__*/ _interop_require_default._(farmRequire("abbbd081"));
function Setting(props) {
    const { trigger } = props;
    const [visible, setVisible] = (0, _react.useState)(false);
    const locale = (0, _useLocale.default)();
    const settings = (0, _reactredux.useSelector)((state)=>state.settings);
    function onCopySettings() {
        (0, _copytoclipboard.default)(JSON.stringify(settings, null, 2));
        _webreact.Message.success(locale['settings.copySettings.message']);
    }
    return _react.default.createElement(_react.default.Fragment, null, trigger ? _react.default.cloneElement(trigger, {
        onClick: ()=>setVisible(true)
    }) : _react.default.createElement(_IconButton.default, {
        icon: _react.default.createElement(_icon.IconSettings, null),
        onClick: ()=>setVisible(true)
    }), _react.default.createElement(_webreact.Drawer, {
        width: 300,
        title: _react.default.createElement(_react.default.Fragment, null, _react.default.createElement(_icon.IconSettings, null), locale['settings.title']),
        visible: visible,
        okText: locale['settings.copySettings'],
        cancelText: locale['settings.close'],
        onOk: onCopySettings,
        onCancel: ()=>setVisible(false)
    }, _react.default.createElement(_block.default, {
        title: locale['settings.themeColor']
    }, _react.default.createElement(_color.default, null)), _react.default.createElement(_block.default, {
        title: locale['settings.content'],
        options: [
            {
                name: 'settings.navbar',
                value: 'navbar'
            },
            {
                name: 'settings.menu',
                value: 'menu'
            },
            {
                name: 'settings.footer',
                value: 'footer'
            },
            {
                name: 'settings.menuWidth',
                value: 'menuWidth',
                type: 'number'
            }
        ]
    }), _react.default.createElement(_block.default, {
        title: locale['settings.otherSettings'],
        options: [
            {
                name: 'settings.colorWeek',
                value: 'colorWeek'
            }
        ]
    }), _react.default.createElement(_webreact.Alert, {
        content: locale['settings.alertContent']
    })));
}
const _default = Setting;

},
"11f60e04": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
const haveReadIds = [];
const getMessageList = ()=>{
    return [
        {
            id: 1,
            type: "message",
            title: "郑曦月",
            subTitle: "的私信",
            avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/8361eeb82904210b4f55fab888fe8416.png~tplv-uwbnlip3yd-webp.webp",
            content: "审批请求已发送，请查收",
            time: "今天 12:30:01"
        },
        {
            id: 2,
            type: "message",
            title: "宁波",
            subTitle: "的回复",
            avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp",
            content: "此处 bug 已经修复，如有问题请查阅文档或者继续 github 提 issue～",
            time: "今天 12:30:01"
        },
        {
            id: 3,
            type: "message",
            title: "宁波",
            subTitle: "的回复",
            avatar: "//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/3ee5f13fb09879ecb5185e440cef6eb9.png~tplv-uwbnlip3yd-webp.webp",
            content: "此处 bug 已经修复",
            time: "今天 12:20:01"
        },
        {
            id: 4,
            type: "todo",
            title: "域名服务",
            content: "内容质检队列于 2021-12-01 19:50:23 进行变更，请重新",
            tag: {
                text: "未开始",
                color: "gray"
            }
        },
        {
            id: 5,
            type: "todo",
            title: "内容审批通知",
            content: "宁静提交于 2021-11-05，需要您在 2011-11-07之前审批",
            tag: {
                text: "进行中",
                color: "arcoblue"
            }
        },
        {
            id: 6,
            type: "notice",
            title: "质检队列变更",
            content: "您的产品使用期限即将截止，如需继续使用产品请前往购…",
            tag: {
                text: "即将到期",
                color: "red"
            }
        },
        {
            id: 7,
            type: "notice",
            title: "规则开通成功",
            subTitle: "",
            avatar: "",
            content: "内容屏蔽规则于 2021-12-01 开通成功并生效。",
            tag: {
                text: "已开通",
                color: "green"
            }
        }
    ].map((item)=>({
            ...item,
            status: haveReadIds.indexOf(item.id) === -1 ? 0 : 1
        }));
};
(0, _setupMock.default)({
    setup: ()=>{
        _mockjs.default.mock(new RegExp("/api/message/list"), ()=>{
            return getMessageList();
        });
        _mockjs.default.mock(new RegExp("/api/message/read"), (params)=>{
            const { ids } = JSON.parse(params.body);
            haveReadIds.push(...ids || []);
            return true;
        });
    }
});

},
"11ff0679": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    default: function() {
        return _default;
    },
    generatePermission: function() {
        return generatePermission;
    },
    routes: function() {
        return routes;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _authentication = /*#__PURE__*/ _interop_require_default._(farmRequire("5e369e13"));
const _react = farmRequire("a0fc9dfd");
const routes = [
    {
        name: "menu.dashboard",
        key: "dashboard",
        children: [
            {
                name: "menu.dashboard.workplace",
                key: "dashboard/workplace"
            },
            {
                name: "menu.dashboard.monitor",
                key: "dashboard/monitor",
                requiredPermissions: [
                    {
                        resource: "menu.dashboard.monitor",
                        actions: [
                            "write"
                        ]
                    }
                ]
            }
        ]
    },
    {
        name: "menu.visualization",
        key: "visualization",
        children: [
            {
                name: "menu.visualization.dataAnalysis",
                key: "visualization/data-analysis",
                requiredPermissions: [
                    {
                        resource: "menu.visualization.dataAnalysis",
                        actions: [
                            "read"
                        ]
                    }
                ]
            },
            {
                name: "menu.visualization.multiDimensionDataAnalysis",
                key: "visualization/multi-dimension-data-analysis",
                requiredPermissions: [
                    {
                        resource: "menu.visualization.dataAnalysis",
                        actions: [
                            "read",
                            "write"
                        ]
                    },
                    {
                        resource: "menu.visualization.multiDimensionDataAnalysis",
                        actions: [
                            "write"
                        ]
                    }
                ],
                oneOfPerm: true
            }
        ]
    },
    {
        name: "menu.list",
        key: "list",
        children: [
            {
                name: "menu.list.searchTable",
                key: "list/search-table"
            },
            {
                name: "menu.list.cardList",
                key: "list/card"
            }
        ]
    },
    {
        name: "menu.form",
        key: "form",
        children: [
            {
                name: "menu.form.group",
                key: "form/group",
                requiredPermissions: [
                    {
                        resource: "menu.form.group",
                        actions: [
                            "read",
                            "write"
                        ]
                    }
                ]
            },
            {
                name: "menu.form.step",
                key: "form/step",
                requiredPermissions: [
                    {
                        resource: "menu.form.step",
                        actions: [
                            "read"
                        ]
                    }
                ]
            }
        ]
    },
    {
        name: "menu.profile",
        key: "profile",
        children: [
            {
                name: "menu.profile.basic",
                key: "profile/basic"
            }
        ]
    },
    {
        name: "menu.result",
        key: "result",
        children: [
            {
                name: "menu.result.success",
                key: "result/success",
                breadcrumb: false
            },
            {
                name: "menu.result.error",
                key: "result/error",
                breadcrumb: false
            }
        ]
    },
    {
        name: "menu.exception",
        key: "exception",
        children: [
            {
                name: "menu.exception.403",
                key: "exception/403"
            },
            {
                name: "menu.exception.404",
                key: "exception/404"
            },
            {
                name: "menu.exception.500",
                key: "exception/500"
            }
        ]
    },
    {
        name: "menu.user",
        key: "user",
        children: [
            {
                name: "menu.user.info",
                key: "user/info"
            },
            {
                name: "menu.user.setting",
                key: "user/setting"
            }
        ]
    }
];
const generatePermission = (role)=>{
    const actions = role === "admin" ? [
        "*"
    ] : [
        "read"
    ];
    const result = {};
    routes.forEach((item)=>{
        if (item.children) {
            item.children.forEach((child)=>{
                result[child.name] = actions;
            });
        }
    });
    return result;
};
const useRoute = (userPermission)=>{
    const filterRoute = (routes, arr = [])=>{
        if (!routes.length) {
            return [];
        }
        for (const route of routes){
            const { requiredPermissions, oneOfPerm } = route;
            let visible = true;
            if (requiredPermissions) {
                visible = (0, _authentication.default)({
                    requiredPermissions,
                    oneOfPerm
                }, userPermission);
            }
            if (!visible) {
                continue;
            }
            if (route.children && route.children.length) {
                const newRoute = {
                    ...route,
                    children: []
                };
                filterRoute(route.children, newRoute.children);
                if (newRoute.children.length) {
                    arr.push(newRoute);
                }
            } else {
                arr.push({
                    ...route
                });
            }
        }
        return arr;
    };
    const [permissionRoute, setPermissionRoute] = (0, _react.useState)(routes);
    (0, _react.useEffect)(()=>{
        const newRoutes = filterRoute(routes);
        setPermissionRoute(newRoutes);
    }, [
        userPermission
    ]);
    const defaultRoute = (0, _react.useMemo)(()=>{
        const first = permissionRoute[0];
        if (first) {
            const firstRoute = first?.children?.[0]?.key || first.key;
            return firstRoute;
        }
        return "";
    }, [
        permissionRoute
    ]);
    return [
        permissionRoute,
        defaultRoute
    ];
};
const _default = useRoute;

},
"16203e8b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _routes = /*#__PURE__*/ _interop_require_default._(farmRequire("11ff0679"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _nprogress = /*#__PURE__*/ _interop_require_default._(farmRequire("80963b0b"));
const _querystring = /*#__PURE__*/ _interop_require_default._(farmRequire("dfa4e0d3"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _reactrouterdom = farmRequire("120e6d2c");
const _Footer = /*#__PURE__*/ _interop_require_default._(farmRequire("d8012317"));
const _NavBar = /*#__PURE__*/ _interop_require_default._(farmRequire("65422662"));
const _layoutmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("c4691837"));
const _getUrlParams = /*#__PURE__*/ _interop_require_default._(farmRequire("ac8842a4"));
const _is = farmRequire("1cb2548c");
const _lazyload = /*#__PURE__*/ _interop_require_default._(farmRequire("7b631473"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const MenuItem = _webreact.Menu.Item;
const SubMenu = _webreact.Menu.SubMenu;
const Sider = _webreact.Layout.Sider;
const Content = _webreact.Layout.Content;
function getIconFromKey(key) {
    switch(key){
        case 'dashboard':
            return _react.default.createElement(_icon.IconDashboard, {
                className: _layoutmoduleless.default.icon
            });
        case 'list':
            return _react.default.createElement(_icon.IconList, {
                className: _layoutmoduleless.default.icon
            });
        case 'form':
            return _react.default.createElement(_icon.IconSettings, {
                className: _layoutmoduleless.default.icon
            });
        case 'profile':
            return _react.default.createElement(_icon.IconFile, {
                className: _layoutmoduleless.default.icon
            });
        case 'visualization':
            return _react.default.createElement(_icon.IconApps, {
                className: _layoutmoduleless.default.icon
            });
        case 'result':
            return _react.default.createElement(_icon.IconCheckCircle, {
                className: _layoutmoduleless.default.icon
            });
        case 'exception':
            return _react.default.createElement(_icon.IconExclamationCircle, {
                className: _layoutmoduleless.default.icon
            });
        case 'user':
            return _react.default.createElement(_icon.IconUser, {
                className: _layoutmoduleless.default.icon
            });
        default:
            return _react.default.createElement("div", {
                className: _layoutmoduleless.default['icon-empty']
            });
    }
}
function getFlattenRoutes(routes) {
    const res = [];
    // function travel(_routes) {
    //   _routes.forEach((route) => {
    //     if (route.key && !route.children) {
    //       route.component = lazyload(() => import(`./pages/${route.key}`));
    //       res.push(route);
    //     } else if (isArray(route.children) && route.children.length) {
    //       travel(route.children);
    //     }
    //   });
    // }
    // travel(routes);
    // You may be very surprised why it is written this way, so am I...
    function travel(_routes) {
        _routes.forEach((route)=>{
            if (route.key && !route.children) {
                if (route.key.includes('dashboard/monitor')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("2f452781"));
                } else if (route.key.includes('dashboard/workplace')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("349cd1ac"));
                } else if (route.key.includes('exception/403')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("d064950f"));
                } else if (route.key.includes('exception/404')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("9e276ce5"));
                } else if (route.key.includes('exception/500')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("37a7b57f"));
                } else if (route.key.includes('form/group')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("9afc8098"));
                } else if (route.key.includes('form/step')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("5e674e64"));
                } else if (route.key.includes('list/card')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("b5c07e81"));
                } else if (route.key.includes('list/search-table')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("542b0a82"));
                } else if (route.key.includes('profile/basic')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("beb95129"));
                } else if (route.key.includes('result/error')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("31dd8c8e"));
                } else if (route.key.includes('result/success')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("87644080"));
                } else if (route.key.includes('user/info')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("0e07cb09"));
                } else if (route.key.includes('user/setting')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("975f2322"));
                } else if (route.key.includes('visualization/data-analysis')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("69c6321e"));
                } else if (route.key.includes('visualization/multi-dimension-data-analysis')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("6570b5fa"));
                } else if (route.key.includes('welcome')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("89272fed"));
                } else if (route.key.includes('login')) {
                    route.component = (0, _lazyload.default)(()=>farmDynamicRequire("96ecd90f"));
                }
                res.push(route);
            } else if ((0, _is.isArray)(route.children) && route.children.length) {
                travel(route.children);
            }
        });
    }
    travel(routes);
    return res;
}
function PageLayout() {
    const urlParams = (0, _getUrlParams.default)();
    const history = (0, _reactrouterdom.useHistory)();
    const pathname = history.location.pathname;
    const currentComponent = _querystring.default.parseUrl(pathname).url.slice(1);
    const locale = (0, _useLocale.default)();
    const settings = (0, _reactredux.useSelector)((state)=>state.settings);
    const userInfo = (0, _reactredux.useSelector)((state)=>state.userInfo);
    const [routes, defaultRoute] = (0, _routes.default)(userInfo?.permissions);
    const defaultSelectedKeys = [
        currentComponent || defaultRoute
    ];
    const paths = (currentComponent || defaultRoute).split('/');
    const defaultOpenKeys = paths.slice(0, paths.length - 1);
    const [breadcrumb, setBreadCrumb] = (0, _react.useState)([]);
    const [collapsed, setCollapsed] = (0, _react.useState)(false);
    const [selectedKeys, setSelectedKeys] = (0, _react.useState)(defaultSelectedKeys);
    const [openKeys, setOpenKeys] = (0, _react.useState)(defaultOpenKeys);
    const routeMap = (0, _react.useRef)(new Map());
    const navbarHeight = 60;
    const menuWidth = collapsed ? 48 : settings.menuWidth;
    const showNavbar = settings.navbar && urlParams.navbar !== false;
    const showMenu = settings.menu && urlParams.menu !== false;
    const showFooter = settings.footer && urlParams.footer !== false;
    const flattenRoutes = (0, _react.useMemo)(()=>getFlattenRoutes(routes) || [], [
        routes
    ]);
    function renderRoutes(locale) {
        const nodes = [];
        routeMap.current.clear();
        function travel(_routes, level, parentNode = []) {
            return _routes.map((route)=>{
                const { breadcrumb = true } = route;
                const iconDom = getIconFromKey(route.key);
                const titleDom = _react.default.createElement(_react.default.Fragment, null, iconDom, " ", locale[route.name] || route.name);
                if (route.component && (!(0, _is.isArray)(route.children) || (0, _is.isArray)(route.children) && !route.children.length)) {
                    routeMap.current.set(`/${route.key}`, breadcrumb ? [
                        ...parentNode,
                        route.name
                    ] : []);
                    if (level > 1) {
                        return _react.default.createElement(MenuItem, {
                            key: route.key
                        }, titleDom);
                    }
                    nodes.push(_react.default.createElement(MenuItem, {
                        key: route.key
                    }, _react.default.createElement(_reactrouterdom.Link, {
                        to: `/${route.key}`
                    }, titleDom)));
                }
                if ((0, _is.isArray)(route.children) && route.children.length) {
                    const parentNode = [];
                    if (iconDom.props.isIcon) {
                        parentNode.push(iconDom);
                    }
                    if (level > 1) {
                        return _react.default.createElement(SubMenu, {
                            key: route.key,
                            title: titleDom
                        }, travel(route.children, level + 1, [
                            ...parentNode,
                            route.name
                        ]));
                    }
                    nodes.push(_react.default.createElement(SubMenu, {
                        key: route.key,
                        title: titleDom
                    }, travel(route.children, level + 1, [
                        ...parentNode,
                        route.name
                    ])));
                }
            });
        }
        travel(routes, 1);
        return nodes;
    }
    function onClickMenuItem(key) {
        const currentRoute = flattenRoutes.find((r)=>r.key === key);
        // const component = currentRoute.component;
        // const preload = component.preload;
        const preload = new Promise((resolve)=>{
            setTimeout(()=>{
                resolve(true);
            }, Math.random() * 500);
        });
        _nprogress.default.start();
        preload.then(()=>{
            setSelectedKeys([
                key
            ]);
            history.push(currentRoute.path ? currentRoute.path : `/${key}`);
            _nprogress.default.done();
        });
    }
    function toggleCollapse() {
        setCollapsed((collapsed)=>!collapsed);
    }
    const paddingLeft = showMenu ? {
        paddingLeft: menuWidth
    } : {};
    const paddingTop = showNavbar ? {
        paddingTop: navbarHeight
    } : {};
    const paddingStyle = {
        ...paddingLeft,
        ...paddingTop
    };
    (0, _react.useEffect)(()=>{
        const routeConfig = routeMap.current.get(pathname);
        setBreadCrumb(routeConfig || []);
    }, [
        pathname
    ]);
    return _react.default.createElement(_webreact.Layout, {
        className: _layoutmoduleless.default.layout
    }, _react.default.createElement("div", {
        className: (0, _classnames.default)(_layoutmoduleless.default['layout-navbar'], {
            [_layoutmoduleless.default['layout-navbar-hidden']]: !showNavbar
        })
    }, _react.default.createElement(_NavBar.default, {
        show: showNavbar
    })), _react.default.createElement(_webreact.Layout, null, showMenu && _react.default.createElement(Sider, {
        className: _layoutmoduleless.default['layout-sider'],
        width: menuWidth,
        collapsed: collapsed,
        onCollapse: setCollapsed,
        trigger: null,
        collapsible: true,
        breakpoint: "xl",
        style: paddingTop
    }, _react.default.createElement("div", {
        className: _layoutmoduleless.default['menu-wrapper']
    }, _react.default.createElement(_webreact.Menu, {
        collapse: collapsed,
        onClickMenuItem: onClickMenuItem,
        selectedKeys: selectedKeys,
        openKeys: openKeys,
        onClickSubMenu: (_, openKeys)=>setOpenKeys(openKeys)
    }, renderRoutes(locale))), _react.default.createElement("div", {
        className: _layoutmoduleless.default['collapse-btn'],
        onClick: toggleCollapse
    }, collapsed ? _react.default.createElement(_icon.IconMenuUnfold, null) : _react.default.createElement(_icon.IconMenuFold, null))), _react.default.createElement(_webreact.Layout, {
        className: _layoutmoduleless.default['layout-content'],
        style: paddingStyle
    }, _react.default.createElement("div", {
        className: _layoutmoduleless.default['layout-content-wrapper']
    }, !!breadcrumb.length && _react.default.createElement("div", {
        className: _layoutmoduleless.default['layout-breadcrumb']
    }, _react.default.createElement(_webreact.Breadcrumb, null, breadcrumb.map((node, index)=>_react.default.createElement(_webreact.Breadcrumb.Item, {
            key: index
        }, typeof node === 'string' ? locale[node] || node : node)))), _react.default.createElement(Content, null, _react.default.createElement(_reactrouterdom.Switch, null, flattenRoutes.map((route, index)=>{
        return _react.default.createElement(_reactrouterdom.Route, {
            key: index,
            path: `/${route.key}`,
            component: route.component
        });
    }), _react.default.createElement(_reactrouterdom.Route, {
        exact: true,
        path: "/"
    }, _react.default.createElement(_reactrouterdom.Redirect, {
        to: `/${defaultRoute}`
    })), _react.default.createElement(_reactrouterdom.Route, {
        path: "*",
        component: (0, _lazyload.default)(()=>farmDynamicRequire("d064950f"))
    })))), showFooter && _react.default.createElement(_Footer.default, null))));
}
const _default = PageLayout;

},
"1e670b54": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return Block;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _blockmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("2ec25f3d"));
function Block(props) {
    const { title, options, children } = props;
    const locale = (0, _useLocale.default)();
    const settings = (0, _reactredux.useSelector)((state)=>state.settings);
    const dispatch = (0, _reactredux.useDispatch)();
    return _react.default.createElement("div", {
        className: _blockmoduleless.default.block
    }, _react.default.createElement("h5", {
        className: _blockmoduleless.default.title
    }, title), options && options.map((option)=>{
        const type = option.type || 'switch';
        return _react.default.createElement("div", {
            className: _blockmoduleless.default['switch-wrapper'],
            key: option.value
        }, _react.default.createElement("span", null, locale[option.name]), type === 'switch' && _react.default.createElement(_webreact.Switch, {
            size: "small",
            checked: !!settings[option.value],
            onChange: (checked)=>{
                const newSetting = {
                    ...settings,
                    [option.value]: checked
                };
                dispatch({
                    type: 'update-settings',
                    payload: {
                        settings: newSetting
                    }
                });
                // set color week
                if (checked && option.value === 'colorWeek') {
                    document.body.style.filter = 'invert(80%)';
                }
                if (!checked && option.value === 'colorWeek') {
                    document.body.style.filter = 'none';
                }
            }
        }), type === 'number' && _react.default.createElement(_webreact.InputNumber, {
            style: {
                width: 80
            },
            size: "small",
            value: settings.menuWidth,
            onChange: (value)=>{
                const newSetting = {
                    ...settings,
                    [option.value]: value
                };
                dispatch({
                    type: 'update-settings',
                    payload: {
                        settings: newSetting
                    }
                });
            }
        }));
    }), children, _react.default.createElement(_webreact.Divider, null));
}

},
"229ffbcb": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "icon-button": `icon-button-6c957379`
};

},
"29b8f8b1": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _routes = farmRequire("11ff0679");
const _is = farmRequire("1cb2548c");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
if (!_is.isSSR) {
    _mockjs.default.XHR.prototype.withCredentials = true;
    (0, _setupMock.default)({
        setup: ()=>{
            // 用户信息
            const userRole = window.localStorage.getItem("userRole") || "admin";
            _mockjs.default.mock(new RegExp("/api/user/userInfo"), ()=>{
                return _mockjs.default.mock({
                    name: "王立群",
                    avatar: "https://lf1-xgcdn-tos.pstatp.com/obj/vcloud/vadmin/start.8e0e4855ee346a46ccff8ff3e24db27b.png",
                    email: "wangliqun@email.com",
                    job: "frontend",
                    jobName: "前端开发工程师",
                    organization: "Frontend",
                    organizationName: "前端",
                    location: "beijing",
                    locationName: "北京",
                    introduction: "王力群并非是一个真实存在的人。",
                    personalWebsite: "https://www.arco.design",
                    verified: true,
                    phoneNumber: /177[*]{6}[0-9]{2}/,
                    accountId: /[a-z]{4}[-][0-9]{8}/,
                    registrationTime: _mockjs.default.Random.datetime("yyyy-MM-dd HH:mm:ss"),
                    permissions: (0, _routes.generatePermission)(userRole)
                });
            });
            // 登录
            _mockjs.default.mock(new RegExp("/api/user/login"), (params)=>{
                const { userName, password } = JSON.parse(params.body);
                if (!userName) {
                    return {
                        status: "error",
                        msg: "用户名不能为空"
                    };
                }
                if (!password) {
                    return {
                        status: "error",
                        msg: "密码不能为空"
                    };
                }
                if (userName === "admin" && password === "admin") {
                    return {
                        status: "ok"
                    };
                }
                return {
                    status: "error",
                    msg: "账号或者密码错误"
                };
            });
        }
    });
}

},
"2c6e69f1": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
"";
const _webreact = farmRequire("050d455e");
const _enUS = /*#__PURE__*/ _interop_require_default._(farmRequire("2c1abce8"));
const _zhCN = /*#__PURE__*/ _interop_require_default._(farmRequire("7b216cda"));
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactdom = /*#__PURE__*/ _interop_require_default._(farmRequire("3d501ffc"));
const _reactredux = farmRequire("e429bf23");
const _reactrouterdom = farmRequire("120e6d2c");
const _redux = farmRequire("ca993b72");
const _context = farmRequire("64285704");
const _layout = /*#__PURE__*/ _interop_require_default._(farmRequire("16203e8b"));
farmRequire("0526fb72");
const _login = /*#__PURE__*/ _interop_require_default._(farmRequire("96ecd90f"));
const _store = /*#__PURE__*/ _interop_require_default._(farmRequire("d49232d1"));
const _changeTheme = /*#__PURE__*/ _interop_require_default._(farmRequire("e84e37e0"));
const _checkLogin = /*#__PURE__*/ _interop_require_default._(farmRequire("8c0e7ffa"));
const _useStorage = /*#__PURE__*/ _interop_require_default._(farmRequire("40a43159"));
const store = (0, _redux.createStore)(_store.default);
function Index() {
    const [lang, setLang] = (0, _useStorage.default)('arco-lang', 'en-US');
    const [theme, setTheme] = (0, _useStorage.default)('arco-theme', 'light');
    function getArcoLocale() {
        switch(lang){
            case 'zh-CN':
                return _zhCN.default;
            case 'en-US':
                return _enUS.default;
            default:
                return _zhCN.default;
        }
    }
    function fetchUserInfo() {
        _axios.default.get('/api/user/userInfo').then((res)=>{
            store.dispatch({
                type: 'update-userInfo',
                payload: {
                    userInfo: res.data
                }
            });
        });
    }
    (0, _react.useEffect)(()=>{
        if ((0, _checkLogin.default)()) {
            fetchUserInfo();
        } else if (window.location.pathname.replace(/\//g, '') !== 'login') {
            window.location.pathname = '/login';
        }
    }, []);
    (0, _react.useEffect)(()=>{
        (0, _changeTheme.default)(theme);
    }, [
        theme
    ]);
    const contextValue = {
        lang,
        setLang,
        theme,
        setTheme
    };
    return _react.default.createElement(_reactrouterdom.BrowserRouter, null, _react.default.createElement(_react.Suspense, {
        fallback: _react.default.createElement("div", null, "loading....")
    }, _react.default.createElement(_webreact.ConfigProvider, {
        locale: getArcoLocale(),
        componentConfig: {
            Card: {
                bordered: false
            },
            List: {
                bordered: false
            },
            Table: {
                border: false
            }
        }
    }, _react.default.createElement(_reactredux.Provider, {
        store: store
    }, _react.default.createElement(_context.GlobalContext.Provider, {
        value: contextValue
    }, _react.default.createElement(_reactrouterdom.Switch, null, _react.default.createElement(_reactrouterdom.Route, {
        path: "/login",
        component: _login.default
    }), _react.default.createElement(_reactrouterdom.Route, {
        path: "/",
        component: _layout.default
    })))))));
}
_reactdom.default.render(_react.default.createElement(Index, null), document.getElementById('root'));

},
"2ec25f3d": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "block": `block-60cb4b95`,
    "switch-wrapper": `switch-wrapper-60cb4b95`,
    "title": `title-60cb4b95`
};

},
"43649516": function(module, exports, farmRequire, farmDynamicRequire) {
module.exports = {
    "colorWeek": false,
    "navbar": true,
    "menu": true,
    "footer": true,
    "themeColor": "#165DFF",
    "menuWidth": 220
};

},
"65422662": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _logosvg = /*#__PURE__*/ _interop_require_default._(farmRequire("c83e07be"));
const _MessageBox = /*#__PURE__*/ _interop_require_default._(farmRequire("a07ab083"));
const _context = farmRequire("64285704");
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("c9845146"));
const _routes = farmRequire("11ff0679");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _useStorage = /*#__PURE__*/ _interop_require_default._(farmRequire("40a43159"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _Settings = /*#__PURE__*/ _interop_require_default._(farmRequire("0f9ad454"));
const _IconButton = /*#__PURE__*/ _interop_require_default._(farmRequire("b04795b8"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("06e27f5c"));
function Navbar({ show }) {
    const t = (0, _useLocale.default)();
    const userInfo = (0, _reactredux.useSelector)((state)=>state.userInfo);
    const dispatch = (0, _reactredux.useDispatch)();
    const [_, setUserStatus] = (0, _useStorage.default)('userStatus');
    const [role, setRole] = (0, _useStorage.default)('userRole', 'admin');
    const { setLang, lang, theme, setTheme } = (0, _react.useContext)(_context.GlobalContext);
    function logout() {
        setUserStatus('logout');
        window.location.href = '/login';
    }
    function onMenuItemClick(key) {
        if (key === 'logout') {
            logout();
        } else {
            _webreact.Message.info(`You clicked ${key}`);
        }
    }
    (0, _react.useEffect)(()=>{
        dispatch({
            type: 'update-userInfo',
            payload: {
                userInfo: {
                    ...userInfo,
                    permissions: (0, _routes.generatePermission)(role)
                }
            }
        });
    }, [
        role
    ]);
    if (!show) {
        return _react.default.createElement("div", {
            className: _indexmoduleless.default['fixed-settings']
        }, _react.default.createElement(_Settings.default, {
            trigger: _react.default.createElement(_webreact.Button, {
                icon: _react.default.createElement(_icon.IconSettings, null),
                type: "primary",
                size: "large"
            })
        }));
    }
    const handleChangeRole = ()=>{
        const newRole = role === 'admin' ? 'user' : 'admin';
        setRole(newRole);
    };
    const droplist = _react.default.createElement(_webreact.Menu, {
        onClickMenuItem: onMenuItemClick
    }, _react.default.createElement(_webreact.Menu.SubMenu, {
        key: "role",
        title: _react.default.createElement(_react.default.Fragment, null, _react.default.createElement(_icon.IconUser, {
            className: _indexmoduleless.default['dropdown-icon']
        }), _react.default.createElement("span", {
            className: _indexmoduleless.default['user-role']
        }, role === 'admin' ? t['menu.user.role.admin'] : t['menu.user.role.user']))
    }, _react.default.createElement(_webreact.Menu.Item, {
        onClick: handleChangeRole,
        key: "switch role"
    }, _react.default.createElement(_icon.IconTag, {
        className: _indexmoduleless.default['dropdown-icon']
    }), t['menu.user.switchRoles'])), _react.default.createElement(_webreact.Menu.Item, {
        key: "setting"
    }, _react.default.createElement(_icon.IconSettings, {
        className: _indexmoduleless.default['dropdown-icon']
    }), t['menu.user.setting']), _react.default.createElement(_webreact.Menu.SubMenu, {
        key: "more",
        title: _react.default.createElement("div", {
            style: {
                width: 80
            }
        }, _react.default.createElement(_icon.IconExperiment, {
            className: _indexmoduleless.default['dropdown-icon']
        }), t['message.seeMore'])
    }, _react.default.createElement(_webreact.Menu.Item, {
        key: "workplace"
    }, _react.default.createElement(_icon.IconDashboard, {
        className: _indexmoduleless.default['dropdown-icon']
    }), t['menu.dashboard.workplace']), _react.default.createElement(_webreact.Menu.Item, {
        key: "card list"
    }, _react.default.createElement(_icon.IconInteraction, {
        className: _indexmoduleless.default['dropdown-icon']
    }), t['menu.list.cardList'])), _react.default.createElement(_webreact.Divider, {
        style: {
            margin: '4px 0'
        }
    }), _react.default.createElement(_webreact.Menu.Item, {
        key: "logout"
    }, _react.default.createElement(_icon.IconPoweroff, {
        className: _indexmoduleless.default['dropdown-icon']
    }), t['navbar.logout']));
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.navbar
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default.left
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default.logo
    }, _react.default.createElement(_logosvg.default, null), _react.default.createElement("div", {
        className: _indexmoduleless.default['logo-name']
    }, "Arco Pro"))), _react.default.createElement("ul", {
        className: _indexmoduleless.default.right
    }, _react.default.createElement("li", null, _react.default.createElement(_webreact.Input.Search, {
        className: _indexmoduleless.default.round,
        placeholder: t['navbar.search.placeholder']
    })), _react.default.createElement("li", null, _react.default.createElement(_webreact.Select, {
        triggerElement: _react.default.createElement(_IconButton.default, {
            icon: _react.default.createElement(_icon.IconLanguage, null)
        }),
        options: [
            {
                label: '中文',
                value: 'zh-CN'
            },
            {
                label: 'English',
                value: 'en-US'
            }
        ],
        value: lang,
        triggerProps: {
            autoAlignPopupWidth: false,
            autoAlignPopupMinWidth: true,
            position: 'br'
        },
        trigger: "hover",
        onChange: (value)=>{
            setLang(value);
            const nextLang = _locale.default[value];
            _webreact.Message.info(`${nextLang['message.lang.tips']}${value}`);
        }
    })), _react.default.createElement("li", null, _react.default.createElement(_MessageBox.default, null, _react.default.createElement(_IconButton.default, {
        icon: _react.default.createElement(_icon.IconNotification, null)
    }))), _react.default.createElement("li", null, _react.default.createElement(_webreact.Tooltip, {
        content: theme === 'light' ? t['settings.navbar.theme.toDark'] : t['settings.navbar.theme.toLight']
    }, _react.default.createElement(_IconButton.default, {
        icon: theme !== 'dark' ? _react.default.createElement(_icon.IconMoonFill, null) : _react.default.createElement(_icon.IconSunFill, null),
        onClick: ()=>setTheme(theme === 'light' ? 'dark' : 'light')
    }))), _react.default.createElement(_Settings.default, null), userInfo && _react.default.createElement("li", null, _react.default.createElement(_webreact.Dropdown, {
        droplist: droplist,
        position: "br"
    }, _react.default.createElement(_webreact.Avatar, {
        size: 32,
        style: {
            cursor: 'pointer'
        }
    }, _react.default.createElement("img", {
        alt: "avatar",
        src: userInfo.avatar
    }))))));
}
const _default = Navbar;

},
"7b631473": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _component = /*#__PURE__*/ _interop_require_default._(farmRequire("87119331"));
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _layoutmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("c4691837"));
// https://github.com/gregberge/loadable-components/pull/226
function load(fn, options) {
    const Component = (0, _component.default)(fn, options);
    Component.preload = fn.requireAsync || fn;
    return Component;
}
function LoadingComponent(props) {
    if (props.error) {
        console.error(props.error);
        return null;
    }
    return _react.default.createElement("div", {
        className: _layoutmoduleless.default.spin
    }, _react.default.createElement(_webreact.Spin, null));
}
const _default = (loader)=>load(loader, {
        fallback: LoadingComponent({
            pastDelay: true,
            error: false,
            timedOut: false
        })
    });

},
"8c0e7ffa": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return checkLogin;
    }
});
function checkLogin() {
    return localStorage.getItem('userStatus') === 'login';
}

},
"a07ab083": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _groupBy = /*#__PURE__*/ _interop_require_default._(farmRequire("0a59b4ed"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _list = /*#__PURE__*/ _interop_require_default._(farmRequire("e0798eff"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("dca11268"));
function DropContent() {
    const t = (0, _useLocale.default)();
    const [loading, setLoading] = (0, _react.useState)(false);
    const [groupData, setGroupData] = (0, _react.useState)({});
    const [sourceData, setSourceData] = (0, _react.useState)([]);
    function fetchSourceData(showLoading = true) {
        showLoading && setLoading(true);
        _axios.default.get('/api/message/list').then((res)=>{
            setSourceData(res.data);
        }).finally(()=>{
            showLoading && setLoading(false);
        });
    }
    function readMessage(data) {
        const ids = data.map((item)=>item.id);
        _axios.default.post('/api/message/read', {
            ids
        }).then(()=>{
            fetchSourceData();
        });
    }
    (0, _react.useEffect)(()=>{
        fetchSourceData();
    }, []);
    (0, _react.useEffect)(()=>{
        const groupData = (0, _groupBy.default)(sourceData, 'type');
        setGroupData(groupData);
    }, [
        sourceData
    ]);
    const tabList = [
        {
            key: 'message',
            title: t['message.tab.title.message'],
            titleIcon: _react.default.createElement(_icon.IconMessage, null)
        },
        {
            key: 'notice',
            title: t['message.tab.title.notice'],
            titleIcon: _react.default.createElement(_icon.IconCustomerService, null)
        },
        {
            key: 'todo',
            title: t['message.tab.title.todo'],
            titleIcon: _react.default.createElement(_icon.IconFile, null),
            avatar: _react.default.createElement(_webreact.Avatar, {
                style: {
                    backgroundColor: '#0FC6C2'
                }
            }, _react.default.createElement(_icon.IconDesktop, null))
        }
    ];
    return _react.default.createElement("div", {
        className: _indexmoduleless.default['message-box']
    }, _react.default.createElement(_webreact.Spin, {
        loading: loading,
        style: {
            display: 'block'
        }
    }, _react.default.createElement(_webreact.Tabs, {
        overflow: "dropdown",
        type: "rounded",
        defaultActiveTab: "message",
        destroyOnHide: true,
        extra: _react.default.createElement(_webreact.Button, {
            type: "text",
            onClick: ()=>setSourceData([])
        }, t['message.empty'])
    }, tabList.map((item)=>{
        const { key, title, avatar } = item;
        const data = groupData[key] || [];
        const unReadData = data.filter((item)=>!item.status);
        return _react.default.createElement(_webreact.Tabs.TabPane, {
            key: key,
            title: _react.default.createElement("span", null, title, unReadData.length ? `(${unReadData.length})` : '')
        }, _react.default.createElement(_list.default, {
            data: data,
            unReadData: unReadData,
            onItemClick: (item)=>{
                readMessage([
                    item
                ]);
            },
            onAllBtnClick: (unReadData)=>{
                readMessage(unReadData);
            }
        }));
    }))));
}
function MessageBox({ children }) {
    return _react.default.createElement(_webreact.Trigger, {
        trigger: "hover",
        popup: ()=>_react.default.createElement(DropContent, null),
        position: "br",
        unmountOnExit: false,
        popupAlign: {
            bottom: 4
        }
    }, _react.default.createElement(_webreact.Badge, {
        count: 9,
        dot: true
    }, children));
}
const _default = MessageBox;

},
"abbbd081": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _color = farmRequire("fe78f3a1");
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactcolor = farmRequire("0ff00a65");
const _reactredux = farmRequire("e429bf23");
const _colorpanelmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("e03623c0"));
function ColorPanel() {
    const theme = document.querySelector('body').getAttribute('arco-theme') || 'light';
    const settings = (0, _reactredux.useSelector)((state)=>state.settings);
    const locale = (0, _useLocale.default)();
    const themeColor = settings.themeColor;
    const list = (0, _color.generate)(themeColor, {
        list: true
    });
    const dispatch = (0, _reactredux.useDispatch)();
    return _react.default.createElement("div", null, _react.default.createElement(_webreact.Trigger, {
        trigger: "hover",
        position: "bl",
        popup: ()=>_react.default.createElement(_reactcolor.SketchPicker, {
                color: themeColor,
                onChangeComplete: (color)=>{
                    const newColor = color.hex;
                    dispatch({
                        type: 'update-settings',
                        payload: {
                            settings: {
                                ...settings,
                                themeColor: newColor
                            }
                        }
                    });
                    const newList = (0, _color.generate)(newColor, {
                        list: true,
                        dark: theme === 'dark'
                    });
                    newList.forEach((l, index)=>{
                        const rgbStr = (0, _color.getRgbStr)(l);
                        document.body.style.setProperty(`--arcoblue-${index + 1}`, rgbStr);
                    });
                }
            })
    }, _react.default.createElement("div", {
        className: _colorpanelmoduleless.default.input
    }, _react.default.createElement("div", {
        className: _colorpanelmoduleless.default.color,
        style: {
            backgroundColor: themeColor
        }
    }), _react.default.createElement("span", null, themeColor))), _react.default.createElement("ul", {
        className: _colorpanelmoduleless.default.ul
    }, list.map((item, index)=>_react.default.createElement("li", {
            key: index,
            className: _colorpanelmoduleless.default.li,
            style: {
                backgroundColor: item
            }
        }))), _react.default.createElement(_webreact.Typography.Paragraph, {
        style: {
            fontSize: 12
        }
    }, locale['settings.color.tooltip']));
}
const _default = ColorPanel;

},
"ac8842a4": function(module, exports, farmRequire, farmDynamicRequire) {
// 仅用于线上预览，实际使用中可以将此逻辑删除
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return getUrlParams;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _querystring = /*#__PURE__*/ _interop_require_default._(farmRequire("dfa4e0d3"));
const _is = farmRequire("1cb2548c");
function getUrlParams() {
    const params = _querystring.default.parseUrl(!_is.isSSR ? window.location.href : "").query;
    const returnParams = {};
    Object.keys(params).forEach((p)=>{
        if (params[p] === "true") {
            returnParams[p] = true;
        }
        if (params[p] === "false") {
            returnParams[p] = false;
        }
    });
    return returnParams;
}

},
"b04795b8": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _webreact = farmRequire("050d455e");
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _iconbuttonmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("229ffbcb"));
function IconButton(props, ref) {
    const { icon, className, ...rest } = props;
    return _react.default.createElement(_webreact.Button, {
        ref: ref,
        icon: icon,
        shape: "circle",
        type: "secondary",
        className: (0, _classnames.default)(_iconbuttonmoduleless.default['icon-button'], className),
        ...rest
    });
}
const _default = (0, _react.forwardRef)(IconButton);

},
"c4691837": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "collapse-btn": `collapse-btn-d620026e`,
    "icon": `icon-d620026e`,
    "icon-empty": `icon-empty-d620026e`,
    "layout": `layout-d620026e`,
    "layout-breadcrumb": `layout-breadcrumb-d620026e`,
    "layout-content": `layout-content-d620026e`,
    "layout-content-wrapper": `layout-content-wrapper-d620026e`,
    "layout-navbar": `layout-navbar-d620026e`,
    "layout-navbar-hidden": `layout-navbar-hidden-d620026e`,
    "layout-sider": `layout-sider-d620026e`,
    "menu-wrapper": `menu-wrapper-d620026e`,
    "spin": `spin-d620026e`
};

},
"d49232d1": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return store;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _settingsjson = /*#__PURE__*/ _interop_require_default._(farmRequire("43649516"));
const initialState = {
    settings: _settingsjson.default,
    userInfo: {
        permissions: {}
    }
};
function store(state = initialState, action) {
    switch(action.type){
        case "update-settings":
            {
                const { settings } = action.payload;
                return {
                    ...state,
                    settings
                };
            }
        case "update-userInfo":
            {
                const { userInfo = initialState.userInfo, userLoading } = action.payload;
                return {
                    ...state,
                    userLoading,
                    userInfo
                };
            }
        default:
            return state;
    }
}

},
"dca11268": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "footer": `footer-ce788f3f`,
    "footer-item": `footer-item-ce788f3f`,
    "message-box": `message-box-ce788f3f`,
    "message-title": `message-title-ce788f3f`
};

},
"e03623c0": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "color": `color-930661ac`,
    "input": `input-930661ac`,
    "li": `li-930661ac`,
    "ul": `ul-930661ac`
};

},
"e0798eff": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("dca11268"));
function MessageList(props) {
    const t = (0, _useLocale.default)();
    const { data, unReadData } = props;
    function onItemClick(item, index) {
        if (item.status) return;
        props.onItemClick && props.onItemClick(item, index);
    }
    function onAllBtnClick() {
        props.onAllBtnClick && props.onAllBtnClick(unReadData, data);
    }
    return _react.default.createElement(_webreact.List, {
        noDataElement: _react.default.createElement(_webreact.Result, {
            status: "404",
            subTitle: t['message.empty.tips']
        }),
        footer: _react.default.createElement("div", {
            className: _indexmoduleless.default.footer
        }, _react.default.createElement("div", {
            className: _indexmoduleless.default['footer-item']
        }, _react.default.createElement(_webreact.Button, {
            type: "text",
            size: "small",
            onClick: onAllBtnClick
        }, t['message.allRead'])), _react.default.createElement("div", {
            className: _indexmoduleless.default['footer-item']
        }, _react.default.createElement(_webreact.Button, {
            type: "text",
            size: "small"
        }, t['message.seeMore'])))
    }, data.map((item, index)=>_react.default.createElement(_webreact.List.Item, {
            key: item.id,
            actionLayout: "vertical",
            style: {
                opacity: item.status ? 0.5 : 1
            }
        }, _react.default.createElement("div", {
            style: {
                cursor: 'pointer'
            },
            onClick: ()=>{
                onItemClick(item, index);
            }
        }, _react.default.createElement(_webreact.List.Item.Meta, {
            avatar: item.avatar && _react.default.createElement(_webreact.Avatar, {
                shape: "circle",
                size: 36
            }, _react.default.createElement("img", {
                src: item.avatar
            })),
            title: _react.default.createElement("div", {
                className: _indexmoduleless.default['message-title']
            }, _react.default.createElement(_webreact.Space, {
                size: 4
            }, _react.default.createElement("span", null, item.title), _react.default.createElement(_webreact.Typography.Text, {
                type: "secondary"
            }, item.subTitle)), item.tag && item.tag.text ? _react.default.createElement(_webreact.Tag, {
                color: item.tag.color
            }, item.tag.text) : null),
            description: _react.default.createElement("div", null, _react.default.createElement(_webreact.Typography.Paragraph, {
                style: {
                    marginBottom: 0
                },
                ellipsis: true
            }, item.content), _react.default.createElement(_webreact.Typography.Text, {
                type: "secondary",
                style: {
                    fontSize: 12
                }
            }, item.time))
        })))));
}
const _default = MessageList;

},
"e84e37e0": function(module, exports, farmRequire, farmDynamicRequire) {
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
function changeTheme(theme) {
    if (theme === "dark") {
        document.body.setAttribute("arco-theme", "dark");
    } else {
        document.body.removeAttribute("arco-theme");
    }
}
const _default = changeTheme;

},});