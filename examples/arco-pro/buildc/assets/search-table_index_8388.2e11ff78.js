(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'search-table_index_8388.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"2b1be6bc": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const SvgVertical = (props)=>_react.createElement("svg", {
        width: 13,
        height: 16,
        viewBox: "0 0 13 16",
        fill: "none",
        xmlns: "http://www.w3.org/2000/svg",
        ...props
    }, _react.createElement("rect", {
        opacity: 0.9,
        width: 13,
        height: 16,
        rx: 1.67,
        fill: "url(#paint0_linear_422_41748)"
    }), _react.createElement("g", {
        opacity: 0.9,
        filter: "url(#filter0_d_422_41748)"
    }, _react.createElement("path", {
        d: "M5 7.91745V5.08255C5 4.61129 5.51837 4.32398 5.918 4.57375L8.18592 5.9912C8.56192 6.2262 8.56192 6.7738 8.18592 7.0088L5.918 8.42625C5.51837 8.67602 5 8.38871 5 7.91745Z",
        fill: "white"
    })), _react.createElement("rect", {
        opacity: 0.8,
        width: 9,
        height: 1,
        rx: 0.315789,
        transform: "matrix(1 0 0 -1 2 12)",
        fill: "#FFF5E8"
    }), _react.createElement("rect", {
        opacity: 0.8,
        width: 6,
        height: 1,
        rx: 0.315789,
        transform: "matrix(1 0 0 -1 2 14)",
        fill: "#FFF5E8"
    }), _react.createElement("defs", null, _react.createElement("filter", {
        id: "filter0_d_422_41748",
        x: 3.73684,
        y: 3.21853,
        width: 5.99424,
        height: 6.56294,
        filterUnits: "userSpaceOnUse",
        colorInterpolationFilters: "sRGB"
    }, _react.createElement("feFlood", {
        floodOpacity: 0,
        result: "BackgroundImageFix"
    }), _react.createElement("feColorMatrix", {
        in: "SourceAlpha",
        type: "matrix",
        values: "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0",
        result: "hardAlpha"
    }), _react.createElement("feOffset", null), _react.createElement("feGaussianBlur", {
        stdDeviation: 0.631579
    }), _react.createElement("feComposite", {
        in2: "hardAlpha",
        operator: "out"
    }), _react.createElement("feColorMatrix", {
        type: "matrix",
        values: "0 0 0 0 0.870833 0 0 0 0 0.554311 0 0 0 0 0.148767 0 0 0 0.8 0"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in2: "BackgroundImageFix",
        result: "effect1_dropShadow_422_41748"
    }), _react.createElement("feBlend", {
        mode: "normal",
        in: "SourceGraphic",
        in2: "effect1_dropShadow_422_41748",
        result: "shape"
    })), _react.createElement("linearGradient", {
        id: "paint0_linear_422_41748",
        x1: 0.5,
        y1: 0.5,
        x2: 12.5,
        y2: 15.5,
        gradientUnits: "userSpaceOnUse"
    }, _react.createElement("stop", {
        stopColor: "#FF8A00"
    }), _react.createElement("stop", {
        offset: 1,
        stopColor: "#FFC581"
    }))));
