(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'multi-dimension-data-analysis_index_e607.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"315f834c": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("c244d0cb"));
const { Text } = _webreact.Typography;
function CustomTooltip(props) {
    const { formatter = (value)=>value, color, name } = props;
    return _react.default.createElement("div", {
        className: _indexmoduleless.default['customer-tooltip']
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['customer-tooltip-title']
    }, _react.default.createElement(Text, {
        bold: true
    }, props.title)), _react.default.createElement("div", null, props.data.map((item, index)=>_react.default.createElement("div", {
            className: _indexmoduleless.default['customer-tooltip-item'],
            key: index
        }, _react.default.createElement("div", null, _react.default.createElement(_webreact.Badge, {
            color: color || item.color
        }), name || item.name), _react.default.createElement("div", null, _react.default.createElement(Text, {
            bold: true
        }, formatter(item.value)))))));
}
const _default = CustomTooltip;

},
"c244d0cb": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "customer-tooltip-item": `customer-tooltip-item-3f86997c`,
    "customer-tooltip-title": `customer-tooltip-title-3f86997c`
};

},});