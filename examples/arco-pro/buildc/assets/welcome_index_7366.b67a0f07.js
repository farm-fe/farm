(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'welcome_index_7366.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"1123b6ff": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "header": `header-7c7c0813`
};

},
"256c974a": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "code-block": `code-block-da150ea7`,
    "code-block-content": `code-block-content-da150ea7`,
    "code-block-copy-btn": `code-block-copy-btn-da150ea7`
};

},
"3ef33d8a": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return CodeBlock;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _clipboard = /*#__PURE__*/ _interop_require_default._(farmRequire("4477845a"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _codeblockmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("256c974a"));
function CodeBlock(props) {
    const { code } = props;
    return _react.default.createElement("pre", {
        className: _codeblockmoduleless.default['code-block']
    }, _react.default.createElement("code", {
        className: _codeblockmoduleless.default['code-block-content']
    }, code), _react.default.createElement(_webreact.Tooltip, {
        content: "点击复制命令"
    }, _react.default.createElement(_webreact.Button, {
        type: "text",
        className: _codeblockmoduleless.default['code-block-copy-btn'],
        icon: _react.default.createElement(_icon.IconCopy, null),
        onClick: ()=>{
            (0, _clipboard.default)(code);
            _webreact.Message.success('复制成功');
        }
    })));
}

},});