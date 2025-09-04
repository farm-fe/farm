import mergeProps from './export';

var Trigger = /** @class */ (function (_super) {
    __extends(Trigger, _super);
    function Trigger(props, context) {
        var _this = _super.call(this, props, context) || this;

        _this.getMergedProps = function (baseProps) {
            var componentConfig = _this.context.componentConfig;
            var props = mergeProps(baseProps || _this.props, defaultProps, componentConfig === null || componentConfig === void 0 ? void 0 : componentConfig.Trigger);
            return props;
        };

        return _this;
    }
    Trigger.prototype.render = function () {
        var _a, _b;
        var _this = this;
        var _c;
        var _d = this.getMergedProps(), children = _d.children, style = _d.style, className = _d.className, arrowProps = _d.arrowProps, disabled = _d.disabled, popup = _d.popup, classNames = _d.classNames, duration = _d.duration, unmountOnExit = _d.unmountOnExit, alignPoint = _d.alignPoint, autoAlignPopupWidth = _d.autoAlignPopupWidth, position = _d.position, childrenPrefix = _d.childrenPrefix, showArrow = _d.showArrow, dropdownPopupStyle = _d.popupStyle;
        var isExistChildren = children || children === 0;
        var _e = this.context, getPrefixCls = _e.getPrefixCls, zIndex = _e.zIndex;
        var _f = this.state, popupVisible = _f.popupVisible, popupStyle = _f.popupStyle;
        if (!popup) {
            return null;
        }
        var mergeProps = {};
        var popupEventProps = {
            onMouseDown: this.onPopupMouseDown,
        };
        if (this.isHoverTrigger() && !disabled) {
            mergeProps.onMouseEnter = this.onMouseEnter;
            mergeProps.onMouseLeave = this.onMouseLeave;
            if (alignPoint) {
                mergeProps.onMouseMove = this.onMouseMove;
            }
            if (!this.isPopupHoverHide()) {
                popupEventProps.onMouseEnter = this.onPopupMouseEnter;
                popupEventProps.onMouseLeave = this.onPopupMouseLeave;
            }
        }
        else {
            mergeProps.onMouseEnter = this.triggerOriginEvent('onMouseEnter');
            mergeProps.onMouseLeave = this.triggerOriginEvent('onMouseLeave');
        }
        if (this.isContextMenuTrigger() && !disabled) {
            mergeProps.onContextMenu = this.onContextMenu;
            mergeProps.onClick = this.hideContextMenu;
        }
        else {
            mergeProps.onContextMenu = this.triggerOriginEvent('onContextMenu');
        }
        if (this.isClickTrigger() && !disabled) {
            mergeProps.onClick = this.onClick;
        }
        else {
            mergeProps.onClick = mergeProps.onClick || this.triggerOriginEvent('onClick');
        }
        if (this.isFocusTrigger() && !disabled) {
            mergeProps.onFocus = this.onFocus;
            if (this.isBlurToHide()) {
                mergeProps.onBlur = this.onBlur;
            }
        }
        else {
            mergeProps.onFocus = this.triggerOriginEvent('onFocus');
            mergeProps.onBlur = this.triggerOriginEvent('onBlur');
        }
        if (!disabled) {
            mergeProps.onKeyDown = this.onKeyDown;
        }
        else {
            mergeProps.onKeyDown = this.triggerOriginEvent('onKeyDown');
        }
        var child = this.getChild();
        var popupChildren = React.Children.only(popup());
        if (child.props.className) {
            mergeProps.className = child.props.className;
        }
        if (childrenPrefix && popupVisible) {
            mergeProps.className = mergeProps.className
                ? mergeProps.className + " " + childrenPrefix + "-open"
                : childrenPrefix + "-open";
        }
        // 只有在focus触发时，设置tabIndex，点击tab键，能触发focus事件，展示弹出框
        if (this.isFocusTrigger()) {
            mergeProps.tabIndex = disabled ? -1 : 0;
        }

    };
    return Trigger;
});
export default Trigger;
