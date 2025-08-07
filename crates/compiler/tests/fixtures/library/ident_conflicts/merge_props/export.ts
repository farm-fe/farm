var __assign = (this && this.__assign) || function () {
  __assign = Object.assign || function(t) {
      for (var s, i = 1, n = arguments.length; i < n; i++) {
          s = arguments[i];
          for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
              t[p] = s[p];
      }
      return t;
  };
  return __assign.apply(this, arguments);
};
export default function mergeProps(componentProps, defaultProps, globalComponentConfig) {
  var _defaultProps = __assign(__assign({}, defaultProps), globalComponentConfig);
  var props = __assign({}, componentProps);
  // https://github.com/facebook/react/blob/cae635054e17a6f107a39d328649137b83f25972/packages/react/src/ReactElement.js#L312
  for (var propName in _defaultProps) {
      if (props[propName] === undefined) {
          props[propName] = _defaultProps[propName];
      }
  }
  return props;
}
