(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'search-table_index_7c2d.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"60420157": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _authentication = /*#__PURE__*/ _interop_require_default._(farmRequire("5e369e13"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const PermissionWrapper = (props)=>{
    const { backup, requiredPermissions, oneOfPerm } = props;
    const [hasPermission, setHasPermission] = (0, _react.useState)(false);
    const userInfo = (0, _reactredux.useSelector)((state)=>state.userInfo);
    (0, _react.useEffect)(()=>{
        const hasPermission = (0, _authentication.default)({
            requiredPermissions,
            oneOfPerm
        }, userInfo.permissions);
        setHasPermission(hasPermission);
    }, [
        requiredPermissions,
        oneOfPerm,
        userInfo.permissions
    ]);
    if (hasPermission) {
        return _react.default.createElement(_react.default.Fragment, null, convertReactElement(props.children));
    }
    if (backup) {
        return _react.default.createElement(_react.default.Fragment, null, convertReactElement(backup));
    }
    return null;
};
function convertReactElement(node) {
    if (!_react.default.isValidElement(node)) {
        return _react.default.createElement(_react.default.Fragment, null, node);
    }
    return node;
}
const _default = PermissionWrapper;

},});