const _default = SvgVertical;

},
"4a7b2bdc": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "menu.list.searchTable": "Search Table",
        "searchTable.form.search": "Search",
        "searchTable.form.reset": "Reset",
        "searchTable.columns.id": "Collection ID",
        "searchTable.columns.name": "Collection Name",
        "searchTable.columns.contentType": "Content genre",
        "searchTable.columns.filterType": "Filter method",
        "searchTable.columns.createdTime": "Creation time",
        "searchTable.columns.status": "Status",
        "searchTable.columns.contentNum": "Content quantity",
        "searchTable.columns.operations": "Operation",
        "searchTable.columns.operations.view": "View",
        "searchTable.columns.operations.update": "Edit",
        "searchTable.columns.operations.offline": "Offline",
        "searchTable.columns.operations.online": "Online",
        "searchTable.operations.add": "New",
        "searchTable.operations.upload": "Bulk upload",
        "searchTable.operation.download": "Download",
        "searchForm.id.placeholder": "Please enter the collection ID",
        "searchForm.name.placeholder": "Please enter the collection name",
        "searchForm.all.placeholder": "all"
    },
    "zh-CN": {
        "menu.list": "列表页",
        "menu.list.searchTable": "查询表格",
        "searchTable.form.search": "查询",
        "searchTable.form.reset": "重置",
        "searchTable.columns.id": "集合编号",
        "searchTable.columns.name": "集合名称",
        "searchTable.columns.contentType": "内容体裁",
        "searchTable.columns.filterType": "筛选方式",
        "searchTable.columns.createdTime": "创建时间",
        "searchTable.columns.status": "状态",
        "searchTable.columns.contentNum": "内容量",
        "searchTable.columns.operations": "操作",
        "searchTable.columns.operations.view": "查看",
        "searchTable.columns.operations.update": "修改",
        "searchTable.columns.operations.online": "上线",
        "searchTable.columns.operations.offline": "下线",
        "searchTable.operations.add": "新建",
        "searchTable.operations.upload": "批量导入",
        "searchTable.operation.download": "下载",
        "searchForm.id.placeholder": "请输入集合编号",
        "searchForm.name.placeholder": "请输入集合名称",
        "searchForm.all.placeholder": "全部"
    }
};
const _default = i18n;

},
"f03587c7": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _dayjs = /*#__PURE__*/ _interop_require_default._(farmRequire("d0dc4dad"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
const _querystring = /*#__PURE__*/ _interop_require_default._(farmRequire("dfa4e0d3"));
const { list } = _mockjs.default.mock({
    "list|100": [
        {
            id: /[0-9]{8}[-][0-9]{4}/,
            name: ()=>_mockjs.default.Random.pick([
                    "每日推荐视频集",
                    "抖音短视频候选集",
                    "国际新闻集合"
                ]),
            "contentType|0-2": 0,
            "filterType|0-1": 0,
            "count|0-2000": 0,
            "createdTime|1-60": 0,
            "status|0-1": 0
        }
    ]
});
const filterData = (rest = {})=>{
    const { id, name, "contentType[]": contentType, "filterType[]": filterType, "createdTime[]": createdTime, "status[]": status } = rest;
    if (id) {
        return list.filter((item)=>item.id === id);
    }
    let result = [
        ...list
    ];
    if (name) {
        result = result.filter((item)=>{
            return item.name.toLowerCase().includes(name.toLowerCase());
        });
    }
    if (contentType) {
        result = result.filter((item)=>contentType.includes(item.contentType.toString()));
    }
    if (filterType) {
        result = result.filter((item)=>filterType.includes(item.filterType.toString()));
    }
    if (createdTime && createdTime.length === 2) {
        const [begin, end] = createdTime;
        result = result.filter((item)=>{
            const time = (0, _dayjs.default)().subtract(item.createdTime, "days").format("YYYY-MM-DD HH:mm:ss");
            return !(0, _dayjs.default)(time).isBefore((0, _dayjs.default)(begin)) && !(0, _dayjs.default)(time).isAfter((0, _dayjs.default)(end));
        });
    }
    if (status && status.length) {
        result = result.filter((item)=>status.includes(item.status.toString()));
    }
    return result;
};
(0, _setupMock.default)({
    setup: ()=>{
        _mockjs.default.mock(new RegExp("/api/list"), (params)=>{
            const { page = 1, pageSize = 10, ...rest } = _querystring.default.parseUrl(params.url).query;
            const p = page;
            const ps = pageSize;
            const result = filterData(rest);
            return {
                list: result.slice((p - 1) * ps, p * ps),
                total: result.length
            };
        });
    }
});

},});