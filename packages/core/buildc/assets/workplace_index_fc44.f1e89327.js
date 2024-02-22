(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'workplace_index_fc44.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"0e2c31dd": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "chart-sub-title": `chart-sub-title-034c3371`,
    "chart-title": `chart-title-034c3371`,
    "container": `container-034c3371`,
    "count": `count-034c3371`,
    "ctw": `ctw-034c3371`,
    "divider": `divider-034c3371`,
    "icon": `icon-034c3371`,
    "item": `item-034c3371`,
    "title": `title-034c3371`,
    "unit": `unit-034c3371`
};

},
"1a79ea92": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("3eca280b"));
const _announcementmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("6c8f33c0"));
function Announcement() {
    const [data, setData] = (0, _react.useState)([]);
    const [loading, setLoading] = (0, _react.useState)(true);
    const t = (0, _useLocale.default)(_locale.default);
    const fetchData = ()=>{
        setLoading(true);
        _axios.default.get('/api/workplace/announcement').then((res)=>{
            setData(res.data);
        }).finally(()=>{
            setLoading(false);
        });
    };
    (0, _react.useEffect)(()=>{
        fetchData();
    }, []);
    function getTagColor(type) {
        switch(type){
            case 'activity':
                return 'orangered';
            case 'info':
                return 'cyan';
            case 'notice':
                return 'arcoblue';
            default:
                return 'arcoblue';
        }
    }
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement("div", {
        style: {
            display: 'flex',
            justifyContent: 'space-between'
        }
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['workplace.announcement']), _react.default.createElement(_webreact.Link, null, t['workplace.seeMore'])), _react.default.createElement(_webreact.Skeleton, {
        loading: loading,
        text: {
            rows: 5,
            width: '100%'
        },
        animation: true
    }, _react.default.createElement("div", null, [].map((d)=>_react.default.createElement("div", {
            key: d.key,
            className: _announcementmoduleless.default.item
        }, _react.default.createElement(_webreact.Tag, {
            color: getTagColor(d.type),
            size: "small"
        }, t[`workplace.${d.type}`]), _react.default.createElement("span", {
            className: _announcementmoduleless.default.link
        }, d.content))))));
}
const _default = Announcement;

},
"349cd1ac": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _announcement = /*#__PURE__*/ _interop_require_default._(farmRequire("1a79ea92"));
const _carousel = /*#__PURE__*/ _interop_require_default._(farmRequire("c25c1505"));
const _contentpercentage = /*#__PURE__*/ _interop_require_default._(farmRequire("5f303d81"));
const _docs = /*#__PURE__*/ _interop_require_default._(farmRequire("4b7748f8"));
farmRequire("956f3d7a");
const _overview = /*#__PURE__*/ _interop_require_default._(farmRequire("82aa9da7"));
const _popularcontents = /*#__PURE__*/ _interop_require_default._(farmRequire("218f3f24"));
const _shortcuts = /*#__PURE__*/ _interop_require_default._(farmRequire("cc60be79"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("bf79862c"));
const { Row, Col } = _webreact.Grid;
const gutter = 16;
function Workplace() {
    return _react.default.createElement(_webreact.Space, {
        size: 16,
        align: "start"
    }, _react.default.createElement(_webreact.Space, {
        size: 16,
        direction: "vertical"
    }, _react.default.createElement(_overview.default, null), _react.default.createElement(Row, {
        gutter: gutter
    }, _react.default.createElement(Col, {
        span: 12
    }, _react.default.createElement(_popularcontents.default, null)), _react.default.createElement(Col, {
        span: 12
    }, _react.default.createElement(_contentpercentage.default, null)))), _react.default.createElement(_webreact.Space, {
        className: _indexmoduleless.default.right,
        size: 16,
        direction: "vertical"
    }, _react.default.createElement(_shortcuts.default, null), _react.default.createElement(_carousel.default, null), _react.default.createElement(_announcement.default, null), _react.default.createElement(_docs.default, null)));
}
const _default = Workplace;

},
"4b7748f8": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("3eca280b"));
const _docsmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("d6b480cf"));
const links = {
    react: 'https://arco.design/react/docs/start',
    vue: 'https://arco.design/vue/docs/start',
    designLab: 'https://arco.design/themes',
    materialMarket: 'https://arco.design/material/'
};
function QuickOperation() {
    const t = (0, _useLocale.default)(_locale.default);
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement("div", {
        style: {
            display: 'flex',
            justifyContent: 'space-between'
        }
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['workplace.docs']), _react.default.createElement(_webreact.Link, null, t['workplace.seeMore'])), _react.default.createElement("div", {
        className: _docsmoduleless.default.docs
    }, Object.entries(links).map(([key, value])=>_react.default.createElement(_webreact.Link, {
            className: _docsmoduleless.default.link,
            key: key,
            href: value,
            target: "_blank"
        }, t[`workplace.${key}`]))));
}
const _default = QuickOperation;

},
"6c8f33c0": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "item": `item-4b7a95bc`,
    "link": `link-4b7a95bc`
};

},
"982ae7d7": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "symbol": `symbol-93380045`
};

},
"bf79862c": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "banner": `banner-2b893121`,
    "content": `content-2b893121`,
    "panel": `panel-2b893121`,
    "right": `right-2b893121`
};

},
"c25c1505": function(module, exports, farmRequire, farmDynamicRequire) {
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
const imageSrc = [
    '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/f7e8fc1e09c42e30682526252365be1c.jpg~tplv-uwbnlip3yd-webp.webp',
    '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/94e8dd2d6dc4efb2c8cfd82c0ff02a2c.jpg~tplv-uwbnlip3yd-webp.webp',
    '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/ec447228c59ae1ebe185bab6cd776ca4.jpg~tplv-uwbnlip3yd-webp.webp',
    '//p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/1d1580d2a5a1e27415ff594c756eabd8.jpg~tplv-uwbnlip3yd-webp.webp'
];
function C() {
    return _react.default.createElement(_webreact.Carousel, {
        indicatorType: "slider",
        showArrow: "never",
        autoPlay: true,
        style: {
            width: '100%',
            height: 160
        }
    }, imageSrc.map((src, index)=>_react.default.createElement("div", {
            key: index
        }, _react.default.createElement("img", {
            src: src,
            style: {
                width: 280,
                transform: 'translateY(-30px)'
            }
        }))));
}
const _default = C;

},
"d6b480cf": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "docs": `docs-e10e9c49`,
    "link": `link-e10e9c49`
};

},
"f3922c4a": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "icon": `icon-746ded81`,
    "item": `item-746ded81`,
    "recent": `recent-746ded81`,
    "shortcuts": `shortcuts-746ded81`,
    "title": `title-746ded81`
};

},});