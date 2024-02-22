(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'card_index_67c2.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"35c3ed68": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "add-card": `add-card-3cdf39bf`,
    "add-icon": `add-icon-3cdf39bf`,
    "card-block": `card-block-3cdf39bf`,
    "card-block-skeleton": `card-block-skeleton-3cdf39bf`,
    "card-content": `card-content-3cdf39bf`,
    "container": `container-3cdf39bf`,
    "content": `content-3cdf39bf`,
    "description": `description-3cdf39bf`,
    "extra": `extra-3cdf39bf`,
    "icon": `icon-3cdf39bf`,
    "more": `more-3cdf39bf`,
    "rules-card": `rules-card-3cdf39bf`,
    "service-card": `service-card-3cdf39bf`,
    "single-content": `single-content-3cdf39bf`,
    "status": `status-3cdf39bf`,
    "time": `time-3cdf39bf`,
    "title": `title-3cdf39bf`,
    "title-more": `title-more-3cdf39bf`
};

},
"72edb2f7": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "menu.list": "List",
        "menu.list.card": "Card List",
        "cardList.tab.title.all": "All",
        "cardList.tab.title.quality": "Content quality",
        "cardList.tab.title.service": "Service opening",
        "cardList.tab.title.rules": "Rule presets",
        "cardList.tab.all.placeholder": "Search",
        "cardList.tab.quality.placeholder": "Search queue",
        "cardList.tab.service.placeholder": "Search service",
        "cardList.tab.rules.placeholder": "Search rule",
        "cardList.searchInput.placeholder": "Search service",
        "cardList.add.quality": "Create quality inspection queue",
        "cardList.enable": "Enable",
        "cardList.disable": "Disable",
        "cardList.action": "action",
        "cardList.detail": "Detail",
        "cardList.tab.title.announcement": "Recent Announcement",
        "cardList.announcement.noData": "No announcement",
        "cardList.statistic.enable": "Enable",
        "cardList.statistic.disable": "Disable",
        "cardList.statistic.applicationNum": "Applications",
        "cardList.options.qualityInspection": "Quality inspection",
        "cardList.options.remove": "Remove",
        "cardList.options.cancel": "Cancel",
        "cardList.options.subscribe": "Subscribe",
        "cardList.options.renewal": "Renewal",
        "cardList.tag.activated": "Activated",
        "cardList.tag.opened": "Already Opened",
        "cardList.tag.expired": "Expired"
    },
    "zh-CN": {
        "menu.list": "列表页",
        "menu.list.card": "卡片列表",
        "cardList.tab.title.all": "全部",
        "cardList.tab.title.quality": "内容质检",
        "cardList.tab.title.service": "服务开通",
        "cardList.tab.title.rules": "规则预置",
        "cardList.tab.all.placeholder": "搜索",
        "cardList.tab.quality.placeholder": "搜索队列",
        "cardList.tab.service.placeholder": "搜索服务",
        "cardList.tab.rules.placeholder": "搜索规则",
        "cardList.searchInput.placeholder": "搜索服务",
        "cardList.add.quality": "点击创建质检内容队列",
        "cardList.enable": "启用",
        "cardList.disable": "禁用",
        "cardList.action": "操作",
        "cardList.detail": "详细信息",
        "cardList.tab.title.announcement": "最近公告",
        "cardList.announcement.noData": "暂无公告",
        "cardList.statistic.enable": "已启用",
        "cardList.statistic.disable": "未启用",
        "cardList.statistic.applicationNum": "应用数",
        "cardList.options.qualityInspection": "质检",
        "cardList.options.remove": "删除",
        "cardList.options.cancel": "取消开通",
        "cardList.options.subscribe": "开通服务",
        "cardList.options.renewal": "续约服务",
        "cardList.tag.activated": "已启用",
        "cardList.tag.opened": "已开通",
        "cardList.tag.expired": "已过期"
    }
};
const _default = i18n;

},
"e576be20": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _icon = farmRequire("f988cd7d");
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("35c3ed68"));
function AddCard(props) {
    return _react.default.createElement(_webreact.Card, {
        className: (0, _classnames.default)(_indexmoduleless.default['card-block'], _indexmoduleless.default['add-card']),
        title: null,
        bordered: true,
        size: "small"
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default.content
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['add-icon']
    }, _react.default.createElement(_icon.IconPlus, null)), _react.default.createElement("div", {
        className: _indexmoduleless.default.description
    }, props.description)));
}
const _default = AddCard;

},});