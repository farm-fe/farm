//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};var __assign = this && this.__assign || function() {
    __assign = Object.assign || function(t) {
        for(var s, i = 1, n = arguments.length; i < n; i++){
            s = arguments[i];
            for(var p in s)if (Object.prototype.hasOwnProperty.call(s, p)) t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
function mergeProps(componentProps, defaultProps, globalComponentConfig) {
    var _defaultProps = __assign(__assign({}, defaultProps), globalComponentConfig);
    var props$1 = __assign({}, componentProps);
    for(var propName in _defaultProps){
        if (props$1[propName] === undefined) {
            props$1[propName] = _defaultProps[propName];
        }
    }
    return props$1;
}

var Trigger = function(_super) {
    __extends(Trigger$1, _super);
    function Trigger$1(props, context) {
        var _this = _super.call(this, props, context) || this;
        _this.getMergedProps = function(baseProps) {
            var componentConfig = _this.context.componentConfig;
            var props$3 = mergeProps(baseProps || _this.props, defaultProps, componentConfig === null || componentConfig === void 0 ? void 0 : componentConfig.Trigger);
            return props$3;
        };
        return _this;
    }
    Trigger$1.prototype.render = function() {
        var _a, _b;
        var _this$1 = this;
        var _c;
        var _d = this.getMergedProps(), children = _d.children, style = _d.style, className = _d.className, arrowProps = _d.arrowProps, disabled = _d.disabled, popup = _d.popup, classNames = _d.classNames, duration = _d.duration, unmountOnExit = _d.unmountOnExit, alignPoint = _d.alignPoint, autoAlignPopupWidth = _d.autoAlignPopupWidth, position = _d.position, childrenPrefix = _d.childrenPrefix, showArrow = _d.showArrow, dropdownPopupStyle = _d.popupStyle;
        var isExistChildren = children || children === 0;
        var _e = this.context, getPrefixCls = _e.getPrefixCls, zIndex = _e.zIndex;
        var _f = this.state, popupVisible = _f.popupVisible, popupStyle = _f.popupStyle;
        if (!popup) {
            return null;
        }
        var mergeProps$1 = {};
        var popupEventProps = {
            onMouseDown: this.onPopupMouseDown
        };
        if (this.isHoverTrigger() && !disabled) {
            mergeProps$1.onMouseEnter = this.onMouseEnter;
            mergeProps$1.onMouseLeave = this.onMouseLeave;
            if (alignPoint) {
                mergeProps$1.onMouseMove = this.onMouseMove;
            }
            if (!this.isPopupHoverHide()) {
                popupEventProps.onMouseEnter = this.onPopupMouseEnter;
                popupEventProps.onMouseLeave = this.onPopupMouseLeave;
            }
        } else {
            mergeProps$1.onMouseEnter = this.triggerOriginEvent('onMouseEnter');
            mergeProps$1.onMouseLeave = this.triggerOriginEvent('onMouseLeave');
        }
        if (this.isContextMenuTrigger() && !disabled) {
            mergeProps$1.onContextMenu = this.onContextMenu;
            mergeProps$1.onClick = this.hideContextMenu;
        } else {
            mergeProps$1.onContextMenu = this.triggerOriginEvent('onContextMenu');
        }
        if (this.isClickTrigger() && !disabled) {
            mergeProps$1.onClick = this.onClick;
        } else {
            mergeProps$1.onClick = mergeProps$1.onClick || this.triggerOriginEvent('onClick');
        }
        if (this.isFocusTrigger() && !disabled) {
            mergeProps$1.onFocus = this.onFocus;
            if (this.isBlurToHide()) {
                mergeProps$1.onBlur = this.onBlur;
            }
        } else {
            mergeProps$1.onFocus = this.triggerOriginEvent('onFocus');
            mergeProps$1.onBlur = this.triggerOriginEvent('onBlur');
        }
        if (!disabled) {
            mergeProps$1.onKeyDown = this.onKeyDown;
        } else {
            mergeProps$1.onKeyDown = this.triggerOriginEvent('onKeyDown');
        }
        var child = this.getChild();
        var popupChildren = React.Children.only(popup());
        if (child.props.className) {
            mergeProps$1.className = child.props.className;
        }
        if (childrenPrefix && popupVisible) {
            mergeProps$1.className = mergeProps$1.className ? mergeProps$1.className + " " + childrenPrefix + "-open" : childrenPrefix + "-open";
        }
        if (this.isFocusTrigger()) {
            mergeProps$1.tabIndex = disabled ? -1 : 0;
        }
    };
    return Trigger$1;
};
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){var filename = ((function(){return import.meta.url})());for(var r in _){_[r].__farm_resource_pot__=filename;global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");