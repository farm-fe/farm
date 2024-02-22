(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_2782.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"01857a72": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * This function is like
 * [`Object.keys`](http://ecma-international.org/ecma-262/7.0/#sec-object.keys)
 * except that it includes inherited enumerable properties.
 *
 * @private
 * @param {Object} object The object to query.
 * @returns {Array} Returns the array of property names.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function nativeKeysIn(object) {
    var result = [];
    if (object != null) {
        for(var key in Object(object)){
            result.push(key);
        }
    }
    return result;
}
const _default = nativeKeysIn;

},
"023b1fe0": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _Symbol = /*#__PURE__*/ _interop_require_default._(farmRequire("df3b0a5b"));
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/**
 * Used to resolve the
 * [`toStringTag`](http://ecma-international.org/ecma-262/7.0/#sec-object.prototype.tostring)
 * of values.
 */ var nativeObjectToString = objectProto.toString;
/** Built-in value references. */ var symToStringTag = _Symbol.default ? _Symbol.default.toStringTag : undefined;
/**
 * A specialized version of `baseGetTag` which ignores `Symbol.toStringTag` values.
 *
 * @private
 * @param {*} value The value to query.
 * @returns {string} Returns the raw `toStringTag`.
 */ function getRawTag(value) {
    var isOwn = hasOwnProperty.call(value, symToStringTag), tag = value[symToStringTag];
    try {
        value[symToStringTag] = undefined;
        var unmasked = true;
    } catch (e) {}
    var result = nativeObjectToString.call(value);
    if (unmasked) {
        if (isOwn) {
            value[symToStringTag] = tag;
        } else {
            delete value[symToStringTag];
        }
    }
    return result;
}
const _default = getRawTag;

},
"05fd2918": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _coreJsData = /*#__PURE__*/ _interop_require_default._(farmRequire("e53a6514"));
/** Used to detect methods masquerading as native. */ var maskSrcKey = function() {
    var uid = /[^.]+$/.exec(_coreJsData.default && _coreJsData.default.keys && _coreJsData.default.keys.IE_PROTO || '');
    return uid ? 'Symbol(src)_1.' + uid : '';
}();
/**
 * Checks if `func` has its source masked.
 *
 * @private
 * @param {Function} func The function to check.
 * @returns {boolean} Returns `true` if `func` is masked, else `false`.
 */ function isMasked(func) {
    return !!maskSrcKey && maskSrcKey in func;
}
const _default = isMasked;

},
"06f8323b": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * A specialized version of `_.forEach` for arrays without support for
 * iteratee shorthands.
 *
 * @private
 * @param {Array} [array] The array to iterate over.
 * @param {Function} iteratee The function invoked per iteration.
 * @returns {Array} Returns `array`.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function arrayEach(array, iteratee) {
    var index = -1, length = array == null ? 0 : array.length;
    while(++index < length){
        if (iteratee(array[index], index, array) === false) {
            break;
        }
    }
    return array;
}
const _default = arrayEach;

},
"095ade58": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _arrayLikeKeys = /*#__PURE__*/ _interop_require_default._(farmRequire("fae4ae0b"));
const _baseKeys = /*#__PURE__*/ _interop_require_default._(farmRequire("80d7fb42"));
const _isArrayLike = /*#__PURE__*/ _interop_require_default._(farmRequire("65cf6153"));
/**
 * Creates an array of the own enumerable property names of `object`.
 *
 * **Note:** Non-object values are coerced to objects. See the
 * [ES spec](http://ecma-international.org/ecma-262/7.0/#sec-object.keys)
 * for more details.
 *
 * @static
 * @since 0.1.0
 * @memberOf _
 * @category Object
 * @param {Object} object The object to query.
 * @returns {Array} Returns the array of property names.
 * @example
 *
 * function Foo() {
 *   this.a = 1;
 *   this.b = 2;
 * }
 *
 * Foo.prototype.c = 3;
 *
 * _.keys(new Foo);
 * // => ['a', 'b'] (iteration order is not guaranteed)
 *
 * _.keys('hi');
 * // => ['0', '1']
 */ function keys(object) {
    return (0, _isArrayLike.default)(object) ? (0, _arrayLikeKeys.default)(object) : (0, _baseKeys.default)(object);
}
const _default = keys;

},
"0c79d683": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Alpha: function() {
        return _Alpha.default;
    },
    Checkboard: function() {
        return _Checkboard.default;
    },
    ColorWrap: function() {
        return _ColorWrap.default;
    },
    EditableInput: function() {
        return _EditableInput.default;
    },
    Hue: function() {
        return _Hue.default;
    },
    Saturation: function() {
        return _Saturation.default;
    },
    Swatch: function() {
        return _Swatch.default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _Alpha = /*#__PURE__*/ _interop_require_default._(farmRequire("61567a8f"));
const _Checkboard = /*#__PURE__*/ _interop_require_default._(farmRequire("83168a07"));
const _EditableInput = /*#__PURE__*/ _interop_require_default._(farmRequire("56568a41"));
const _Hue = /*#__PURE__*/ _interop_require_default._(farmRequire("d35f90df"));
const _Saturation = /*#__PURE__*/ _interop_require_default._(farmRequire("105d8ee9"));
const _ColorWrap = /*#__PURE__*/ _interop_require_default._(farmRequire("813cc28a"));
const _Swatch = /*#__PURE__*/ _interop_require_default._(farmRequire("b2779319"));

},
"0ff00a65": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "SketchPicker", {
    enumerable: true,
    get: function() {
        return _Sketch.default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _Chrome = /*#__PURE__*/ _interop_require_default._(farmRequire("6bfe3296"));
const _Sketch = /*#__PURE__*/ _interop_require_default._(farmRequire("1f149ea1"));

},
"105d8ee9": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Saturation: function() {
        return Saturation;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _throttle = /*#__PURE__*/ _interop_require_default._(farmRequire("4aa7e9f2"));
const _saturation = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("b437633f"));
var _createClass = function() {
    function defineProperties(target, props) {
        for(var i = 0; i < props.length; i++){
            var descriptor = props[i];
            descriptor.enumerable = descriptor.enumerable || false;
            descriptor.configurable = true;
            if ("value" in descriptor) descriptor.writable = true;
            Object.defineProperty(target, descriptor.key, descriptor);
        }
    }
    return function(Constructor, protoProps, staticProps) {
        if (protoProps) defineProperties(Constructor.prototype, protoProps);
        if (staticProps) defineProperties(Constructor, staticProps);
        return Constructor;
    };
}();
function _classCallCheck(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
function _possibleConstructorReturn(self, call) {
    if (!self) {
        throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
    }
    return call && (typeof call === "object" || typeof call === "function") ? call : self;
}
function _inherits(subClass, superClass) {
    if (typeof superClass !== "function" && superClass !== null) {
        throw new TypeError("Super expression must either be null or a function, not " + typeof superClass);
    }
    subClass.prototype = Object.create(superClass && superClass.prototype, {
        constructor: {
            value: subClass,
            enumerable: false,
            writable: true,
            configurable: true
        }
    });
    if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass;
}
var Saturation = function(_ref) {
    _inherits(Saturation, _ref);
    function Saturation(props) {
        _classCallCheck(this, Saturation);
        var _this = _possibleConstructorReturn(this, (Saturation.__proto__ || Object.getPrototypeOf(Saturation)).call(this, props));
        _this.handleChange = function(e) {
            typeof _this.props.onChange === 'function' && _this.throttle(_this.props.onChange, _saturation.calculateChange(e, _this.props.hsl, _this.container), e);
        };
        _this.handleMouseDown = function(e) {
            _this.handleChange(e);
            var renderWindow = _this.getContainerRenderWindow();
            renderWindow.addEventListener('mousemove', _this.handleChange);
            renderWindow.addEventListener('mouseup', _this.handleMouseUp);
        };
        _this.handleMouseUp = function() {
            _this.unbindEventListeners();
        };
        _this.throttle = (0, _throttle.default)(function(fn, data, e) {
            fn(data, e);
        }, 50);
        return _this;
    }
    _createClass(Saturation, [
        {
            key: 'componentWillUnmount',
            value: function componentWillUnmount() {
                this.throttle.cancel();
                this.unbindEventListeners();
            }
        },
        {
            key: 'getContainerRenderWindow',
            value: function getContainerRenderWindow() {
                var container = this.container;
                var renderWindow = window;
                while(!renderWindow.document.contains(container) && renderWindow.parent !== renderWindow){
                    renderWindow = renderWindow.parent;
                }
                return renderWindow;
            }
        },
        {
            key: 'unbindEventListeners',
            value: function unbindEventListeners() {
                var renderWindow = this.getContainerRenderWindow();
                renderWindow.removeEventListener('mousemove', this.handleChange);
                renderWindow.removeEventListener('mouseup', this.handleMouseUp);
            }
        },
        {
            key: 'render',
            value: function render() {
                var _this2 = this;
                var _ref2 = this.props.style || {}, color = _ref2.color, white = _ref2.white, black = _ref2.black, pointer = _ref2.pointer, circle = _ref2.circle;
                var styles = (0, _reactcss.default)({
                    'default': {
                        color: {
                            absolute: '0px 0px 0px 0px',
                            background: 'hsl(' + this.props.hsl.h + ',100%, 50%)',
                            borderRadius: this.props.radius
                        },
                        white: {
                            absolute: '0px 0px 0px 0px',
                            borderRadius: this.props.radius
                        },
                        black: {
                            absolute: '0px 0px 0px 0px',
                            boxShadow: this.props.shadow,
                            borderRadius: this.props.radius
                        },
                        pointer: {
                            position: 'absolute',
                            top: -(this.props.hsv.v * 100) + 100 + '%',
                            left: this.props.hsv.s * 100 + '%',
                            cursor: 'default'
                        },
                        circle: {
                            width: '4px',
                            height: '4px',
                            boxShadow: '0 0 0 1.5px #fff, inset 0 0 1px 1px rgba(0,0,0,.3),\n            0 0 1px 2px rgba(0,0,0,.4)',
                            borderRadius: '50%',
                            cursor: 'hand',
                            transform: 'translate(-2px, -2px)'
                        }
                    },
                    'custom': {
                        color: color,
                        white: white,
                        black: black,
                        pointer: pointer,
                        circle: circle
                    }
                }, {
                    'custom': !!this.props.style
                });
                return _react.default.createElement('div', {
                    style: styles.color,
                    ref: function ref(container) {
                        return _this2.container = container;
                    },
                    onMouseDown: this.handleMouseDown,
                    onTouchMove: this.handleChange,
                    onTouchStart: this.handleChange
                }, _react.default.createElement('style', null, '\n          .saturation-white {\n            background: -webkit-linear-gradient(to right, #fff, rgba(255,255,255,0));\n            background: linear-gradient(to right, #fff, rgba(255,255,255,0));\n          }\n          .saturation-black {\n            background: -webkit-linear-gradient(to top, #000, rgba(0,0,0,0));\n            background: linear-gradient(to top, #000, rgba(0,0,0,0));\n          }\n        '), _react.default.createElement('div', {
                    style: styles.white,
                    className: 'saturation-white'
                }, _react.default.createElement('div', {
                    style: styles.black,
                    className: 'saturation-black'
                }), _react.default.createElement('div', {
                    style: styles.pointer
                }, this.props.pointer ? _react.default.createElement(this.props.pointer, this.props) : _react.default.createElement('div', {
                    style: styles.circle
                }))));
            }
        }
    ]);
    return Saturation;
}(_react.PureComponent || _react.Component);
const _default = Saturation;

},
"1144a658": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _Hash = /*#__PURE__*/ _interop_require_default._(farmRequire("e09a6b9e"));
const _ListCache = /*#__PURE__*/ _interop_require_default._(farmRequire("47a014c7"));
const _Map = /*#__PURE__*/ _interop_require_default._(farmRequire("5aa5795a"));
/**
 * Removes all key-value entries from the map.
 *
 * @private
 * @name clear
 * @memberOf MapCache
 */ function mapCacheClear() {
    this.size = 0;
    this.__data__ = {
        'hash': new _Hash.default,
        'map': new (_Map.default || _ListCache.default),
        'string': new _Hash.default
    };
}
const _default = mapCacheClear;

},
"121032f9": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Creates a base function for methods like `_.forIn` and `_.forOwn`.
 *
 * @private
 * @param {boolean} [fromRight] Specify iterating from right to left.
 * @returns {Function} Returns the new base function.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function createBaseFor(fromRight) {
    return function(object, iteratee, keysFunc) {
        var index = -1, iterable = Object(object), props = keysFunc(object), length = props.length;
        while(length--){
            var key = props[fromRight ? length : ++index];
            if (iteratee(iterable[key], key, iterable) === false) {
                break;
            }
        }
        return object;
    };
}
const _default = createBaseFor;

},
"1485eb1e": function(module, exports, farmRequire, farmDynamicRequire) {
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
    ChromePointerCircle: function() {
        return ChromePointerCircle;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
var ChromePointerCircle = function ChromePointerCircle() {
    var styles = (0, _reactcss.default)({
        'default': {
            picker: {
                width: '12px',
                height: '12px',
                borderRadius: '6px',
                boxShadow: 'inset 0 0 0 1px #fff',
                transform: 'translate(-6px, -6px)'
            }
        }
    });
    return _react.default.createElement('div', {
        style: styles.picker
    });
};
const _default = ChromePointerCircle;

},
"14ae416a": function(module, exports, farmRequire, farmDynamicRequire) {
/** Used to detect hot functions by number of calls within a span of milliseconds. */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var HOT_COUNT = 800, HOT_SPAN = 16;
/* Built-in method references for those with the same name as other `lodash` methods. */ var nativeNow = Date.now;
/**
 * Creates a function that'll short out and invoke `identity` instead
 * of `func` when it's called `HOT_COUNT` or more times in `HOT_SPAN`
 * milliseconds.
 *
 * @private
 * @param {Function} func The function to restrict.
 * @returns {Function} Returns the new shortable function.
 */ function shortOut(func) {
    var count = 0, lastCalled = 0;
    return function() {
        var stamp = nativeNow(), remaining = HOT_SPAN - (stamp - lastCalled);
        lastCalled = stamp;
        if (remaining > 0) {
            if (++count >= HOT_COUNT) {
                return arguments[0];
            }
        } else {
            count = 0;
        }
        return func.apply(undefined, arguments);
    };
}
const _default = shortOut;

},
"1954b1ef": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Checks if `value` is object-like. A value is object-like if it's not `null`
 * and has a `typeof` result of "object".
 *
 * @static
 * @memberOf _
 * @since 4.0.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is object-like, else `false`.
 * @example
 *
 * _.isObjectLike({});
 * // => true
 *
 * _.isObjectLike([1, 2, 3]);
 * // => true
 *
 * _.isObjectLike(_.noop);
 * // => false
 *
 * _.isObjectLike(null);
 * // => false
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function isObjectLike(value) {
    return value != null && typeof value == 'object';
}
const _default = isObjectLike;

},
"19eb8ac9": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _assignMergeValue = /*#__PURE__*/ _interop_require_default._(farmRequire("ec3f319a"));
const _cloneBuffer = /*#__PURE__*/ _interop_require_default._(farmRequire("8feb9431"));
const _cloneTypedArray = /*#__PURE__*/ _interop_require_default._(farmRequire("a55fc3a6"));
const _copyArray = /*#__PURE__*/ _interop_require_default._(farmRequire("50a4b614"));
const _initCloneObject = /*#__PURE__*/ _interop_require_default._(farmRequire("b93a8038"));
const _isArguments = /*#__PURE__*/ _interop_require_default._(farmRequire("23576120"));
const _isArray = /*#__PURE__*/ _interop_require_default._(farmRequire("f1ceb9be"));
const _isArrayLikeObject = /*#__PURE__*/ _interop_require_default._(farmRequire("b10d87d6"));
const _isBuffer = /*#__PURE__*/ _interop_require_default._(farmRequire("356fea7f"));
const _isFunction = /*#__PURE__*/ _interop_require_default._(farmRequire("7fd4ad6b"));
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
const _isPlainObject = /*#__PURE__*/ _interop_require_default._(farmRequire("fac7b69f"));
const _isTypedArray = /*#__PURE__*/ _interop_require_default._(farmRequire("a8b5d78d"));
const _safeGet = /*#__PURE__*/ _interop_require_default._(farmRequire("5057e1a2"));
const _toPlainObject = /*#__PURE__*/ _interop_require_default._(farmRequire("ab83cd8f"));
/**
 * A specialized version of `baseMerge` for arrays and objects which performs
 * deep merges and tracks traversed objects enabling objects with circular
 * references to be merged.
 *
 * @private
 * @param {Object} object The destination object.
 * @param {Object} source The source object.
 * @param {string} key The key of the value to merge.
 * @param {number} srcIndex The index of `source`.
 * @param {Function} mergeFunc The function to merge values.
 * @param {Function} [customizer] The function to customize assigned values.
 * @param {Object} [stack] Tracks traversed source values and their merged
 *  counterparts.
 */ function baseMergeDeep(object, source, key, srcIndex, mergeFunc, customizer, stack) {
    var objValue = (0, _safeGet.default)(object, key), srcValue = (0, _safeGet.default)(source, key), stacked = stack.get(srcValue);
    if (stacked) {
        (0, _assignMergeValue.default)(object, key, stacked);
        return;
    }
    var newValue = customizer ? customizer(objValue, srcValue, key + '', object, source, stack) : undefined;
    var isCommon = newValue === undefined;
    if (isCommon) {
        var isArr = (0, _isArray.default)(srcValue), isBuff = !isArr && (0, _isBuffer.default)(srcValue), isTyped = !isArr && !isBuff && (0, _isTypedArray.default)(srcValue);
        newValue = srcValue;
        if (isArr || isBuff || isTyped) {
            if ((0, _isArray.default)(objValue)) {
                newValue = objValue;
            } else if ((0, _isArrayLikeObject.default)(objValue)) {
                newValue = (0, _copyArray.default)(objValue);
            } else if (isBuff) {
                isCommon = false;
                newValue = (0, _cloneBuffer.default)(srcValue, true);
            } else if (isTyped) {
                isCommon = false;
                newValue = (0, _cloneTypedArray.default)(srcValue, true);
            } else {
                newValue = [];
            }
        } else if ((0, _isPlainObject.default)(srcValue) || (0, _isArguments.default)(srcValue)) {
            newValue = objValue;
            if ((0, _isArguments.default)(objValue)) {
                newValue = (0, _toPlainObject.default)(objValue);
            } else if (!(0, _isObject.default)(objValue) || (0, _isFunction.default)(objValue)) {
                newValue = (0, _initCloneObject.default)(srcValue);
            }
        } else {
            isCommon = false;
        }
    }
    if (isCommon) {
        // Recursively merge objects and arrays (susceptible to call stack limits).
        stack.set(srcValue, newValue);
        mergeFunc(newValue, srcValue, srcIndex, customizer, stack);
        stack['delete'](srcValue);
    }
    (0, _assignMergeValue.default)(object, key, newValue);
}
const _default = baseMergeDeep;

},
"1f149ea1": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Sketch: function() {
        return Sketch;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _proptypes = /*#__PURE__*/ _interop_require_default._(farmRequire("75a1a40c"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _merge = /*#__PURE__*/ _interop_require_default._(farmRequire("30490595"));
const _common = farmRequire("0c79d683");
const _SketchFields = /*#__PURE__*/ _interop_require_default._(farmRequire("f40f3977"));
const _SketchPresetColors = /*#__PURE__*/ _interop_require_default._(farmRequire("7d285119"));
var _extends = Object.assign || function(target) {
    for(var i = 1; i < arguments.length; i++){
        var source = arguments[i];
        for(var key in source){
            if (Object.prototype.hasOwnProperty.call(source, key)) {
                target[key] = source[key];
            }
        }
    }
    return target;
};
var Sketch = function Sketch(_ref) {
    var width = _ref.width, rgb = _ref.rgb, hex = _ref.hex, hsv = _ref.hsv, hsl = _ref.hsl, onChange = _ref.onChange, onSwatchHover = _ref.onSwatchHover, disableAlpha = _ref.disableAlpha, presetColors = _ref.presetColors, renderers = _ref.renderers, _ref$styles = _ref.styles, passedStyles = _ref$styles === undefined ? {} : _ref$styles, _ref$className = _ref.className, className = _ref$className === undefined ? '' : _ref$className;
    var styles = (0, _reactcss.default)((0, _merge.default)({
        'default': _extends({
            picker: {
                width: width,
                padding: '10px 10px 0',
                boxSizing: 'initial',
                background: '#fff',
                borderRadius: '4px',
                boxShadow: '0 0 0 1px rgba(0,0,0,.15), 0 8px 16px rgba(0,0,0,.15)'
            },
            saturation: {
                width: '100%',
                paddingBottom: '75%',
                position: 'relative',
                overflow: 'hidden'
            },
            Saturation: {
                radius: '3px',
                shadow: 'inset 0 0 0 1px rgba(0,0,0,.15), inset 0 0 4px rgba(0,0,0,.25)'
            },
            controls: {
                display: 'flex'
            },
            sliders: {
                padding: '4px 0',
                flex: '1'
            },
            color: {
                width: '24px',
                height: '24px',
                position: 'relative',
                marginTop: '4px',
                marginLeft: '4px',
                borderRadius: '3px'
            },
            activeColor: {
                absolute: '0px 0px 0px 0px',
                borderRadius: '2px',
                background: 'rgba(' + rgb.r + ',' + rgb.g + ',' + rgb.b + ',' + rgb.a + ')',
                boxShadow: 'inset 0 0 0 1px rgba(0,0,0,.15), inset 0 0 4px rgba(0,0,0,.25)'
            },
            hue: {
                position: 'relative',
                height: '10px',
                overflow: 'hidden'
            },
            Hue: {
                radius: '2px',
                shadow: 'inset 0 0 0 1px rgba(0,0,0,.15), inset 0 0 4px rgba(0,0,0,.25)'
            },
            alpha: {
                position: 'relative',
                height: '10px',
                marginTop: '4px',
                overflow: 'hidden'
            },
            Alpha: {
                radius: '2px',
                shadow: 'inset 0 0 0 1px rgba(0,0,0,.15), inset 0 0 4px rgba(0,0,0,.25)'
            }
        }, passedStyles),
        'disableAlpha': {
            color: {
                height: '10px'
            },
            hue: {
                height: '10px'
            },
            alpha: {
                display: 'none'
            }
        }
    }, passedStyles), {
        disableAlpha: disableAlpha
    });
    return _react.default.createElement('div', {
        style: styles.picker,
        className: 'sketch-picker ' + className
    }, _react.default.createElement('div', {
        style: styles.saturation
    }, _react.default.createElement(_common.Saturation, {
        style: styles.Saturation,
        hsl: hsl,
        hsv: hsv,
        onChange: onChange
    })), _react.default.createElement('div', {
        style: styles.controls,
        className: 'flexbox-fix'
    }, _react.default.createElement('div', {
        style: styles.sliders
    }, _react.default.createElement('div', {
        style: styles.hue
    }, _react.default.createElement(_common.Hue, {
        style: styles.Hue,
        hsl: hsl,
        onChange: onChange
    })), _react.default.createElement('div', {
        style: styles.alpha
    }, _react.default.createElement(_common.Alpha, {
        style: styles.Alpha,
        rgb: rgb,
        hsl: hsl,
        renderers: renderers,
        onChange: onChange
    }))), _react.default.createElement('div', {
        style: styles.color
    }, _react.default.createElement(_common.Checkboard, null), _react.default.createElement('div', {
        style: styles.activeColor
    }))), _react.default.createElement(_SketchFields.default, {
        rgb: rgb,
        hsl: hsl,
        hex: hex,
        onChange: onChange,
        disableAlpha: disableAlpha
    }), _react.default.createElement(_SketchPresetColors.default, {
        colors: presetColors,
        onClick: onChange,
        onSwatchHover: onSwatchHover
    }));
};
Sketch.propTypes = {
    disableAlpha: _proptypes.default.bool,
    width: _proptypes.default.oneOfType([
        _proptypes.default.string,
        _proptypes.default.number
    ]),
    styles: _proptypes.default.object
};
Sketch.defaultProps = {
    disableAlpha: false,
    width: 200,
    styles: {},
    presetColors: [
        '#D0021B',
        '#F5A623',
        '#F8E71C',
        '#8B572A',
        '#7ED321',
        '#417505',
        '#BD10E0',
        '#9013FE',
        '#4A90E2',
        '#50E3C2',
        '#B8E986',
        '#000000',
        '#4A4A4A',
        '#9B9B9B',
        '#FFFFFF'
    ]
};
const _default = (0, _common.ColorWrap)(Sketch);

},
"2031bec1": function(module, exports, farmRequire, farmDynamicRequire) {
/** Used as references for various `Number` constants. */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var MAX_SAFE_INTEGER = 9007199254740991;
/**
 * Checks if `value` is a valid array-like length.
 *
 * **Note:** This method is loosely based on
 * [`ToLength`](http://ecma-international.org/ecma-262/7.0/#sec-tolength).
 *
 * @static
 * @memberOf _
 * @since 4.0.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a valid length, else `false`.
 * @example
 *
 * _.isLength(3);
 * // => true
 *
 * _.isLength(Number.MIN_VALUE);
 * // => false
 *
 * _.isLength(Infinity);
 * // => false
 *
 * _.isLength('3');
 * // => false
 */ function isLength(value) {
    return typeof value == 'number' && value > -1 && value % 1 == 0 && value <= MAX_SAFE_INTEGER;
}
const _default = isLength;

},
"204518c5": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Checks if a stack value for `key` exists.
 *
 * @private
 * @name has
 * @memberOf Stack
 * @param {string} key The key of the entry to check.
 * @returns {boolean} Returns `true` if an entry for `key` exists, else `false`.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function stackHas(key) {
    return this.__data__.has(key);
}
const _default = stackHas;

},
"2074a2eb": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseGetTag = /*#__PURE__*/ _interop_require_default._(farmRequire("4ffcc116"));
const _isObjectLike = /*#__PURE__*/ _interop_require_default._(farmRequire("1954b1ef"));
/** `Object#toString` result references. */ var symbolTag = '[object Symbol]';
/**
 * Checks if `value` is classified as a `Symbol` primitive or object.
 *
 * @static
 * @memberOf _
 * @since 4.0.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a symbol, else `false`.
 * @example
 *
 * _.isSymbol(Symbol.iterator);
 * // => true
 *
 * _.isSymbol('abc');
 * // => false
 */ function isSymbol(value) {
    return typeof value == 'symbol' || (0, _isObjectLike.default)(value) && (0, _baseGetTag.default)(value) == symbolTag;
}
const _default = isSymbol;

},
"23576120": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseIsArguments = /*#__PURE__*/ _interop_require_default._(farmRequire("5f1cd00b"));
const _isObjectLike = /*#__PURE__*/ _interop_require_default._(farmRequire("1954b1ef"));
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/** Built-in value references. */ var propertyIsEnumerable = objectProto.propertyIsEnumerable;
/**
 * Checks if `value` is likely an `arguments` object.
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is an `arguments` object,
 *  else `false`.
 * @example
 *
 * _.isArguments(function() { return arguments; }());
 * // => true
 *
 * _.isArguments([1, 2, 3]);
 * // => false
 */ var isArguments = (0, _baseIsArguments.default)(function() {
    return arguments;
}()) ? _baseIsArguments.default : function(value) {
    return (0, _isObjectLike.default)(value) && hasOwnProperty.call(value, 'callee') && !propertyIsEnumerable.call(value, 'callee');
};
const _default = isArguments;

},
"23f33b47": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _getNative = /*#__PURE__*/ _interop_require_default._(farmRequire("7d9eff84"));
var defineProperty = function() {
    try {
        var func = (0, _getNative.default)(Object, 'defineProperty');
        func({}, '', {});
        return func;
    } catch (e) {}
}();
const _default = defineProperty;

},
"277986a6": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Gets the value at `key` of `object`.
 *
 * @private
 * @param {Object} [object] The object to query.
 * @param {string} key The key of the property to get.
 * @returns {*} Returns the property value.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function getValue(object, key) {
    return object == null ? undefined : object[key];
}
const _default = getValue;

},
"283458f2": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _assignValue = /*#__PURE__*/ _interop_require_default._(farmRequire("77f602cd"));
const _baseAssignValue = /*#__PURE__*/ _interop_require_default._(farmRequire("53dc8b2a"));
/**
 * Copies properties of `source` to `object`.
 *
 * @private
 * @param {Object} source The object to copy properties from.
 * @param {Array} props The property identifiers to copy.
 * @param {Object} [object={}] The object to copy properties to.
 * @param {Function} [customizer] The function to customize copied values.
 * @returns {Object} Returns `object`.
 */ function copyObject(source, props, object, customizer) {
    var isNew = !object;
    object || (object = {});
    var index = -1, length = props.length;
    while(++index < length){
        var key = props[index];
        var newValue = customizer ? customizer(object[key], source[key], key, object, source) : undefined;
        if (newValue === undefined) {
            newValue = source[key];
        }
        if (isNew) {
            (0, _baseAssignValue.default)(object, key, newValue);
        } else {
            (0, _assignValue.default)(object, key, newValue);
        }
    }
    return object;
}
const _default = copyObject;

},
"2b1f08c0": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
/** Built-in value references. */ var objectCreate = Object.create;
/**
 * The base implementation of `_.create` without support for assigning
 * properties to the created object.
 *
 * @private
 * @param {Object} proto The object to inherit from.
 * @returns {Object} Returns the new object.
 */ var baseCreate = function() {
    function object() {}
    return function(proto) {
        if (!(0, _isObject.default)(proto)) {
            return {};
        }
        if (objectCreate) {
            return objectCreate(proto);
        }
        object.prototype = proto;
        var result = new object;
        object.prototype = undefined;
        return result;
    };
}();
const _default = baseCreate;

},
"2c52b90d": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _assocIndexOf = /*#__PURE__*/ _interop_require_default._(farmRequire("9270f0e3"));
/**
 * Gets the list cache value for `key`.
 *
 * @private
 * @name get
 * @memberOf ListCache
 * @param {string} key The key of the value to get.
 * @returns {*} Returns the entry value.
 */ function listCacheGet(key) {
    var data = this.__data__, index = (0, _assocIndexOf.default)(data, key);
    return index < 0 ? undefined : data[index][1];
}
const _default = listCacheGet;

},
"30490595": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseMerge = /*#__PURE__*/ _interop_require_default._(farmRequire("e4aeffaa"));
const _createAssigner = /*#__PURE__*/ _interop_require_default._(farmRequire("e46625fb"));
/**
 * This method is like `_.assign` except that it recursively merges own and
 * inherited enumerable string keyed properties of source objects into the
 * destination object. Source properties that resolve to `undefined` are
 * skipped if a destination value exists. Array and plain object properties
 * are merged recursively. Other objects and value types are overridden by
 * assignment. Source objects are applied from left to right. Subsequent
 * sources overwrite property assignments of previous sources.
 *
 * **Note:** This method mutates `object`.
 *
 * @static
 * @memberOf _
 * @since 0.5.0
 * @category Object
 * @param {Object} object The destination object.
 * @param {...Object} [sources] The source objects.
 * @returns {Object} Returns `object`.
 * @example
 *
 * var object = {
 *   'a': [{ 'b': 2 }, { 'd': 4 }]
 * };
 *
 * var other = {
 *   'a': [{ 'c': 3 }, { 'e': 5 }]
 * };
 *
 * _.merge(object, other);
 * // => { 'a': [{ 'b': 2, 'c': 3 }, { 'd': 4, 'e': 5 }] }
 */ var merge = (0, _createAssigner.default)(function(object, source, srcIndex) {
    (0, _baseMerge.default)(object, source, srcIndex);
});
const _default = merge;

},
"33c4445f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _apply = /*#__PURE__*/ _interop_require_default._(farmRequire("cbe2a7de"));
/* Built-in method references for those with the same name as other `lodash` methods. */ var nativeMax = Math.max;
/**
 * A specialized version of `baseRest` which transforms the rest array.
 *
 * @private
 * @param {Function} func The function to apply a rest parameter to.
 * @param {number} [start=func.length-1] The start position of the rest parameter.
 * @param {Function} transform The rest array transform.
 * @returns {Function} Returns the new function.
 */ function overRest(func, start, transform) {
    start = nativeMax(start === undefined ? func.length - 1 : start, 0);
    return function() {
        var args = arguments, index = -1, length = nativeMax(args.length - start, 0), array = Array(length);
        while(++index < length){
            array[index] = args[start + index];
        }
        index = -1;
        var otherArgs = Array(start + 1);
        while(++index < start){
            otherArgs[index] = args[index];
        }
        otherArgs[start] = transform(array);
        return (0, _apply.default)(func, this, otherArgs);
    };
}
const _default = overRest;

},
"35076a58": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _nativeCreate = /*#__PURE__*/ _interop_require_default._(farmRequire("dc218a61"));
/**
 * Removes all key-value entries from the hash.
 *
 * @private
 * @name clear
 * @memberOf Hash
 */ function hashClear() {
    this.__data__ = _nativeCreate.default ? (0, _nativeCreate.default)(null) : {};
    this.size = 0;
}
const _default = hashClear;

},
"356fea7f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _root = /*#__PURE__*/ _interop_require_default._(farmRequire("6bf7b771"));
const _stubFalse = /*#__PURE__*/ _interop_require_default._(farmRequire("c14d35a7"));
/** Detect free variable `exports`. */ var freeExports = typeof exports == 'object' && exports && !exports.nodeType && exports;
/** Detect free variable `module`. */ var freeModule = freeExports && typeof module == 'object' && module && !module.nodeType && module;
/** Detect the popular CommonJS extension `module.exports`. */ var moduleExports = freeModule && freeModule.exports === freeExports;
/** Built-in value references. */ var Buffer = moduleExports ? _root.default.Buffer : undefined;
/* Built-in method references for those with the same name as other `lodash` methods. */ var nativeIsBuffer = Buffer ? Buffer.isBuffer : undefined;
/**
 * Checks if `value` is a buffer.
 *
 * @static
 * @memberOf _
 * @since 4.3.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a buffer, else `false`.
 * @example
 *
 * _.isBuffer(new Buffer(2));
 * // => true
 *
 * _.isBuffer(new Uint8Array(2));
 * // => false
 */ var isBuffer = nativeIsBuffer || _stubFalse.default;
const _default = isBuffer;

},
"3e794592": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Performs a
 * [`SameValueZero`](http://ecma-international.org/ecma-262/7.0/#sec-samevaluezero)
 * comparison between two values to determine if they are equivalent.
 *
 * @static
 * @memberOf _
 * @since 4.0.0
 * @category Lang
 * @param {*} value The value to compare.
 * @param {*} other The other value to compare.
 * @returns {boolean} Returns `true` if the values are equivalent, else `false`.
 * @example
 *
 * var object = { 'a': 1 };
 * var other = { 'a': 1 };
 *
 * _.eq(object, object);
 * // => true
 *
 * _.eq(object, other);
 * // => false
 *
 * _.eq('a', 'a');
 * // => true
 *
 * _.eq('a', Object('a'));
 * // => false
 *
 * _.eq(NaN, NaN);
 * // => true
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function eq(value, other) {
    return value === other || value !== value && other !== other;
}
const _default = eq;

},
"3ed12767": function(module, exports, farmRequire, farmDynamicRequire) {
/** Used for built-in method references. */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var objectProto = Object.prototype;
/**
 * Used to resolve the
 * [`toStringTag`](http://ecma-international.org/ecma-262/7.0/#sec-object.prototype.tostring)
 * of values.
 */ var nativeObjectToString = objectProto.toString;
/**
 * Converts `value` to a string using `Object.prototype.toString`.
 *
 * @private
 * @param {*} value The value to convert.
 * @returns {string} Returns the converted string.
 */ function objectToString(value) {
    return nativeObjectToString.call(value);
}
const _default = objectToString;

},
"3f23c083": function(module, exports, farmRequire, farmDynamicRequire) {
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
    ChromeFields: function() {
        return ChromeFields;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _color = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("889a06b2"));
const _isUndefined = /*#__PURE__*/ _interop_require_default._(farmRequire("ea0fed91"));
const _common = farmRequire("0c79d683");
const _UnfoldMoreHorizontalIcon = /*#__PURE__*/ _interop_require_default._(farmRequire("4c6d47bb"));
var _createClass = function() {
    function defineProperties(target, props) {
        for(var i = 0; i < props.length; i++){
            var descriptor = props[i];
            descriptor.enumerable = descriptor.enumerable || false;
            descriptor.configurable = true;
            if ("value" in descriptor) descriptor.writable = true;
            Object.defineProperty(target, descriptor.key, descriptor);
        }
    }
    return function(Constructor, protoProps, staticProps) {
        if (protoProps) defineProperties(Constructor.prototype, protoProps);
        if (staticProps) defineProperties(Constructor, staticProps);
        return Constructor;
    };
}();
function _classCallCheck(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
function _possibleConstructorReturn(self, call) {
    if (!self) {
        throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
    }
    return call && (typeof call === "object" || typeof call === "function") ? call : self;
}
function _inherits(subClass, superClass) {
    if (typeof superClass !== "function" && superClass !== null) {
        throw new TypeError("Super expression must either be null or a function, not " + typeof superClass);
    }
    subClass.prototype = Object.create(superClass && superClass.prototype, {
        constructor: {
            value: subClass,
            enumerable: false,
            writable: true,
            configurable: true
        }
    });
    if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass;
}
var ChromeFields = function(_React$Component) {
    _inherits(ChromeFields, _React$Component);
    function ChromeFields(props) {
        _classCallCheck(this, ChromeFields);
        var _this = _possibleConstructorReturn(this, (ChromeFields.__proto__ || Object.getPrototypeOf(ChromeFields)).call(this));
        _this.toggleViews = function() {
            if (_this.state.view === 'hex') {
                _this.setState({
                    view: 'rgb'
                });
            } else if (_this.state.view === 'rgb') {
                _this.setState({
                    view: 'hsl'
                });
            } else if (_this.state.view === 'hsl') {
                if (_this.props.hsl.a === 1) {
                    _this.setState({
                        view: 'hex'
                    });
                } else {
                    _this.setState({
                        view: 'rgb'
                    });
                }
            }
        };
        _this.handleChange = function(data, e) {
            if (data.hex) {
                _color.isValidHex(data.hex) && _this.props.onChange({
                    hex: data.hex,
                    source: 'hex'
                }, e);
            } else if (data.r || data.g || data.b) {
                _this.props.onChange({
                    r: data.r || _this.props.rgb.r,
                    g: data.g || _this.props.rgb.g,
                    b: data.b || _this.props.rgb.b,
                    source: 'rgb'
                }, e);
            } else if (data.a) {
                if (data.a < 0) {
                    data.a = 0;
                } else if (data.a > 1) {
                    data.a = 1;
                }
                _this.props.onChange({
                    h: _this.props.hsl.h,
                    s: _this.props.hsl.s,
                    l: _this.props.hsl.l,
                    a: Math.round(data.a * 100) / 100,
                    source: 'rgb'
                }, e);
            } else if (data.h || data.s || data.l) {
                // Remove any occurances of '%'.
                if (typeof data.s === 'string' && data.s.includes('%')) {
                    data.s = data.s.replace('%', '');
                }
                if (typeof data.l === 'string' && data.l.includes('%')) {
                    data.l = data.l.replace('%', '');
                }
                // We store HSL as a unit interval so we need to override the 1 input to 0.01
                if (data.s == 1) {
                    data.s = 0.01;
                } else if (data.l == 1) {
                    data.l = 0.01;
                }
                _this.props.onChange({
                    h: data.h || _this.props.hsl.h,
                    s: Number(!(0, _isUndefined.default)(data.s) ? data.s : _this.props.hsl.s),
                    l: Number(!(0, _isUndefined.default)(data.l) ? data.l : _this.props.hsl.l),
                    source: 'hsl'
                }, e);
            }
        };
        _this.showHighlight = function(e) {
            e.currentTarget.style.background = '#eee';
        };
        _this.hideHighlight = function(e) {
            e.currentTarget.style.background = 'transparent';
        };
        if (props.hsl.a !== 1 && props.view === "hex") {
            _this.state = {
                view: "rgb"
            };
        } else {
            _this.state = {
                view: props.view
            };
        }
        return _this;
    }
    _createClass(ChromeFields, [
        {
            key: 'render',
            value: function render() {
                var _this2 = this;
                var styles = (0, _reactcss.default)({
                    'default': {
                        wrap: {
                            paddingTop: '16px',
                            display: 'flex'
                        },
                        fields: {
                            flex: '1',
                            display: 'flex',
                            marginLeft: '-6px'
                        },
                        field: {
                            paddingLeft: '6px',
                            width: '100%'
                        },
                        alpha: {
                            paddingLeft: '6px',
                            width: '100%'
                        },
                        toggle: {
                            width: '32px',
                            textAlign: 'right',
                            position: 'relative'
                        },
                        icon: {
                            marginRight: '-4px',
                            marginTop: '12px',
                            cursor: 'pointer',
                            position: 'relative'
                        },
                        iconHighlight: {
                            position: 'absolute',
                            width: '24px',
                            height: '28px',
                            background: '#eee',
                            borderRadius: '4px',
                            top: '10px',
                            left: '12px',
                            display: 'none'
                        },
                        input: {
                            fontSize: '11px',
                            color: '#333',
                            width: '100%',
                            borderRadius: '2px',
                            border: 'none',
                            boxShadow: 'inset 0 0 0 1px #dadada',
                            height: '21px',
                            textAlign: 'center'
                        },
                        label: {
                            textTransform: 'uppercase',
                            fontSize: '11px',
                            lineHeight: '11px',
                            color: '#969696',
                            textAlign: 'center',
                            display: 'block',
                            marginTop: '12px'
                        },
                        svg: {
                            fill: '#333',
                            width: '24px',
                            height: '24px',
                            border: '1px transparent solid',
                            borderRadius: '5px'
                        }
                    },
                    'disableAlpha': {
                        alpha: {
                            display: 'none'
                        }
                    }
                }, this.props, this.state);
                var fields = void 0;
                if (this.state.view === 'hex') {
                    fields = _react.default.createElement('div', {
                        style: styles.fields,
                        className: 'flexbox-fix'
                    }, _react.default.createElement('div', {
                        style: styles.field
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 'hex',
                        value: this.props.hex,
                        onChange: this.handleChange
                    })));
                } else if (this.state.view === 'rgb') {
                    fields = _react.default.createElement('div', {
                        style: styles.fields,
                        className: 'flexbox-fix'
                    }, _react.default.createElement('div', {
                        style: styles.field
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 'r',
                        value: this.props.rgb.r,
                        onChange: this.handleChange
                    })), _react.default.createElement('div', {
                        style: styles.field
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 'g',
                        value: this.props.rgb.g,
                        onChange: this.handleChange
                    })), _react.default.createElement('div', {
                        style: styles.field
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 'b',
                        value: this.props.rgb.b,
                        onChange: this.handleChange
                    })), _react.default.createElement('div', {
                        style: styles.alpha
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 'a',
                        value: this.props.rgb.a,
                        arrowOffset: 0.01,
                        onChange: this.handleChange
                    })));
                } else if (this.state.view === 'hsl') {
                    fields = _react.default.createElement('div', {
                        style: styles.fields,
                        className: 'flexbox-fix'
                    }, _react.default.createElement('div', {
                        style: styles.field
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 'h',
                        value: Math.round(this.props.hsl.h),
                        onChange: this.handleChange
                    })), _react.default.createElement('div', {
                        style: styles.field
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 's',
                        value: Math.round(this.props.hsl.s * 100) + '%',
                        onChange: this.handleChange
                    })), _react.default.createElement('div', {
                        style: styles.field
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 'l',
                        value: Math.round(this.props.hsl.l * 100) + '%',
                        onChange: this.handleChange
                    })), _react.default.createElement('div', {
                        style: styles.alpha
                    }, _react.default.createElement(_common.EditableInput, {
                        style: {
                            input: styles.input,
                            label: styles.label
                        },
                        label: 'a',
                        value: this.props.hsl.a,
                        arrowOffset: 0.01,
                        onChange: this.handleChange
                    })));
                }
                return _react.default.createElement('div', {
                    style: styles.wrap,
                    className: 'flexbox-fix'
                }, fields, _react.default.createElement('div', {
                    style: styles.toggle
                }, _react.default.createElement('div', {
                    style: styles.icon,
                    onClick: this.toggleViews,
                    ref: function ref(icon) {
                        return _this2.icon = icon;
                    }
                }, _react.default.createElement(_UnfoldMoreHorizontalIcon.default, {
                    style: styles.svg,
                    onMouseOver: this.showHighlight,
                    onMouseEnter: this.showHighlight,
                    onMouseOut: this.hideHighlight
                }))));
            }
        }
    ], [
        {
            key: 'getDerivedStateFromProps',
            value: function getDerivedStateFromProps(nextProps, state) {
                if (nextProps.hsl.a !== 1 && state.view === 'hex') {
                    return {
                        view: 'rgb'
                    };
                }
                return null;
            }
        }
    ]);
    return ChromeFields;
}(_react.default.Component);
ChromeFields.defaultProps = {
    view: "hex"
};
const _default = ChromeFields;

},
"41498ac6": function(module, exports, farmRequire, farmDynamicRequire) {
/** Used to match a single whitespace character. */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var reWhitespace = /\s/;
/**
 * Used by `_.trim` and `_.trimEnd` to get the index of the last non-whitespace
 * character of `string`.
 *
 * @private
 * @param {string} string The string to inspect.
 * @returns {number} Returns the index of the last non-whitespace character.
 */ function trimmedEndIndex(string) {
    var index = string.length;
    while(index-- && reWhitespace.test(string.charAt(index))){}
    return index;
}
const _default = trimmedEndIndex;

},
"47a014c7": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _listCacheClear = /*#__PURE__*/ _interop_require_default._(farmRequire("e8467ae5"));
const _listCacheDelete = /*#__PURE__*/ _interop_require_default._(farmRequire("c8075e33"));
const _listCacheGet = /*#__PURE__*/ _interop_require_default._(farmRequire("2c52b90d"));
const _listCacheHas = /*#__PURE__*/ _interop_require_default._(farmRequire("8043bb60"));
const _listCacheSet = /*#__PURE__*/ _interop_require_default._(farmRequire("8352887b"));
/**
 * Creates an list cache object.
 *
 * @private
 * @constructor
 * @param {Array} [entries] The key-value pairs to cache.
 */ function ListCache(entries) {
    var index = -1, length = entries == null ? 0 : entries.length;
    this.clear();
    while(++index < length){
        var entry = entries[index];
        this.set(entry[0], entry[1]);
    }
}
// Add methods to `ListCache`.
ListCache.prototype.clear = _listCacheClear.default;
ListCache.prototype['delete'] = _listCacheDelete.default;
ListCache.prototype.get = _listCacheGet.default;
ListCache.prototype.has = _listCacheHas.default;
ListCache.prototype.set = _listCacheSet.default;
const _default = ListCache;

},
"4aa7e9f2": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _debounce = /*#__PURE__*/ _interop_require_default._(farmRequire("bda4f991"));
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
/** Error message constants. */ var FUNC_ERROR_TEXT = 'Expected a function';
/**
 * Creates a throttled function that only invokes `func` at most once per
 * every `wait` milliseconds. The throttled function comes with a `cancel`
 * method to cancel delayed `func` invocations and a `flush` method to
 * immediately invoke them. Provide `options` to indicate whether `func`
 * should be invoked on the leading and/or trailing edge of the `wait`
 * timeout. The `func` is invoked with the last arguments provided to the
 * throttled function. Subsequent calls to the throttled function return the
 * result of the last `func` invocation.
 *
 * **Note:** If `leading` and `trailing` options are `true`, `func` is
 * invoked on the trailing edge of the timeout only if the throttled function
 * is invoked more than once during the `wait` timeout.
 *
 * If `wait` is `0` and `leading` is `false`, `func` invocation is deferred
 * until to the next tick, similar to `setTimeout` with a timeout of `0`.
 *
 * See [David Corbacho's article](https://css-tricks.com/debouncing-throttling-explained-examples/)
 * for details over the differences between `_.throttle` and `_.debounce`.
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @category Function
 * @param {Function} func The function to throttle.
 * @param {number} [wait=0] The number of milliseconds to throttle invocations to.
 * @param {Object} [options={}] The options object.
 * @param {boolean} [options.leading=true]
 *  Specify invoking on the leading edge of the timeout.
 * @param {boolean} [options.trailing=true]
 *  Specify invoking on the trailing edge of the timeout.
 * @returns {Function} Returns the new throttled function.
 * @example
 *
 * // Avoid excessively updating the position while scrolling.
 * jQuery(window).on('scroll', _.throttle(updatePosition, 100));
 *
 * // Invoke `renewToken` when the click event is fired, but not more than once every 5 minutes.
 * var throttled = _.throttle(renewToken, 300000, { 'trailing': false });
 * jQuery(element).on('click', throttled);
 *
 * // Cancel the trailing throttled invocation.
 * jQuery(window).on('popstate', throttled.cancel);
 */ function throttle(func, wait, options) {
    var leading = true, trailing = true;
    if (typeof func != 'function') {
        throw new TypeError(FUNC_ERROR_TEXT);
    }
    if ((0, _isObject.default)(options)) {
        leading = 'leading' in options ? !!options.leading : leading;
        trailing = 'trailing' in options ? !!options.trailing : trailing;
    }
    return (0, _debounce.default)(func, wait, {
        'leading': leading,
        'maxWait': wait,
        'trailing': trailing
    });
}
const _default = throttle;

},
"4ffcc116": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _Symbol = /*#__PURE__*/ _interop_require_default._(farmRequire("df3b0a5b"));
const _getRawTag = /*#__PURE__*/ _interop_require_default._(farmRequire("023b1fe0"));
const _objectToString = /*#__PURE__*/ _interop_require_default._(farmRequire("3ed12767"));
/** `Object#toString` result references. */ var nullTag = '[object Null]', undefinedTag = '[object Undefined]';
/** Built-in value references. */ var symToStringTag = _Symbol.default ? _Symbol.default.toStringTag : undefined;
/**
 * The base implementation of `getTag` without fallbacks for buggy environments.
 *
 * @private
 * @param {*} value The value to query.
 * @returns {string} Returns the `toStringTag`.
 */ function baseGetTag(value) {
    if (value == null) {
        return value === undefined ? undefinedTag : nullTag;
    }
    return symToStringTag && symToStringTag in Object(value) ? (0, _getRawTag.default)(value) : (0, _objectToString.default)(value);
}
const _default = baseGetTag;

},
"5057e1a2": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Gets the value at `key`, unless `key` is "__proto__" or "constructor".
 *
 * @private
 * @param {Object} object The object to query.
 * @param {string} key The key of the property to get.
 * @returns {*} Returns the property value.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function safeGet(object, key) {
    if (key === 'constructor' && typeof object[key] === 'function') {
        return;
    }
    if (key == '__proto__') {
        return;
    }
    return object[key];
}
const _default = safeGet;

},
"509c9c80": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _getMapData = /*#__PURE__*/ _interop_require_default._(farmRequire("d6ffea67"));
/**
 * Checks if a map value for `key` exists.
 *
 * @private
 * @name has
 * @memberOf MapCache
 * @param {string} key The key of the entry to check.
 * @returns {boolean} Returns `true` if an entry for `key` exists, else `false`.
 */ function mapCacheHas(key) {
    return (0, _getMapData.default)(this, key).has(key);
}
const _default = mapCacheHas;

},
"50a4b614": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Copies the values of `source` to `array`.
 *
 * @private
 * @param {Array} source The array to copy values from.
 * @param {Array} [array=[]] The array to copy values to.
 * @returns {Array} Returns `array`.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function copyArray(source, array) {
    var index = -1, length = source.length;
    array || (array = Array(length));
    while(++index < length){
        array[index] = source[index];
    }
    return array;
}
const _default = copyArray;

},
"53dc8b2a": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _defineProperty = /*#__PURE__*/ _interop_require_default._(farmRequire("23f33b47"));
/**
 * The base implementation of `assignValue` and `assignMergeValue` without
 * value checks.
 *
 * @private
 * @param {Object} object The object to modify.
 * @param {string} key The key of the property to assign.
 * @param {*} value The value to assign.
 */ function baseAssignValue(object, key, value) {
    if (key == '__proto__' && _defineProperty.default) {
        (0, _defineProperty.default)(object, key, {
            'configurable': true,
            'enumerable': true,
            'value': value,
            'writable': true
        });
    } else {
        object[key] = value;
    }
}
const _default = baseAssignValue;

},
"5508b53f": function(module, exports, farmRequire, farmDynamicRequire) {
/** Used for built-in method references. */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var funcProto = Function.prototype;
/** Used to resolve the decompiled source of functions. */ var funcToString = funcProto.toString;
/**
 * Converts `func` to its source code.
 *
 * @private
 * @param {Function} func The function to convert.
 * @returns {string} Returns the source code.
 */ function toSource(func) {
    if (func != null) {
        try {
            return funcToString.call(func);
        } catch (e) {}
        try {
            return func + '';
        } catch (e) {}
    }
    return '';
}
const _default = toSource;

},
"553d97a1": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isFunction = /*#__PURE__*/ _interop_require_default._(farmRequire("7fd4ad6b"));
const _isMasked = /*#__PURE__*/ _interop_require_default._(farmRequire("05fd2918"));
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
const _toSource = /*#__PURE__*/ _interop_require_default._(farmRequire("5508b53f"));
/**
 * Used to match `RegExp`
 * [syntax characters](http://ecma-international.org/ecma-262/7.0/#sec-patterns).
 */ var reRegExpChar = /[\\^$.*+?()[\]{}|]/g;
/** Used to detect host constructors (Safari). */ var reIsHostCtor = /^\[object .+?Constructor\]$/;
/** Used for built-in method references. */ var funcProto = Function.prototype, objectProto = Object.prototype;
/** Used to resolve the decompiled source of functions. */ var funcToString = funcProto.toString;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/** Used to detect if a method is native. */ var reIsNative = RegExp('^' + funcToString.call(hasOwnProperty).replace(reRegExpChar, '\\$&').replace(/hasOwnProperty|(function).*?(?=\\\()| for .+?(?=\\\])/g, '$1.*?') + '$');
/**
 * The base implementation of `_.isNative` without bad shim checks.
 *
 * @private
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a native function,
 *  else `false`.
 */ function baseIsNative(value) {
    if (!(0, _isObject.default)(value) || (0, _isMasked.default)(value)) {
        return false;
    }
    var pattern = (0, _isFunction.default)(value) ? reIsNative : reIsHostCtor;
    return pattern.test((0, _toSource.default)(value));
}
const _default = baseIsNative;

},
"56568a41": function(module, exports, farmRequire, farmDynamicRequire) {
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
    EditableInput: function() {
        return EditableInput;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
var _createClass = function() {
    function defineProperties(target, props) {
        for(var i = 0; i < props.length; i++){
            var descriptor = props[i];
            descriptor.enumerable = descriptor.enumerable || false;
            descriptor.configurable = true;
            if ("value" in descriptor) descriptor.writable = true;
            Object.defineProperty(target, descriptor.key, descriptor);
        }
    }
    return function(Constructor, protoProps, staticProps) {
        if (protoProps) defineProperties(Constructor.prototype, protoProps);
        if (staticProps) defineProperties(Constructor, staticProps);
        return Constructor;
    };
}();
function _defineProperty(obj, key, value) {
    if (key in obj) {
        Object.defineProperty(obj, key, {
            value: value,
            enumerable: true,
            configurable: true,
            writable: true
        });
    } else {
        obj[key] = value;
    }
    return obj;
}
function _classCallCheck(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
function _possibleConstructorReturn(self, call) {
    if (!self) {
        throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
    }
    return call && (typeof call === "object" || typeof call === "function") ? call : self;
}
function _inherits(subClass, superClass) {
    if (typeof superClass !== "function" && superClass !== null) {
        throw new TypeError("Super expression must either be null or a function, not " + typeof superClass);
    }
    subClass.prototype = Object.create(superClass && superClass.prototype, {
        constructor: {
            value: subClass,
            enumerable: false,
            writable: true,
            configurable: true
        }
    });
    if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass;
}
var DEFAULT_ARROW_OFFSET = 1;
var UP_KEY_CODE = 38;
var DOWN_KEY_CODE = 40;
var VALID_KEY_CODES = [
    UP_KEY_CODE,
    DOWN_KEY_CODE
];
var isValidKeyCode = function isValidKeyCode(keyCode) {
    return VALID_KEY_CODES.indexOf(keyCode) > -1;
};
var getNumberValue = function getNumberValue(value) {
    return Number(String(value).replace(/%/g, ''));
};
var idCounter = 1;
var EditableInput = function(_ref) {
    _inherits(EditableInput, _ref);
    function EditableInput(props) {
        _classCallCheck(this, EditableInput);
        var _this = _possibleConstructorReturn(this, (EditableInput.__proto__ || Object.getPrototypeOf(EditableInput)).call(this));
        _this.handleBlur = function() {
            if (_this.state.blurValue) {
                _this.setState({
                    value: _this.state.blurValue,
                    blurValue: null
                });
            }
        };
        _this.handleChange = function(e) {
            _this.setUpdatedValue(e.target.value, e);
        };
        _this.handleKeyDown = function(e) {
            // In case `e.target.value` is a percentage remove the `%` character
            // and update accordingly with a percentage
            // https://github.com/casesandberg/react-color/issues/383
            var value = getNumberValue(e.target.value);
            if (!isNaN(value) && isValidKeyCode(e.keyCode)) {
                var offset = _this.getArrowOffset();
                var updatedValue = e.keyCode === UP_KEY_CODE ? value + offset : value - offset;
                _this.setUpdatedValue(updatedValue, e);
            }
        };
        _this.handleDrag = function(e) {
            if (_this.props.dragLabel) {
                var newValue = Math.round(_this.props.value + e.movementX);
                if (newValue >= 0 && newValue <= _this.props.dragMax) {
                    _this.props.onChange && _this.props.onChange(_this.getValueObjectWithLabel(newValue), e);
                }
            }
        };
        _this.handleMouseDown = function(e) {
            if (_this.props.dragLabel) {
                e.preventDefault();
                _this.handleDrag(e);
                window.addEventListener('mousemove', _this.handleDrag);
                window.addEventListener('mouseup', _this.handleMouseUp);
            }
        };
        _this.handleMouseUp = function() {
            _this.unbindEventListeners();
        };
        _this.unbindEventListeners = function() {
            window.removeEventListener('mousemove', _this.handleDrag);
            window.removeEventListener('mouseup', _this.handleMouseUp);
        };
        _this.state = {
            value: String(props.value).toUpperCase(),
            blurValue: String(props.value).toUpperCase()
        };
        _this.inputId = 'rc-editable-input-' + idCounter++;
        return _this;
    }
    _createClass(EditableInput, [
        {
            key: 'componentDidUpdate',
            value: function componentDidUpdate(prevProps, prevState) {
                if (this.props.value !== this.state.value && (prevProps.value !== this.props.value || prevState.value !== this.state.value)) {
                    if (this.input === document.activeElement) {
                        this.setState({
                            blurValue: String(this.props.value).toUpperCase()
                        });
                    } else {
                        this.setState({
                            value: String(this.props.value).toUpperCase(),
                            blurValue: !this.state.blurValue && String(this.props.value).toUpperCase()
                        });
                    }
                }
            }
        },
        {
            key: 'componentWillUnmount',
            value: function componentWillUnmount() {
                this.unbindEventListeners();
            }
        },
        {
            key: 'getValueObjectWithLabel',
            value: function getValueObjectWithLabel(value) {
                return _defineProperty({}, this.props.label, value);
            }
        },
        {
            key: 'getArrowOffset',
            value: function getArrowOffset() {
                return this.props.arrowOffset || DEFAULT_ARROW_OFFSET;
            }
        },
        {
            key: 'setUpdatedValue',
            value: function setUpdatedValue(value, e) {
                var onChangeValue = this.props.label ? this.getValueObjectWithLabel(value) : value;
                this.props.onChange && this.props.onChange(onChangeValue, e);
                this.setState({
                    value: value
                });
            }
        },
        {
            key: 'render',
            value: function render() {
                var _this2 = this;
                var styles = (0, _reactcss.default)({
                    'default': {
                        wrap: {
                            position: 'relative'
                        }
                    },
                    'user-override': {
                        wrap: this.props.style && this.props.style.wrap ? this.props.style.wrap : {},
                        input: this.props.style && this.props.style.input ? this.props.style.input : {},
                        label: this.props.style && this.props.style.label ? this.props.style.label : {}
                    },
                    'dragLabel-true': {
                        label: {
                            cursor: 'ew-resize'
                        }
                    }
                }, {
                    'user-override': true
                }, this.props);
                return _react.default.createElement('div', {
                    style: styles.wrap
                }, _react.default.createElement('input', {
                    id: this.inputId,
                    style: styles.input,
                    ref: function ref(input) {
                        return _this2.input = input;
                    },
                    value: this.state.value,
                    onKeyDown: this.handleKeyDown,
                    onChange: this.handleChange,
                    onBlur: this.handleBlur,
                    placeholder: this.props.placeholder,
                    spellCheck: 'false'
                }), this.props.label && !this.props.hideLabel ? _react.default.createElement('label', {
                    htmlFor: this.inputId,
                    style: styles.label,
                    onMouseDown: this.handleMouseDown
                }, this.props.label) : null);
            }
        }
    ]);
    return EditableInput;
}(_react.PureComponent || _react.Component);
const _default = EditableInput;

},
"577cafe7": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _eq = /*#__PURE__*/ _interop_require_default._(farmRequire("3e794592"));
const _isArrayLike = /*#__PURE__*/ _interop_require_default._(farmRequire("65cf6153"));
const _isIndex = /*#__PURE__*/ _interop_require_default._(farmRequire("6f06a530"));
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
/**
 * Checks if the given arguments are from an iteratee call.
 *
 * @private
 * @param {*} value The potential iteratee value argument.
 * @param {*} index The potential iteratee index or key argument.
 * @param {*} object The potential iteratee object argument.
 * @returns {boolean} Returns `true` if the arguments are from an iteratee call,
 *  else `false`.
 */ function isIterateeCall(value, index, object) {
    if (!(0, _isObject.default)(object)) {
        return false;
    }
    var type = typeof index;
    if (type == 'number' ? (0, _isArrayLike.default)(object) && (0, _isIndex.default)(index, object.length) : type == 'string' && index in object) {
        return (0, _eq.default)(object[index], value);
    }
    return false;
}
const _default = isIterateeCall;

},
"5a62e586": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _freeGlobal = /*#__PURE__*/ _interop_require_default._(farmRequire("c5dbc97f"));
/** Detect free variable `exports`. */ var freeExports = typeof exports == 'object' && exports && !exports.nodeType && exports;
/** Detect free variable `module`. */ var freeModule = freeExports && typeof module == 'object' && module && !module.nodeType && module;
/** Detect the popular CommonJS extension `module.exports`. */ var moduleExports = freeModule && freeModule.exports === freeExports;
/** Detect free variable `process` from Node.js. */ var freeProcess = moduleExports && _freeGlobal.default.process;
/** Used to access faster Node.js helpers. */ var nodeUtil = function() {
    try {
        // Use `util.types` for Node.js 10+.
        var types = freeModule && freeModule.require && freeModule.require('util').types;
        if (types) {
            return types;
        }
        // Legacy `process.binding('util')` for Node.js < 10.
        return freeProcess && freeProcess.binding && freeProcess.binding('util');
    } catch (e) {}
}();
const _default = nodeUtil;

},
"5a9e6c9a": function(module, exports, farmRequire, farmDynamicRequire) {
// This file is autogenerated. It's used to publish CJS to npm.
(function(global, factory) {
    typeof exports === 'object' && typeof module !== 'undefined' ? module.exports = factory() : typeof define === 'function' && define.amd ? define(factory) : (global = typeof globalThis !== 'undefined' ? globalThis : global || self, global.tinycolor = factory());
})(this, function() {
    'use strict';
    function _typeof(obj) {
        "@babel/helpers - typeof";
        return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(obj) {
            return typeof obj;
        } : function(obj) {
            return obj && "function" == typeof Symbol && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj;
        }, _typeof(obj);
    }
    // https://github.com/bgrins/TinyColor
    // Brian Grinstead, MIT License
    var trimLeft = /^\s+/;
    var trimRight = /\s+$/;
    function tinycolor(color, opts) {
        color = color ? color : "";
        opts = opts || {};
        // If input is already a tinycolor, return itself
        if (color instanceof tinycolor) {
            return color;
        }
        // If we are called as a function, call using new instead
        if (!(this instanceof tinycolor)) {
            return new tinycolor(color, opts);
        }
        var rgb = inputToRGB(color);
        this._originalInput = color, this._r = rgb.r, this._g = rgb.g, this._b = rgb.b, this._a = rgb.a, this._roundA = Math.round(100 * this._a) / 100, this._format = opts.format || rgb.format;
        this._gradientType = opts.gradientType;
        // Don't let the range of [0,255] come back in [0,1].
        // Potentially lose a little bit of precision here, but will fix issues where
        // .5 gets interpreted as half of the total, instead of half of 1
        // If it was supposed to be 128, this was already taken care of by `inputToRgb`
        if (this._r < 1) this._r = Math.round(this._r);
        if (this._g < 1) this._g = Math.round(this._g);
        if (this._b < 1) this._b = Math.round(this._b);
        this._ok = rgb.ok;
    }
    tinycolor.prototype = {
        isDark: function isDark() {
            return this.getBrightness() < 128;
        },
        isLight: function isLight() {
            return !this.isDark();
        },
        isValid: function isValid() {
            return this._ok;
        },
        getOriginalInput: function getOriginalInput() {
            return this._originalInput;
        },
        getFormat: function getFormat() {
            return this._format;
        },
        getAlpha: function getAlpha() {
            return this._a;
        },
        getBrightness: function getBrightness() {
            //http://www.w3.org/TR/AERT#color-contrast
            var rgb = this.toRgb();
            return (rgb.r * 299 + rgb.g * 587 + rgb.b * 114) / 1000;
        },
        getLuminance: function getLuminance() {
            //http://www.w3.org/TR/2008/REC-WCAG20-20081211/#relativeluminancedef
            var rgb = this.toRgb();
            var RsRGB, GsRGB, BsRGB, R, G, B;
            RsRGB = rgb.r / 255;
            GsRGB = rgb.g / 255;
            BsRGB = rgb.b / 255;
            if (RsRGB <= 0.03928) R = RsRGB / 12.92;
            else R = Math.pow((RsRGB + 0.055) / 1.055, 2.4);
            if (GsRGB <= 0.03928) G = GsRGB / 12.92;
            else G = Math.pow((GsRGB + 0.055) / 1.055, 2.4);
            if (BsRGB <= 0.03928) B = BsRGB / 12.92;
            else B = Math.pow((BsRGB + 0.055) / 1.055, 2.4);
            return 0.2126 * R + 0.7152 * G + 0.0722 * B;
        },
        setAlpha: function setAlpha(value) {
            this._a = boundAlpha(value);
            this._roundA = Math.round(100 * this._a) / 100;
            return this;
        },
        toHsv: function toHsv() {
            var hsv = rgbToHsv(this._r, this._g, this._b);
            return {
                h: hsv.h * 360,
                s: hsv.s,
                v: hsv.v,
                a: this._a
            };
        },
        toHsvString: function toHsvString() {
            var hsv = rgbToHsv(this._r, this._g, this._b);
            var h = Math.round(hsv.h * 360), s = Math.round(hsv.s * 100), v = Math.round(hsv.v * 100);
            return this._a == 1 ? "hsv(" + h + ", " + s + "%, " + v + "%)" : "hsva(" + h + ", " + s + "%, " + v + "%, " + this._roundA + ")";
        },
        toHsl: function toHsl() {
            var hsl = rgbToHsl(this._r, this._g, this._b);
            return {
                h: hsl.h * 360,
                s: hsl.s,
                l: hsl.l,
                a: this._a
            };
        },
        toHslString: function toHslString() {
            var hsl = rgbToHsl(this._r, this._g, this._b);
            var h = Math.round(hsl.h * 360), s = Math.round(hsl.s * 100), l = Math.round(hsl.l * 100);
            return this._a == 1 ? "hsl(" + h + ", " + s + "%, " + l + "%)" : "hsla(" + h + ", " + s + "%, " + l + "%, " + this._roundA + ")";
        },
        toHex: function toHex(allow3Char) {
            return rgbToHex(this._r, this._g, this._b, allow3Char);
        },
        toHexString: function toHexString(allow3Char) {
            return "#" + this.toHex(allow3Char);
        },
        toHex8: function toHex8(allow4Char) {
            return rgbaToHex(this._r, this._g, this._b, this._a, allow4Char);
        },
        toHex8String: function toHex8String(allow4Char) {
            return "#" + this.toHex8(allow4Char);
        },
        toRgb: function toRgb() {
            return {
                r: Math.round(this._r),
                g: Math.round(this._g),
                b: Math.round(this._b),
                a: this._a
            };
        },
        toRgbString: function toRgbString() {
            return this._a == 1 ? "rgb(" + Math.round(this._r) + ", " + Math.round(this._g) + ", " + Math.round(this._b) + ")" : "rgba(" + Math.round(this._r) + ", " + Math.round(this._g) + ", " + Math.round(this._b) + ", " + this._roundA + ")";
        },
        toPercentageRgb: function toPercentageRgb() {
            return {
                r: Math.round(bound01(this._r, 255) * 100) + "%",
                g: Math.round(bound01(this._g, 255) * 100) + "%",
                b: Math.round(bound01(this._b, 255) * 100) + "%",
                a: this._a
            };
        },
        toPercentageRgbString: function toPercentageRgbString() {
            return this._a == 1 ? "rgb(" + Math.round(bound01(this._r, 255) * 100) + "%, " + Math.round(bound01(this._g, 255) * 100) + "%, " + Math.round(bound01(this._b, 255) * 100) + "%)" : "rgba(" + Math.round(bound01(this._r, 255) * 100) + "%, " + Math.round(bound01(this._g, 255) * 100) + "%, " + Math.round(bound01(this._b, 255) * 100) + "%, " + this._roundA + ")";
        },
        toName: function toName() {
            if (this._a === 0) {
                return "transparent";
            }
            if (this._a < 1) {
                return false;
            }
            return hexNames[rgbToHex(this._r, this._g, this._b, true)] || false;
        },
        toFilter: function toFilter(secondColor) {
            var hex8String = "#" + rgbaToArgbHex(this._r, this._g, this._b, this._a);
            var secondHex8String = hex8String;
            var gradientType = this._gradientType ? "GradientType = 1, " : "";
            if (secondColor) {
                var s = tinycolor(secondColor);
                secondHex8String = "#" + rgbaToArgbHex(s._r, s._g, s._b, s._a);
            }
            return "progid:DXImageTransform.Microsoft.gradient(" + gradientType + "startColorstr=" + hex8String + ",endColorstr=" + secondHex8String + ")";
        },
        toString: function toString(format) {
            var formatSet = !!format;
            format = format || this._format;
            var formattedString = false;
            var hasAlpha = this._a < 1 && this._a >= 0;
            var needsAlphaFormat = !formatSet && hasAlpha && (format === "hex" || format === "hex6" || format === "hex3" || format === "hex4" || format === "hex8" || format === "name");
            if (needsAlphaFormat) {
                // Special case for "transparent", all other non-alpha formats
                // will return rgba when there is transparency.
                if (format === "name" && this._a === 0) {
                    return this.toName();
                }
                return this.toRgbString();
            }
            if (format === "rgb") {
                formattedString = this.toRgbString();
            }
            if (format === "prgb") {
                formattedString = this.toPercentageRgbString();
            }
            if (format === "hex" || format === "hex6") {
                formattedString = this.toHexString();
            }
            if (format === "hex3") {
                formattedString = this.toHexString(true);
            }
            if (format === "hex4") {
                formattedString = this.toHex8String(true);
            }
            if (format === "hex8") {
                formattedString = this.toHex8String();
            }
            if (format === "name") {
                formattedString = this.toName();
            }
            if (format === "hsl") {
                formattedString = this.toHslString();
            }
            if (format === "hsv") {
                formattedString = this.toHsvString();
            }
            return formattedString || this.toHexString();
        },
        clone: function clone() {
            return tinycolor(this.toString());
        },
        _applyModification: function _applyModification(fn, args) {
            var color = fn.apply(null, [
                this
            ].concat([].slice.call(args)));
            this._r = color._r;
            this._g = color._g;
            this._b = color._b;
            this.setAlpha(color._a);
            return this;
        },
        lighten: function lighten() {
            return this._applyModification(_lighten, arguments);
        },
        brighten: function brighten() {
            return this._applyModification(_brighten, arguments);
        },
        darken: function darken() {
            return this._applyModification(_darken, arguments);
        },
        desaturate: function desaturate() {
            return this._applyModification(_desaturate, arguments);
        },
        saturate: function saturate() {
            return this._applyModification(_saturate, arguments);
        },
        greyscale: function greyscale() {
            return this._applyModification(_greyscale, arguments);
        },
        spin: function spin() {
            return this._applyModification(_spin, arguments);
        },
        _applyCombination: function _applyCombination(fn, args) {
            return fn.apply(null, [
                this
            ].concat([].slice.call(args)));
        },
        analogous: function analogous() {
            return this._applyCombination(_analogous, arguments);
        },
        complement: function complement() {
            return this._applyCombination(_complement, arguments);
        },
        monochromatic: function monochromatic() {
            return this._applyCombination(_monochromatic, arguments);
        },
        splitcomplement: function splitcomplement() {
            return this._applyCombination(_splitcomplement, arguments);
        },
        // Disabled until https://github.com/bgrins/TinyColor/issues/254
        // polyad: function (number) {
        //   return this._applyCombination(polyad, [number]);
        // },
        triad: function triad() {
            return this._applyCombination(polyad, [
                3
            ]);
        },
        tetrad: function tetrad() {
            return this._applyCombination(polyad, [
                4
            ]);
        }
    };
    // If input is an object, force 1 into "1.0" to handle ratios properly
    // String input requires "1.0" as input, so 1 will be treated as 1
    tinycolor.fromRatio = function(color, opts) {
        if (_typeof(color) == "object") {
            var newColor = {};
            for(var i in color){
                if (color.hasOwnProperty(i)) {
                    if (i === "a") {
                        newColor[i] = color[i];
                    } else {
                        newColor[i] = convertToPercentage(color[i]);
                    }
                }
            }
            color = newColor;
        }
        return tinycolor(color, opts);
    };
    // Given a string or object, convert that input to RGB
    // Possible string inputs:
    //
    //     "red"
    //     "#f00" or "f00"
    //     "#ff0000" or "ff0000"
    //     "#ff000000" or "ff000000"
    //     "rgb 255 0 0" or "rgb (255, 0, 0)"
    //     "rgb 1.0 0 0" or "rgb (1, 0, 0)"
    //     "rgba (255, 0, 0, 1)" or "rgba 255, 0, 0, 1"
    //     "rgba (1.0, 0, 0, 1)" or "rgba 1.0, 0, 0, 1"
    //     "hsl(0, 100%, 50%)" or "hsl 0 100% 50%"
    //     "hsla(0, 100%, 50%, 1)" or "hsla 0 100% 50%, 1"
    //     "hsv(0, 100%, 100%)" or "hsv 0 100% 100%"
    //
    function inputToRGB(color) {
        var rgb = {
            r: 0,
            g: 0,
            b: 0
        };
        var a = 1;
        var s = null;
        var v = null;
        var l = null;
        var ok = false;
        var format = false;
        if (typeof color == "string") {
            color = stringInputToObject(color);
        }
        if (_typeof(color) == "object") {
            if (isValidCSSUnit(color.r) && isValidCSSUnit(color.g) && isValidCSSUnit(color.b)) {
                rgb = rgbToRgb(color.r, color.g, color.b);
                ok = true;
                format = String(color.r).substr(-1) === "%" ? "prgb" : "rgb";
            } else if (isValidCSSUnit(color.h) && isValidCSSUnit(color.s) && isValidCSSUnit(color.v)) {
                s = convertToPercentage(color.s);
                v = convertToPercentage(color.v);
                rgb = hsvToRgb(color.h, s, v);
                ok = true;
                format = "hsv";
            } else if (isValidCSSUnit(color.h) && isValidCSSUnit(color.s) && isValidCSSUnit(color.l)) {
                s = convertToPercentage(color.s);
                l = convertToPercentage(color.l);
                rgb = hslToRgb(color.h, s, l);
                ok = true;
                format = "hsl";
            }
            if (color.hasOwnProperty("a")) {
                a = color.a;
            }
        }
        a = boundAlpha(a);
        return {
            ok: ok,
            format: color.format || format,
            r: Math.min(255, Math.max(rgb.r, 0)),
            g: Math.min(255, Math.max(rgb.g, 0)),
            b: Math.min(255, Math.max(rgb.b, 0)),
            a: a
        };
    }
    // Conversion Functions
    // --------------------
    // `rgbToHsl`, `rgbToHsv`, `hslToRgb`, `hsvToRgb` modified from:
    // <http://mjijackson.com/2008/02/rgb-to-hsl-and-rgb-to-hsv-color-model-conversion-algorithms-in-javascript>
    // `rgbToRgb`
    // Handle bounds / percentage checking to conform to CSS color spec
    // <http://www.w3.org/TR/css3-color/>
    // *Assumes:* r, g, b in [0, 255] or [0, 1]
    // *Returns:* { r, g, b } in [0, 255]
    function rgbToRgb(r, g, b) {
        return {
            r: bound01(r, 255) * 255,
            g: bound01(g, 255) * 255,
            b: bound01(b, 255) * 255
        };
    }
    // `rgbToHsl`
    // Converts an RGB color value to HSL.
    // *Assumes:* r, g, and b are contained in [0, 255] or [0, 1]
    // *Returns:* { h, s, l } in [0,1]
    function rgbToHsl(r, g, b) {
        r = bound01(r, 255);
        g = bound01(g, 255);
        b = bound01(b, 255);
        var max = Math.max(r, g, b), min = Math.min(r, g, b);
        var h, s, l = (max + min) / 2;
        if (max == min) {
            h = s = 0; // achromatic
        } else {
            var d = max - min;
            s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
            switch(max){
                case r:
                    h = (g - b) / d + (g < b ? 6 : 0);
                    break;
                case g:
                    h = (b - r) / d + 2;
                    break;
                case b:
                    h = (r - g) / d + 4;
                    break;
            }
            h /= 6;
        }
        return {
            h: h,
            s: s,
            l: l
        };
    }
    // `hslToRgb`
    // Converts an HSL color value to RGB.
    // *Assumes:* h is contained in [0, 1] or [0, 360] and s and l are contained [0, 1] or [0, 100]
    // *Returns:* { r, g, b } in the set [0, 255]
    function hslToRgb(h, s, l) {
        var r, g, b;
        h = bound01(h, 360);
        s = bound01(s, 100);
        l = bound01(l, 100);
        function hue2rgb(p, q, t) {
            if (t < 0) t += 1;
            if (t > 1) t -= 1;
            if (t < 1 / 6) return p + (q - p) * 6 * t;
            if (t < 1 / 2) return q;
            if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
            return p;
        }
        if (s === 0) {
            r = g = b = l; // achromatic
        } else {
            var q = l < 0.5 ? l * (1 + s) : l + s - l * s;
            var p = 2 * l - q;
            r = hue2rgb(p, q, h + 1 / 3);
            g = hue2rgb(p, q, h);
            b = hue2rgb(p, q, h - 1 / 3);
        }
        return {
            r: r * 255,
            g: g * 255,
            b: b * 255
        };
    }
    // `rgbToHsv`
    // Converts an RGB color value to HSV
    // *Assumes:* r, g, and b are contained in the set [0, 255] or [0, 1]
    // *Returns:* { h, s, v } in [0,1]
    function rgbToHsv(r, g, b) {
        r = bound01(r, 255);
        g = bound01(g, 255);
        b = bound01(b, 255);
        var max = Math.max(r, g, b), min = Math.min(r, g, b);
        var h, s, v = max;
        var d = max - min;
        s = max === 0 ? 0 : d / max;
        if (max == min) {
            h = 0; // achromatic
        } else {
            switch(max){
                case r:
                    h = (g - b) / d + (g < b ? 6 : 0);
                    break;
                case g:
                    h = (b - r) / d + 2;
                    break;
                case b:
                    h = (r - g) / d + 4;
                    break;
            }
            h /= 6;
        }
        return {
            h: h,
            s: s,
            v: v
        };
    }
    // `hsvToRgb`
    // Converts an HSV color value to RGB.
    // *Assumes:* h is contained in [0, 1] or [0, 360] and s and v are contained in [0, 1] or [0, 100]
    // *Returns:* { r, g, b } in the set [0, 255]
    function hsvToRgb(h, s, v) {
        h = bound01(h, 360) * 6;
        s = bound01(s, 100);
        v = bound01(v, 100);
        var i = Math.floor(h), f = h - i, p = v * (1 - s), q = v * (1 - f * s), t = v * (1 - (1 - f) * s), mod = i % 6, r = [
            v,
            q,
            p,
            p,
            t,
            v
        ][mod], g = [
            t,
            v,
            v,
            q,
            p,
            p
        ][mod], b = [
            p,
            p,
            t,
            v,
            v,
            q
        ][mod];
        return {
            r: r * 255,
            g: g * 255,
            b: b * 255
        };
    }
    // `rgbToHex`
    // Converts an RGB color to hex
    // Assumes r, g, and b are contained in the set [0, 255]
    // Returns a 3 or 6 character hex
    function rgbToHex(r, g, b, allow3Char) {
        var hex = [
            pad2(Math.round(r).toString(16)),
            pad2(Math.round(g).toString(16)),
            pad2(Math.round(b).toString(16))
        ];
        // Return a 3 character hex if possible
        if (allow3Char && hex[0].charAt(0) == hex[0].charAt(1) && hex[1].charAt(0) == hex[1].charAt(1) && hex[2].charAt(0) == hex[2].charAt(1)) {
            return hex[0].charAt(0) + hex[1].charAt(0) + hex[2].charAt(0);
        }
        return hex.join("");
    }
    // `rgbaToHex`
    // Converts an RGBA color plus alpha transparency to hex
    // Assumes r, g, b are contained in the set [0, 255] and
    // a in [0, 1]. Returns a 4 or 8 character rgba hex
    function rgbaToHex(r, g, b, a, allow4Char) {
        var hex = [
            pad2(Math.round(r).toString(16)),
            pad2(Math.round(g).toString(16)),
            pad2(Math.round(b).toString(16)),
            pad2(convertDecimalToHex(a))
        ];
        // Return a 4 character hex if possible
        if (allow4Char && hex[0].charAt(0) == hex[0].charAt(1) && hex[1].charAt(0) == hex[1].charAt(1) && hex[2].charAt(0) == hex[2].charAt(1) && hex[3].charAt(0) == hex[3].charAt(1)) {
            return hex[0].charAt(0) + hex[1].charAt(0) + hex[2].charAt(0) + hex[3].charAt(0);
        }
        return hex.join("");
    }
    // `rgbaToArgbHex`
    // Converts an RGBA color to an ARGB Hex8 string
    // Rarely used, but required for "toFilter()"
    function rgbaToArgbHex(r, g, b, a) {
        var hex = [
            pad2(convertDecimalToHex(a)),
            pad2(Math.round(r).toString(16)),
            pad2(Math.round(g).toString(16)),
            pad2(Math.round(b).toString(16))
        ];
        return hex.join("");
    }
    // `equals`
    // Can be called with any tinycolor input
    tinycolor.equals = function(color1, color2) {
        if (!color1 || !color2) return false;
        return tinycolor(color1).toRgbString() == tinycolor(color2).toRgbString();
    };
    tinycolor.random = function() {
        return tinycolor.fromRatio({
            r: Math.random(),
            g: Math.random(),
            b: Math.random()
        });
    };
    // Modification Functions
    // ----------------------
    // Thanks to less.js for some of the basics here
    // <https://github.com/cloudhead/less.js/blob/master/lib/less/functions.js>
    function _desaturate(color, amount) {
        amount = amount === 0 ? 0 : amount || 10;
        var hsl = tinycolor(color).toHsl();
        hsl.s -= amount / 100;
        hsl.s = clamp01(hsl.s);
        return tinycolor(hsl);
    }
    function _saturate(color, amount) {
        amount = amount === 0 ? 0 : amount || 10;
        var hsl = tinycolor(color).toHsl();
        hsl.s += amount / 100;
        hsl.s = clamp01(hsl.s);
        return tinycolor(hsl);
    }
    function _greyscale(color) {
        return tinycolor(color).desaturate(100);
    }
    function _lighten(color, amount) {
        amount = amount === 0 ? 0 : amount || 10;
        var hsl = tinycolor(color).toHsl();
        hsl.l += amount / 100;
        hsl.l = clamp01(hsl.l);
        return tinycolor(hsl);
    }
    function _brighten(color, amount) {
        amount = amount === 0 ? 0 : amount || 10;
        var rgb = tinycolor(color).toRgb();
        rgb.r = Math.max(0, Math.min(255, rgb.r - Math.round(255 * -(amount / 100))));
        rgb.g = Math.max(0, Math.min(255, rgb.g - Math.round(255 * -(amount / 100))));
        rgb.b = Math.max(0, Math.min(255, rgb.b - Math.round(255 * -(amount / 100))));
        return tinycolor(rgb);
    }
    function _darken(color, amount) {
        amount = amount === 0 ? 0 : amount || 10;
        var hsl = tinycolor(color).toHsl();
        hsl.l -= amount / 100;
        hsl.l = clamp01(hsl.l);
        return tinycolor(hsl);
    }
    // Spin takes a positive or negative amount within [-360, 360] indicating the change of hue.
    // Values outside of this range will be wrapped into this range.
    function _spin(color, amount) {
        var hsl = tinycolor(color).toHsl();
        var hue = (hsl.h + amount) % 360;
        hsl.h = hue < 0 ? 360 + hue : hue;
        return tinycolor(hsl);
    }
    // Combination Functions
    // ---------------------
    // Thanks to jQuery xColor for some of the ideas behind these
    // <https://github.com/infusion/jQuery-xcolor/blob/master/jquery.xcolor.js>
    function _complement(color) {
        var hsl = tinycolor(color).toHsl();
        hsl.h = (hsl.h + 180) % 360;
        return tinycolor(hsl);
    }
    function polyad(color, number) {
        if (isNaN(number) || number <= 0) {
            throw new Error("Argument to polyad must be a positive number");
        }
        var hsl = tinycolor(color).toHsl();
        var result = [
            tinycolor(color)
        ];
        var step = 360 / number;
        for(var i = 1; i < number; i++){
            result.push(tinycolor({
                h: (hsl.h + i * step) % 360,
                s: hsl.s,
                l: hsl.l
            }));
        }
        return result;
    }
    function _splitcomplement(color) {
        var hsl = tinycolor(color).toHsl();
        var h = hsl.h;
        return [
            tinycolor(color),
            tinycolor({
                h: (h + 72) % 360,
                s: hsl.s,
                l: hsl.l
            }),
            tinycolor({
                h: (h + 216) % 360,
                s: hsl.s,
                l: hsl.l
            })
        ];
    }
    function _analogous(color, results, slices) {
        results = results || 6;
        slices = slices || 30;
        var hsl = tinycolor(color).toHsl();
        var part = 360 / slices;
        var ret = [
            tinycolor(color)
        ];
        for(hsl.h = (hsl.h - (part * results >> 1) + 720) % 360; --results;){
            hsl.h = (hsl.h + part) % 360;
            ret.push(tinycolor(hsl));
        }
        return ret;
    }
    function _monochromatic(color, results) {
        results = results || 6;
        var hsv = tinycolor(color).toHsv();
        var h = hsv.h, s = hsv.s, v = hsv.v;
        var ret = [];
        var modification = 1 / results;
        while(results--){
            ret.push(tinycolor({
                h: h,
                s: s,
                v: v
            }));
            v = (v + modification) % 1;
        }
        return ret;
    }
    // Utility Functions
    // ---------------------
    tinycolor.mix = function(color1, color2, amount) {
        amount = amount === 0 ? 0 : amount || 50;
        var rgb1 = tinycolor(color1).toRgb();
        var rgb2 = tinycolor(color2).toRgb();
        var p = amount / 100;
        var rgba = {
            r: (rgb2.r - rgb1.r) * p + rgb1.r,
            g: (rgb2.g - rgb1.g) * p + rgb1.g,
            b: (rgb2.b - rgb1.b) * p + rgb1.b,
            a: (rgb2.a - rgb1.a) * p + rgb1.a
        };
        return tinycolor(rgba);
    };
    // Readability Functions
    // ---------------------
    // <http://www.w3.org/TR/2008/REC-WCAG20-20081211/#contrast-ratiodef (WCAG Version 2)
    // `contrast`
    // Analyze the 2 colors and returns the color contrast defined by (WCAG Version 2)
    tinycolor.readability = function(color1, color2) {
        var c1 = tinycolor(color1);
        var c2 = tinycolor(color2);
        return (Math.max(c1.getLuminance(), c2.getLuminance()) + 0.05) / (Math.min(c1.getLuminance(), c2.getLuminance()) + 0.05);
    };
    // `isReadable`
    // Ensure that foreground and background color combinations meet WCAG2 guidelines.
    // The third argument is an optional Object.
    //      the 'level' property states 'AA' or 'AAA' - if missing or invalid, it defaults to 'AA';
    //      the 'size' property states 'large' or 'small' - if missing or invalid, it defaults to 'small'.
    // If the entire object is absent, isReadable defaults to {level:"AA",size:"small"}.
    // *Example*
    //    tinycolor.isReadable("#000", "#111") => false
    //    tinycolor.isReadable("#000", "#111",{level:"AA",size:"large"}) => false
    tinycolor.isReadable = function(color1, color2, wcag2) {
        var readability = tinycolor.readability(color1, color2);
        var wcag2Parms, out;
        out = false;
        wcag2Parms = validateWCAG2Parms(wcag2);
        switch(wcag2Parms.level + wcag2Parms.size){
            case "AAsmall":
            case "AAAlarge":
                out = readability >= 4.5;
                break;
            case "AAlarge":
                out = readability >= 3;
                break;
            case "AAAsmall":
                out = readability >= 7;
                break;
        }
        return out;
    };
    // `mostReadable`
    // Given a base color and a list of possible foreground or background
    // colors for that base, returns the most readable color.
    // Optionally returns Black or White if the most readable color is unreadable.
    // *Example*
    //    tinycolor.mostReadable(tinycolor.mostReadable("#123", ["#124", "#125"],{includeFallbackColors:false}).toHexString(); // "#112255"
    //    tinycolor.mostReadable(tinycolor.mostReadable("#123", ["#124", "#125"],{includeFallbackColors:true}).toHexString();  // "#ffffff"
    //    tinycolor.mostReadable("#a8015a", ["#faf3f3"],{includeFallbackColors:true,level:"AAA",size:"large"}).toHexString(); // "#faf3f3"
    //    tinycolor.mostReadable("#a8015a", ["#faf3f3"],{includeFallbackColors:true,level:"AAA",size:"small"}).toHexString(); // "#ffffff"
    tinycolor.mostReadable = function(baseColor, colorList, args) {
        var bestColor = null;
        var bestScore = 0;
        var readability;
        var includeFallbackColors, level, size;
        args = args || {};
        includeFallbackColors = args.includeFallbackColors;
        level = args.level;
        size = args.size;
        for(var i = 0; i < colorList.length; i++){
            readability = tinycolor.readability(baseColor, colorList[i]);
            if (readability > bestScore) {
                bestScore = readability;
                bestColor = tinycolor(colorList[i]);
            }
        }
        if (tinycolor.isReadable(baseColor, bestColor, {
            level: level,
            size: size
        }) || !includeFallbackColors) {
            return bestColor;
        } else {
            args.includeFallbackColors = false;
            return tinycolor.mostReadable(baseColor, [
                "#fff",
                "#000"
            ], args);
        }
    };
    // Big List of Colors
    // ------------------
    // <https://www.w3.org/TR/css-color-4/#named-colors>
    var names = tinycolor.names = {
        aliceblue: "f0f8ff",
        antiquewhite: "faebd7",
        aqua: "0ff",
        aquamarine: "7fffd4",
        azure: "f0ffff",
        beige: "f5f5dc",
        bisque: "ffe4c4",
        black: "000",
        blanchedalmond: "ffebcd",
        blue: "00f",
        blueviolet: "8a2be2",
        brown: "a52a2a",
        burlywood: "deb887",
        burntsienna: "ea7e5d",
        cadetblue: "5f9ea0",
        chartreuse: "7fff00",
        chocolate: "d2691e",
        coral: "ff7f50",
        cornflowerblue: "6495ed",
        cornsilk: "fff8dc",
        crimson: "dc143c",
        cyan: "0ff",
        darkblue: "00008b",
        darkcyan: "008b8b",
        darkgoldenrod: "b8860b",
        darkgray: "a9a9a9",
        darkgreen: "006400",
        darkgrey: "a9a9a9",
        darkkhaki: "bdb76b",
        darkmagenta: "8b008b",
        darkolivegreen: "556b2f",
        darkorange: "ff8c00",
        darkorchid: "9932cc",
        darkred: "8b0000",
        darksalmon: "e9967a",
        darkseagreen: "8fbc8f",
        darkslateblue: "483d8b",
        darkslategray: "2f4f4f",
        darkslategrey: "2f4f4f",
        darkturquoise: "00ced1",
        darkviolet: "9400d3",
        deeppink: "ff1493",
        deepskyblue: "00bfff",
        dimgray: "696969",
        dimgrey: "696969",
        dodgerblue: "1e90ff",
        firebrick: "b22222",
        floralwhite: "fffaf0",
        forestgreen: "228b22",
        fuchsia: "f0f",
        gainsboro: "dcdcdc",
        ghostwhite: "f8f8ff",
        gold: "ffd700",
        goldenrod: "daa520",
        gray: "808080",
        green: "008000",
        greenyellow: "adff2f",
        grey: "808080",
        honeydew: "f0fff0",
        hotpink: "ff69b4",
        indianred: "cd5c5c",
        indigo: "4b0082",
        ivory: "fffff0",
        khaki: "f0e68c",
        lavender: "e6e6fa",
        lavenderblush: "fff0f5",
        lawngreen: "7cfc00",
        lemonchiffon: "fffacd",
        lightblue: "add8e6",
        lightcoral: "f08080",
        lightcyan: "e0ffff",
        lightgoldenrodyellow: "fafad2",
        lightgray: "d3d3d3",
        lightgreen: "90ee90",
        lightgrey: "d3d3d3",
        lightpink: "ffb6c1",
        lightsalmon: "ffa07a",
        lightseagreen: "20b2aa",
        lightskyblue: "87cefa",
        lightslategray: "789",
        lightslategrey: "789",
        lightsteelblue: "b0c4de",
        lightyellow: "ffffe0",
        lime: "0f0",
        limegreen: "32cd32",
        linen: "faf0e6",
        magenta: "f0f",
        maroon: "800000",
        mediumaquamarine: "66cdaa",
        mediumblue: "0000cd",
        mediumorchid: "ba55d3",
        mediumpurple: "9370db",
        mediumseagreen: "3cb371",
        mediumslateblue: "7b68ee",
        mediumspringgreen: "00fa9a",
        mediumturquoise: "48d1cc",
        mediumvioletred: "c71585",
        midnightblue: "191970",
        mintcream: "f5fffa",
        mistyrose: "ffe4e1",
        moccasin: "ffe4b5",
        navajowhite: "ffdead",
        navy: "000080",
        oldlace: "fdf5e6",
        olive: "808000",
        olivedrab: "6b8e23",
        orange: "ffa500",
        orangered: "ff4500",
        orchid: "da70d6",
        palegoldenrod: "eee8aa",
        palegreen: "98fb98",
        paleturquoise: "afeeee",
        palevioletred: "db7093",
        papayawhip: "ffefd5",
        peachpuff: "ffdab9",
        peru: "cd853f",
        pink: "ffc0cb",
        plum: "dda0dd",
        powderblue: "b0e0e6",
        purple: "800080",
        rebeccapurple: "663399",
        red: "f00",
        rosybrown: "bc8f8f",
        royalblue: "4169e1",
        saddlebrown: "8b4513",
        salmon: "fa8072",
        sandybrown: "f4a460",
        seagreen: "2e8b57",
        seashell: "fff5ee",
        sienna: "a0522d",
        silver: "c0c0c0",
        skyblue: "87ceeb",
        slateblue: "6a5acd",
        slategray: "708090",
        slategrey: "708090",
        snow: "fffafa",
        springgreen: "00ff7f",
        steelblue: "4682b4",
        tan: "d2b48c",
        teal: "008080",
        thistle: "d8bfd8",
        tomato: "ff6347",
        turquoise: "40e0d0",
        violet: "ee82ee",
        wheat: "f5deb3",
        white: "fff",
        whitesmoke: "f5f5f5",
        yellow: "ff0",
        yellowgreen: "9acd32"
    };
    // Make it easy to access colors via `hexNames[hex]`
    var hexNames = tinycolor.hexNames = flip(names);
    // Utilities
    // ---------
    // `{ 'name1': 'val1' }` becomes `{ 'val1': 'name1' }`
    function flip(o) {
        var flipped = {};
        for(var i in o){
            if (o.hasOwnProperty(i)) {
                flipped[o[i]] = i;
            }
        }
        return flipped;
    }
    // Return a valid alpha value [0,1] with all invalid values being set to 1
    function boundAlpha(a) {
        a = parseFloat(a);
        if (isNaN(a) || a < 0 || a > 1) {
            a = 1;
        }
        return a;
    }
    // Take input from [0, n] and return it as [0, 1]
    function bound01(n, max) {
        if (isOnePointZero(n)) n = "100%";
        var processPercent = isPercentage(n);
        n = Math.min(max, Math.max(0, parseFloat(n)));
        // Automatically convert percentage into number
        if (processPercent) {
            n = parseInt(n * max, 10) / 100;
        }
        // Handle floating point rounding errors
        if (Math.abs(n - max) < 0.000001) {
            return 1;
        }
        // Convert into [0, 1] range if it isn't already
        return n % max / parseFloat(max);
    }
    // Force a number between 0 and 1
    function clamp01(val) {
        return Math.min(1, Math.max(0, val));
    }
    // Parse a base-16 hex value into a base-10 integer
    function parseIntFromHex(val) {
        return parseInt(val, 16);
    }
    // Need to handle 1.0 as 100%, since once it is a number, there is no difference between it and 1
    // <http://stackoverflow.com/questions/7422072/javascript-how-to-detect-number-as-a-decimal-including-1-0>
    function isOnePointZero(n) {
        return typeof n == "string" && n.indexOf(".") != -1 && parseFloat(n) === 1;
    }
    // Check to see if string passed in is a percentage
    function isPercentage(n) {
        return typeof n === "string" && n.indexOf("%") != -1;
    }
    // Force a hex value to have 2 characters
    function pad2(c) {
        return c.length == 1 ? "0" + c : "" + c;
    }
    // Replace a decimal with it's percentage value
    function convertToPercentage(n) {
        if (n <= 1) {
            n = n * 100 + "%";
        }
        return n;
    }
    // Converts a decimal to a hex value
    function convertDecimalToHex(d) {
        return Math.round(parseFloat(d) * 255).toString(16);
    }
    // Converts a hex value to a decimal
    function convertHexToDecimal(h) {
        return parseIntFromHex(h) / 255;
    }
    var matchers = function() {
        // <http://www.w3.org/TR/css3-values/#integers>
        var CSS_INTEGER = "[-\\+]?\\d+%?";
        // <http://www.w3.org/TR/css3-values/#number-value>
        var CSS_NUMBER = "[-\\+]?\\d*\\.\\d+%?";
        // Allow positive/negative integer/number.  Don't capture the either/or, just the entire outcome.
        var CSS_UNIT = "(?:" + CSS_NUMBER + ")|(?:" + CSS_INTEGER + ")";
        // Actual matching.
        // Parentheses and commas are optional, but not required.
        // Whitespace can take the place of commas or opening paren
        var PERMISSIVE_MATCH3 = "[\\s|\\(]+(" + CSS_UNIT + ")[,|\\s]+(" + CSS_UNIT + ")[,|\\s]+(" + CSS_UNIT + ")\\s*\\)?";
        var PERMISSIVE_MATCH4 = "[\\s|\\(]+(" + CSS_UNIT + ")[,|\\s]+(" + CSS_UNIT + ")[,|\\s]+(" + CSS_UNIT + ")[,|\\s]+(" + CSS_UNIT + ")\\s*\\)?";
        return {
            CSS_UNIT: new RegExp(CSS_UNIT),
            rgb: new RegExp("rgb" + PERMISSIVE_MATCH3),
            rgba: new RegExp("rgba" + PERMISSIVE_MATCH4),
            hsl: new RegExp("hsl" + PERMISSIVE_MATCH3),
            hsla: new RegExp("hsla" + PERMISSIVE_MATCH4),
            hsv: new RegExp("hsv" + PERMISSIVE_MATCH3),
            hsva: new RegExp("hsva" + PERMISSIVE_MATCH4),
            hex3: /^#?([0-9a-fA-F]{1})([0-9a-fA-F]{1})([0-9a-fA-F]{1})$/,
            hex6: /^#?([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/,
            hex4: /^#?([0-9a-fA-F]{1})([0-9a-fA-F]{1})([0-9a-fA-F]{1})([0-9a-fA-F]{1})$/,
            hex8: /^#?([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/
        };
    }();
    // `isValidCSSUnit`
    // Take in a single string / number and check to see if it looks like a CSS unit
    // (see `matchers` above for definition).
    function isValidCSSUnit(color) {
        return !!matchers.CSS_UNIT.exec(color);
    }
    // `stringInputToObject`
    // Permissive string parsing.  Take in a number of formats, and output an object
    // based on detected format.  Returns `{ r, g, b }` or `{ h, s, l }` or `{ h, s, v}`
    function stringInputToObject(color) {
        color = color.replace(trimLeft, "").replace(trimRight, "").toLowerCase();
        var named = false;
        if (names[color]) {
            color = names[color];
            named = true;
        } else if (color == "transparent") {
            return {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
                format: "name"
            };
        }
        // Try to match string input using regular expressions.
        // Keep most of the number bounding out of this function - don't worry about [0,1] or [0,100] or [0,360]
        // Just return an object and let the conversion functions handle that.
        // This way the result will be the same whether the tinycolor is initialized with string or object.
        var match;
        if (match = matchers.rgb.exec(color)) {
            return {
                r: match[1],
                g: match[2],
                b: match[3]
            };
        }
        if (match = matchers.rgba.exec(color)) {
            return {
                r: match[1],
                g: match[2],
                b: match[3],
                a: match[4]
            };
        }
        if (match = matchers.hsl.exec(color)) {
            return {
                h: match[1],
                s: match[2],
                l: match[3]
            };
        }
        if (match = matchers.hsla.exec(color)) {
            return {
                h: match[1],
                s: match[2],
                l: match[3],
                a: match[4]
            };
        }
        if (match = matchers.hsv.exec(color)) {
            return {
                h: match[1],
                s: match[2],
                v: match[3]
            };
        }
        if (match = matchers.hsva.exec(color)) {
            return {
                h: match[1],
                s: match[2],
                v: match[3],
                a: match[4]
            };
        }
        if (match = matchers.hex8.exec(color)) {
            return {
                r: parseIntFromHex(match[1]),
                g: parseIntFromHex(match[2]),
                b: parseIntFromHex(match[3]),
                a: convertHexToDecimal(match[4]),
                format: named ? "name" : "hex8"
            };
        }
        if (match = matchers.hex6.exec(color)) {
            return {
                r: parseIntFromHex(match[1]),
                g: parseIntFromHex(match[2]),
                b: parseIntFromHex(match[3]),
                format: named ? "name" : "hex"
            };
        }
        if (match = matchers.hex4.exec(color)) {
            return {
                r: parseIntFromHex(match[1] + "" + match[1]),
                g: parseIntFromHex(match[2] + "" + match[2]),
                b: parseIntFromHex(match[3] + "" + match[3]),
                a: convertHexToDecimal(match[4] + "" + match[4]),
                format: named ? "name" : "hex8"
            };
        }
        if (match = matchers.hex3.exec(color)) {
            return {
                r: parseIntFromHex(match[1] + "" + match[1]),
                g: parseIntFromHex(match[2] + "" + match[2]),
                b: parseIntFromHex(match[3] + "" + match[3]),
                format: named ? "name" : "hex"
            };
        }
        return false;
    }
    function validateWCAG2Parms(parms) {
        // return valid WCAG2 parms for isReadable.
        // If input parms are invalid, return {"level":"AA", "size":"small"}
        var level, size;
        parms = parms || {
            level: "AA",
            size: "small"
        };
        level = (parms.level || "AA").toUpperCase();
        size = (parms.size || "small").toLowerCase();
        if (level !== "AA" && level !== "AAA") {
            level = "AA";
        }
        if (size !== "small" && size !== "large") {
            size = "small";
        }
        return {
            level: level,
            size: size
        };
    }
    return tinycolor;
});

},
"5aa5795a": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _getNative = /*#__PURE__*/ _interop_require_default._(farmRequire("7d9eff84"));
const _root = /*#__PURE__*/ _interop_require_default._(farmRequire("6bf7b771"));
/* Built-in method references that are verified to be native. */ var Map = (0, _getNative.default)(_root.default, 'Map');
const _default = Map;

},
"5be84108": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _ListCache = /*#__PURE__*/ _interop_require_default._(farmRequire("47a014c7"));
const _Map = /*#__PURE__*/ _interop_require_default._(farmRequire("5aa5795a"));
const _MapCache = /*#__PURE__*/ _interop_require_default._(farmRequire("f96f20cf"));
/** Used as the size to enable large array optimizations. */ var LARGE_ARRAY_SIZE = 200;
/**
 * Sets the stack `key` to `value`.
 *
 * @private
 * @name set
 * @memberOf Stack
 * @param {string} key The key of the value to set.
 * @param {*} value The value to set.
 * @returns {Object} Returns the stack cache instance.
 */ function stackSet(key, value) {
    var data = this.__data__;
    if (data instanceof _ListCache.default) {
        var pairs = data.__data__;
        if (!_Map.default || pairs.length < LARGE_ARRAY_SIZE - 1) {
            pairs.push([
                key,
                value
            ]);
            this.size = ++data.size;
            return this;
        }
        data = this.__data__ = new _MapCache.default(pairs);
    }
    data.set(key, value);
    this.size = data.size;
    return this;
}
const _default = stackSet;

},
"5f1cd00b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseGetTag = /*#__PURE__*/ _interop_require_default._(farmRequire("4ffcc116"));
const _isObjectLike = /*#__PURE__*/ _interop_require_default._(farmRequire("1954b1ef"));
/** `Object#toString` result references. */ var argsTag = '[object Arguments]';
/**
 * The base implementation of `_.isArguments`.
 *
 * @private
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is an `arguments` object,
 */ function baseIsArguments(value) {
    return (0, _isObjectLike.default)(value) && (0, _baseGetTag.default)(value) == argsTag;
}
const _default = baseIsArguments;

},
"61567a8f": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Alpha: function() {
        return Alpha;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _alpha = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("b3b7aa28"));
const _Checkboard = /*#__PURE__*/ _interop_require_default._(farmRequire("83168a07"));
var _extends = Object.assign || function(target) {
    for(var i = 1; i < arguments.length; i++){
        var source = arguments[i];
        for(var key in source){
            if (Object.prototype.hasOwnProperty.call(source, key)) {
                target[key] = source[key];
            }
        }
    }
    return target;
};
var _createClass = function() {
    function defineProperties(target, props) {
        for(var i = 0; i < props.length; i++){
            var descriptor = props[i];
            descriptor.enumerable = descriptor.enumerable || false;
            descriptor.configurable = true;
            if ("value" in descriptor) descriptor.writable = true;
            Object.defineProperty(target, descriptor.key, descriptor);
        }
    }
    return function(Constructor, protoProps, staticProps) {
        if (protoProps) defineProperties(Constructor.prototype, protoProps);
        if (staticProps) defineProperties(Constructor, staticProps);
        return Constructor;
    };
}();
function _classCallCheck(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
function _possibleConstructorReturn(self, call) {
    if (!self) {
        throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
    }
    return call && (typeof call === "object" || typeof call === "function") ? call : self;
}
function _inherits(subClass, superClass) {
    if (typeof superClass !== "function" && superClass !== null) {
        throw new TypeError("Super expression must either be null or a function, not " + typeof superClass);
    }
    subClass.prototype = Object.create(superClass && superClass.prototype, {
        constructor: {
            value: subClass,
            enumerable: false,
            writable: true,
            configurable: true
        }
    });
    if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass;
}
var Alpha = function(_ref) {
    _inherits(Alpha, _ref);
    function Alpha() {
        var _ref2;
        var _temp, _this, _ret;
        _classCallCheck(this, Alpha);
        for(var _len = arguments.length, args = Array(_len), _key = 0; _key < _len; _key++){
            args[_key] = arguments[_key];
        }
        return _ret = (_temp = (_this = _possibleConstructorReturn(this, (_ref2 = Alpha.__proto__ || Object.getPrototypeOf(Alpha)).call.apply(_ref2, [
            this
        ].concat(args))), _this), _this.handleChange = function(e) {
            var change = _alpha.calculateChange(e, _this.props.hsl, _this.props.direction, _this.props.a, _this.container);
            change && typeof _this.props.onChange === 'function' && _this.props.onChange(change, e);
        }, _this.handleMouseDown = function(e) {
            _this.handleChange(e);
            window.addEventListener('mousemove', _this.handleChange);
            window.addEventListener('mouseup', _this.handleMouseUp);
        }, _this.handleMouseUp = function() {
            _this.unbindEventListeners();
        }, _this.unbindEventListeners = function() {
            window.removeEventListener('mousemove', _this.handleChange);
            window.removeEventListener('mouseup', _this.handleMouseUp);
        }, _temp), _possibleConstructorReturn(_this, _ret);
    }
    _createClass(Alpha, [
        {
            key: 'componentWillUnmount',
            value: function componentWillUnmount() {
                this.unbindEventListeners();
            }
        },
        {
            key: 'render',
            value: function render() {
                var _this2 = this;
                var rgb = this.props.rgb;
                var styles = (0, _reactcss.default)({
                    'default': {
                        alpha: {
                            absolute: '0px 0px 0px 0px',
                            borderRadius: this.props.radius
                        },
                        checkboard: {
                            absolute: '0px 0px 0px 0px',
                            overflow: 'hidden',
                            borderRadius: this.props.radius
                        },
                        gradient: {
                            absolute: '0px 0px 0px 0px',
                            background: 'linear-gradient(to right, rgba(' + rgb.r + ',' + rgb.g + ',' + rgb.b + ', 0) 0%,\n           rgba(' + rgb.r + ',' + rgb.g + ',' + rgb.b + ', 1) 100%)',
                            boxShadow: this.props.shadow,
                            borderRadius: this.props.radius
                        },
                        container: {
                            position: 'relative',
                            height: '100%',
                            margin: '0 3px'
                        },
                        pointer: {
                            position: 'absolute',
                            left: rgb.a * 100 + '%'
                        },
                        slider: {
                            width: '4px',
                            borderRadius: '1px',
                            height: '8px',
                            boxShadow: '0 0 2px rgba(0, 0, 0, .6)',
                            background: '#fff',
                            marginTop: '1px',
                            transform: 'translateX(-2px)'
                        }
                    },
                    'vertical': {
                        gradient: {
                            background: 'linear-gradient(to bottom, rgba(' + rgb.r + ',' + rgb.g + ',' + rgb.b + ', 0) 0%,\n           rgba(' + rgb.r + ',' + rgb.g + ',' + rgb.b + ', 1) 100%)'
                        },
                        pointer: {
                            left: 0,
                            top: rgb.a * 100 + '%'
                        }
                    },
                    'overwrite': _extends({}, this.props.style)
                }, {
                    vertical: this.props.direction === 'vertical',
                    overwrite: true
                });
                return _react.default.createElement('div', {
                    style: styles.alpha
                }, _react.default.createElement('div', {
                    style: styles.checkboard
                }, _react.default.createElement(_Checkboard.default, {
                    renderers: this.props.renderers
                })), _react.default.createElement('div', {
                    style: styles.gradient
                }), _react.default.createElement('div', {
                    style: styles.container,
                    ref: function ref(container) {
                        return _this2.container = container;
                    },
                    onMouseDown: this.handleMouseDown,
                    onTouchMove: this.handleChange,
                    onTouchStart: this.handleChange
                }, _react.default.createElement('div', {
                    style: styles.pointer
                }, this.props.pointer ? _react.default.createElement(this.props.pointer, this.props) : _react.default.createElement('div', {
                    style: styles.slider
                }))));
            }
        }
    ]);
    return Alpha;
}(_react.PureComponent || _react.Component);
const _default = Alpha;

},
"6287afc4": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isArrayLike = /*#__PURE__*/ _interop_require_default._(farmRequire("65cf6153"));
/**
 * Creates a `baseEach` or `baseEachRight` function.
 *
 * @private
 * @param {Function} eachFunc The function to iterate over a collection.
 * @param {boolean} [fromRight] Specify iterating from right to left.
 * @returns {Function} Returns the new base function.
 */ function createBaseEach(eachFunc, fromRight) {
    return function(collection, iteratee) {
        if (collection == null) {
            return collection;
        }
        if (!(0, _isArrayLike.default)(collection)) {
            return eachFunc(collection, iteratee);
        }
        var length = collection.length, index = fromRight ? length : -1, iterable = Object(collection);
        while(fromRight ? index-- : ++index < length){
            if (iteratee(iterable[index], index, iterable) === false) {
                break;
            }
        }
        return collection;
    };
}
const _default = createBaseEach;

},
"65cf6153": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isFunction = /*#__PURE__*/ _interop_require_default._(farmRequire("7fd4ad6b"));
const _isLength = /*#__PURE__*/ _interop_require_default._(farmRequire("2031bec1"));
/**
 * Checks if `value` is array-like. A value is considered array-like if it's
 * not a function and has a `value.length` that's an integer greater than or
 * equal to `0` and less than or equal to `Number.MAX_SAFE_INTEGER`.
 *
 * @static
 * @memberOf _
 * @since 4.0.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is array-like, else `false`.
 * @example
 *
 * _.isArrayLike([1, 2, 3]);
 * // => true
 *
 * _.isArrayLike(document.body.children);
 * // => true
 *
 * _.isArrayLike('abc');
 * // => true
 *
 * _.isArrayLike(_.noop);
 * // => false
 */ function isArrayLike(value) {
    return value != null && (0, _isLength.default)(value.length) && !(0, _isFunction.default)(value);
}
const _default = isArrayLike;

},
"66b9aa18": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseFor = /*#__PURE__*/ _interop_require_default._(farmRequire("91fbac55"));
const _keys = /*#__PURE__*/ _interop_require_default._(farmRequire("095ade58"));
/**
 * The base implementation of `_.forOwn` without support for iteratee shorthands.
 *
 * @private
 * @param {Object} object The object to iterate over.
 * @param {Function} iteratee The function invoked per iteration.
 * @returns {Object} Returns `object`.
 */ function baseForOwn(object, iteratee) {
    return object && (0, _baseFor.default)(object, iteratee, _keys.default);
}
const _default = baseForOwn;

},
"6bf7b771": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _freeGlobal = /*#__PURE__*/ _interop_require_default._(farmRequire("c5dbc97f"));
/** Detect free variable `self`. */ var freeSelf = typeof self == 'object' && self && self.Object === Object && self;
/** Used as a reference to the global object. */ var root = _freeGlobal.default || freeSelf || Function('return this')();
const _default = root;

},
"6bfe3296": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Chrome: function() {
        return Chrome;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _proptypes = /*#__PURE__*/ _interop_require_default._(farmRequire("75a1a40c"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _merge = /*#__PURE__*/ _interop_require_default._(farmRequire("30490595"));
const _common = farmRequire("0c79d683");
const _ChromeFields = /*#__PURE__*/ _interop_require_default._(farmRequire("3f23c083"));
const _ChromePointer = /*#__PURE__*/ _interop_require_default._(farmRequire("da2fd2b6"));
const _ChromePointerCircle = /*#__PURE__*/ _interop_require_default._(farmRequire("1485eb1e"));
var Chrome = function Chrome(_ref) {
    var width = _ref.width, onChange = _ref.onChange, disableAlpha = _ref.disableAlpha, rgb = _ref.rgb, hsl = _ref.hsl, hsv = _ref.hsv, hex = _ref.hex, renderers = _ref.renderers, _ref$styles = _ref.styles, passedStyles = _ref$styles === undefined ? {} : _ref$styles, _ref$className = _ref.className, className = _ref$className === undefined ? '' : _ref$className, defaultView = _ref.defaultView;
    var styles = (0, _reactcss.default)((0, _merge.default)({
        'default': {
            picker: {
                width: width,
                background: '#fff',
                borderRadius: '2px',
                boxShadow: '0 0 2px rgba(0,0,0,.3), 0 4px 8px rgba(0,0,0,.3)',
                boxSizing: 'initial',
                fontFamily: 'Menlo'
            },
            saturation: {
                width: '100%',
                paddingBottom: '55%',
                position: 'relative',
                borderRadius: '2px 2px 0 0',
                overflow: 'hidden'
            },
            Saturation: {
                radius: '2px 2px 0 0'
            },
            body: {
                padding: '16px 16px 12px'
            },
            controls: {
                display: 'flex'
            },
            color: {
                width: '32px'
            },
            swatch: {
                marginTop: '6px',
                width: '16px',
                height: '16px',
                borderRadius: '8px',
                position: 'relative',
                overflow: 'hidden'
            },
            active: {
                absolute: '0px 0px 0px 0px',
                borderRadius: '8px',
                boxShadow: 'inset 0 0 0 1px rgba(0,0,0,.1)',
                background: 'rgba(' + rgb.r + ', ' + rgb.g + ', ' + rgb.b + ', ' + rgb.a + ')',
                zIndex: '2'
            },
            toggles: {
                flex: '1'
            },
            hue: {
                height: '10px',
                position: 'relative',
                marginBottom: '8px'
            },
            Hue: {
                radius: '2px'
            },
            alpha: {
                height: '10px',
                position: 'relative'
            },
            Alpha: {
                radius: '2px'
            }
        },
        'disableAlpha': {
            color: {
                width: '22px'
            },
            alpha: {
                display: 'none'
            },
            hue: {
                marginBottom: '0px'
            },
            swatch: {
                width: '10px',
                height: '10px',
                marginTop: '0px'
            }
        }
    }, passedStyles), {
        disableAlpha: disableAlpha
    });
    return _react.default.createElement('div', {
        style: styles.picker,
        className: 'chrome-picker ' + className
    }, _react.default.createElement('div', {
        style: styles.saturation
    }, _react.default.createElement(_common.Saturation, {
        style: styles.Saturation,
        hsl: hsl,
        hsv: hsv,
        pointer: _ChromePointerCircle.default,
        onChange: onChange
    })), _react.default.createElement('div', {
        style: styles.body
    }, _react.default.createElement('div', {
        style: styles.controls,
        className: 'flexbox-fix'
    }, _react.default.createElement('div', {
        style: styles.color
    }, _react.default.createElement('div', {
        style: styles.swatch
    }, _react.default.createElement('div', {
        style: styles.active
    }), _react.default.createElement(_common.Checkboard, {
        renderers: renderers
    }))), _react.default.createElement('div', {
        style: styles.toggles
    }, _react.default.createElement('div', {
        style: styles.hue
    }, _react.default.createElement(_common.Hue, {
        style: styles.Hue,
        hsl: hsl,
        pointer: _ChromePointer.default,
        onChange: onChange
    })), _react.default.createElement('div', {
        style: styles.alpha
    }, _react.default.createElement(_common.Alpha, {
        style: styles.Alpha,
        rgb: rgb,
        hsl: hsl,
        pointer: _ChromePointer.default,
        renderers: renderers,
        onChange: onChange
    })))), _react.default.createElement(_ChromeFields.default, {
        rgb: rgb,
        hsl: hsl,
        hex: hex,
        view: defaultView,
        onChange: onChange,
        disableAlpha: disableAlpha
    })));
};
Chrome.propTypes = {
    width: _proptypes.default.oneOfType([
        _proptypes.default.string,
        _proptypes.default.number
    ]),
    disableAlpha: _proptypes.default.bool,
    styles: _proptypes.default.object,
    defaultView: _proptypes.default.oneOf([
        "hex",
        "rgb",
        "hsl"
    ])
};
Chrome.defaultProps = {
    width: 225,
    disableAlpha: false,
    styles: {}
};
const _default = (0, _common.ColorWrap)(Chrome);

},
"6c30181d": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseTrim = /*#__PURE__*/ _interop_require_default._(farmRequire("d932635f"));
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
const _isSymbol = /*#__PURE__*/ _interop_require_default._(farmRequire("2074a2eb"));
/** Used as references for various `Number` constants. */ var NAN = 0 / 0;
/** Used to detect bad signed hexadecimal string values. */ var reIsBadHex = /^[-+]0x[0-9a-f]+$/i;
/** Used to detect binary string values. */ var reIsBinary = /^0b[01]+$/i;
/** Used to detect octal string values. */ var reIsOctal = /^0o[0-7]+$/i;
/** Built-in method references without a dependency on `root`. */ var freeParseInt = parseInt;
/**
 * Converts `value` to a number.
 *
 * @static
 * @memberOf _
 * @since 4.0.0
 * @category Lang
 * @param {*} value The value to process.
 * @returns {number} Returns the number.
 * @example
 *
 * _.toNumber(3.2);
 * // => 3.2
 *
 * _.toNumber(Number.MIN_VALUE);
 * // => 5e-324
 *
 * _.toNumber(Infinity);
 * // => Infinity
 *
 * _.toNumber('3.2');
 * // => 3.2
 */ function toNumber(value) {
    if (typeof value == 'number') {
        return value;
    }
    if ((0, _isSymbol.default)(value)) {
        return NAN;
    }
    if ((0, _isObject.default)(value)) {
        var other = typeof value.valueOf == 'function' ? value.valueOf() : value;
        value = (0, _isObject.default)(other) ? other + '' : other;
    }
    if (typeof value != 'string') {
        return value === 0 ? value : +value;
    }
    value = (0, _baseTrim.default)(value);
    var isBinary = reIsBinary.test(value);
    return isBinary || reIsOctal.test(value) ? freeParseInt(value.slice(2), isBinary ? 2 : 8) : reIsBadHex.test(value) ? NAN : +value;
}
const _default = toNumber;

},
"6d294af6": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _nativeCreate = /*#__PURE__*/ _interop_require_default._(farmRequire("dc218a61"));
/** Used to stand-in for `undefined` hash values. */ var HASH_UNDEFINED = '__lodash_hash_undefined__';
/**
 * Sets the hash `key` to `value`.
 *
 * @private
 * @name set
 * @memberOf Hash
 * @param {string} key The key of the value to set.
 * @param {*} value The value to set.
 * @returns {Object} Returns the hash instance.
 */ function hashSet(key, value) {
    var data = this.__data__;
    this.size += this.has(key) ? 0 : 1;
    data[key] = _nativeCreate.default && value === undefined ? HASH_UNDEFINED : value;
    return this;
}
const _default = hashSet;

},
"6f06a530": function(module, exports, farmRequire, farmDynamicRequire) {
/** Used as references for various `Number` constants. */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var MAX_SAFE_INTEGER = 9007199254740991;
/** Used to detect unsigned integer values. */ var reIsUint = /^(?:0|[1-9]\d*)$/;
/**
 * Checks if `value` is a valid array-like index.
 *
 * @private
 * @param {*} value The value to check.
 * @param {number} [length=MAX_SAFE_INTEGER] The upper bounds of a valid index.
 * @returns {boolean} Returns `true` if `value` is a valid index, else `false`.
 */ function isIndex(value, length) {
    var type = typeof value;
    length = length == null ? MAX_SAFE_INTEGER : length;
    return !!length && (type == 'number' || type != 'symbol' && reIsUint.test(value)) && value > -1 && value % 1 == 0 && value < length;
}
const _default = isIndex;

},
"70020fba": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _nativeCreate = /*#__PURE__*/ _interop_require_default._(farmRequire("dc218a61"));
/** Used to stand-in for `undefined` hash values. */ var HASH_UNDEFINED = '__lodash_hash_undefined__';
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/**
 * Gets the hash value for `key`.
 *
 * @private
 * @name get
 * @memberOf Hash
 * @param {string} key The key of the value to get.
 * @returns {*} Returns the entry value.
 */ function hashGet(key) {
    var data = this.__data__;
    if (_nativeCreate.default) {
        var result = data[key];
        return result === HASH_UNDEFINED ? undefined : result;
    }
    return hasOwnProperty.call(data, key) ? data[key] : undefined;
}
const _default = hashGet;

},
"70a9c618": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "calculateChange", {
    enumerable: true,
    get: function() {
        return calculateChange;
    }
});
var calculateChange = function calculateChange(e, direction, hsl, container) {
    var containerWidth = container.clientWidth;
    var containerHeight = container.clientHeight;
    var x = typeof e.pageX === 'number' ? e.pageX : e.touches[0].pageX;
    var y = typeof e.pageY === 'number' ? e.pageY : e.touches[0].pageY;
    var left = x - (container.getBoundingClientRect().left + window.pageXOffset);
    var top = y - (container.getBoundingClientRect().top + window.pageYOffset);
    if (direction === 'vertical') {
        var h = void 0;
        if (top < 0) {
            h = 359;
        } else if (top > containerHeight) {
            h = 0;
        } else {
            var percent = -(top * 100 / containerHeight) + 100;
            h = 360 * percent / 100;
        }
        if (hsl.h !== h) {
            return {
                h: h,
                s: hsl.s,
                l: hsl.l,
                a: hsl.a,
                source: 'hsl'
            };
        }
    } else {
        var _h = void 0;
        if (left < 0) {
            _h = 0;
        } else if (left > containerWidth) {
            _h = 359;
        } else {
            var _percent = left * 100 / containerWidth;
            _h = 360 * _percent / 100;
        }
        if (hsl.h !== _h) {
            return {
                h: _h,
                s: hsl.s,
                l: hsl.l,
                a: hsl.a,
                source: 'hsl'
            };
        }
    }
    return null;
};

},
"77e9a7bd": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _Uint8Array = /*#__PURE__*/ _interop_require_default._(farmRequire("cdbeebb3"));
/**
 * Creates a clone of `arrayBuffer`.
 *
 * @private
 * @param {ArrayBuffer} arrayBuffer The array buffer to clone.
 * @returns {ArrayBuffer} Returns the cloned array buffer.
 */ function cloneArrayBuffer(arrayBuffer) {
    var result = new arrayBuffer.constructor(arrayBuffer.byteLength);
    new _Uint8Array.default(result).set(new _Uint8Array.default(arrayBuffer));
    return result;
}
const _default = cloneArrayBuffer;

},
"77f602cd": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseAssignValue = /*#__PURE__*/ _interop_require_default._(farmRequire("53dc8b2a"));
const _eq = /*#__PURE__*/ _interop_require_default._(farmRequire("3e794592"));
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/**
 * Assigns `value` to `key` of `object` if the existing value is not equivalent
 * using [`SameValueZero`](http://ecma-international.org/ecma-262/7.0/#sec-samevaluezero)
 * for equality comparisons.
 *
 * @private
 * @param {Object} object The object to modify.
 * @param {string} key The key of the property to assign.
 * @param {*} value The value to assign.
 */ function assignValue(object, key, value) {
    var objValue = object[key];
    if (!(hasOwnProperty.call(object, key) && (0, _eq.default)(objValue, value)) || value === undefined && !(key in object)) {
        (0, _baseAssignValue.default)(object, key, value);
    }
}
const _default = assignValue;

},
"78b2e4fc": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Creates a function that returns `value`.
 *
 * @static
 * @memberOf _
 * @since 2.4.0
 * @category Util
 * @param {*} value The value to return from the new function.
 * @returns {Function} Returns the new constant function.
 * @example
 *
 * var objects = _.times(2, _.constant({ 'a': 1 }));
 *
 * console.log(objects);
 * // => [{ 'a': 1 }, { 'a': 1 }]
 *
 * console.log(objects[0] === objects[1]);
 * // => true
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function constant(value) {
    return function() {
        return value;
    };
}
const _default = constant;

},
"7bb75378": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * The base implementation of `_.times` without support for iteratee shorthands
 * or max array length checks.
 *
 * @private
 * @param {number} n The number of times to invoke `iteratee`.
 * @param {Function} iteratee The function invoked per iteration.
 * @returns {Array} Returns the array of results.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function baseTimes(n, iteratee) {
    var index = -1, result = Array(n);
    while(++index < n){
        result[index] = iteratee(index);
    }
    return result;
}
const _default = baseTimes;

},
"7d285119": function(module, exports, farmRequire, farmDynamicRequire) {
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
    SketchPresetColors: function() {
        return SketchPresetColors;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _proptypes = /*#__PURE__*/ _interop_require_default._(farmRequire("75a1a40c"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _common = farmRequire("0c79d683");
var _extends = Object.assign || function(target) {
    for(var i = 1; i < arguments.length; i++){
        var source = arguments[i];
        for(var key in source){
            if (Object.prototype.hasOwnProperty.call(source, key)) {
                target[key] = source[key];
            }
        }
    }
    return target;
};
var SketchPresetColors = function SketchPresetColors(_ref) {
    var colors = _ref.colors, _ref$onClick = _ref.onClick, onClick = _ref$onClick === undefined ? function() {} : _ref$onClick, onSwatchHover = _ref.onSwatchHover;
    var styles = (0, _reactcss.default)({
        'default': {
            colors: {
                margin: '0 -10px',
                padding: '10px 0 0 10px',
                borderTop: '1px solid #eee',
                display: 'flex',
                flexWrap: 'wrap',
                position: 'relative'
            },
            swatchWrap: {
                width: '16px',
                height: '16px',
                margin: '0 10px 10px 0'
            },
            swatch: {
                borderRadius: '3px',
                boxShadow: 'inset 0 0 0 1px rgba(0,0,0,.15)'
            }
        },
        'no-presets': {
            colors: {
                display: 'none'
            }
        }
    }, {
        'no-presets': !colors || !colors.length
    });
    var handleClick = function handleClick(hex, e) {
        onClick({
            hex: hex,
            source: 'hex'
        }, e);
    };
    return _react.default.createElement('div', {
        style: styles.colors,
        className: 'flexbox-fix'
    }, colors.map(function(colorObjOrString) {
        var c = typeof colorObjOrString === 'string' ? {
            color: colorObjOrString
        } : colorObjOrString;
        var key = '' + c.color + (c.title || '');
        return _react.default.createElement('div', {
            key: key,
            style: styles.swatchWrap
        }, _react.default.createElement(_common.Swatch, _extends({}, c, {
            style: styles.swatch,
            onClick: handleClick,
            onHover: onSwatchHover,
            focusStyle: {
                boxShadow: 'inset 0 0 0 1px rgba(0,0,0,.15), 0 0 4px ' + c.color
            }
        })));
    }));
};
SketchPresetColors.propTypes = {
    colors: _proptypes.default.arrayOf(_proptypes.default.oneOfType([
        _proptypes.default.string,
        _proptypes.default.shape({
            color: _proptypes.default.string,
            title: _proptypes.default.string
        })
    ])).isRequired
};
const _default = SketchPresetColors;

},
"7d8013fd": function(module, exports, farmRequire, farmDynamicRequire) {
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
    get: function() {
        return get;
    },
    render: function() {
        return render;
    }
});
var checkboardCache = {};
var render = function render(c1, c2, size, serverCanvas) {
    if (typeof document === 'undefined' && !serverCanvas) {
        return null;
    }
    var canvas = serverCanvas ? new serverCanvas() : document.createElement('canvas');
    canvas.width = size * 2;
    canvas.height = size * 2;
    var ctx = canvas.getContext('2d');
    if (!ctx) {
        return null;
    } // If no context can be found, return early.
    ctx.fillStyle = c1;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = c2;
    ctx.fillRect(0, 0, size, size);
    ctx.translate(size, size);
    ctx.fillRect(0, 0, size, size);
    return canvas.toDataURL();
};
var get = function get(c1, c2, size, serverCanvas) {
    var key = c1 + '-' + c2 + '-' + size + (serverCanvas ? '-server' : '');
    if (checkboardCache[key]) {
        return checkboardCache[key];
    }
    var checkboard = render(c1, c2, size, serverCanvas);
    checkboardCache[key] = checkboard;
    return checkboard;
};

},
"7d9eff84": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseIsNative = /*#__PURE__*/ _interop_require_default._(farmRequire("553d97a1"));
const _getValue = /*#__PURE__*/ _interop_require_default._(farmRequire("277986a6"));
/**
 * Gets the native function at `key` of `object`.
 *
 * @private
 * @param {Object} object The object to query.
 * @param {string} key The key of the method to get.
 * @returns {*} Returns the function if it's native, else `undefined`.
 */ function getNative(object, key) {
    var value = (0, _getValue.default)(object, key);
    return (0, _baseIsNative.default)(value) ? value : undefined;
}
const _default = getNative;

},
"7fd4ad6b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseGetTag = /*#__PURE__*/ _interop_require_default._(farmRequire("4ffcc116"));
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
/** `Object#toString` result references. */ var asyncTag = '[object AsyncFunction]', funcTag = '[object Function]', genTag = '[object GeneratorFunction]', proxyTag = '[object Proxy]';
/**
 * Checks if `value` is classified as a `Function` object.
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a function, else `false`.
 * @example
 *
 * _.isFunction(_);
 * // => true
 *
 * _.isFunction(/abc/);
 * // => false
 */ function isFunction(value) {
    if (!(0, _isObject.default)(value)) {
        return false;
    }
    // The use of `Object#toString` avoids issues with the `typeof` operator
    // in Safari 9 which returns 'object' for typed arrays and other constructors.
    var tag = (0, _baseGetTag.default)(value);
    return tag == funcTag || tag == genTag || tag == asyncTag || tag == proxyTag;
}
const _default = isFunction;

},
"8043bb60": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _assocIndexOf = /*#__PURE__*/ _interop_require_default._(farmRequire("9270f0e3"));
/**
 * Checks if a list cache value for `key` exists.
 *
 * @private
 * @name has
 * @memberOf ListCache
 * @param {string} key The key of the entry to check.
 * @returns {boolean} Returns `true` if an entry for `key` exists, else `false`.
 */ function listCacheHas(key) {
    return (0, _assocIndexOf.default)(this.__data__, key) > -1;
}
const _default = listCacheHas;

},
"80d7fb42": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isPrototype = /*#__PURE__*/ _interop_require_default._(farmRequire("ce92b344"));
const _nativeKeys = /*#__PURE__*/ _interop_require_default._(farmRequire("8e90d37b"));
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/**
 * The base implementation of `_.keys` which doesn't treat sparse arrays as dense.
 *
 * @private
 * @param {Object} object The object to query.
 * @returns {Array} Returns the array of property names.
 */ function baseKeys(object) {
    if (!(0, _isPrototype.default)(object)) {
        return (0, _nativeKeys.default)(object);
    }
    var result = [];
    for(var key in Object(object)){
        if (hasOwnProperty.call(object, key) && key != 'constructor') {
            result.push(key);
        }
    }
    return result;
}
const _default = baseKeys;

},
"811d4135": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _constant = /*#__PURE__*/ _interop_require_default._(farmRequire("78b2e4fc"));
const _defineProperty = /*#__PURE__*/ _interop_require_default._(farmRequire("23f33b47"));
const _identity = /*#__PURE__*/ _interop_require_default._(farmRequire("86d49f49"));
/**
 * The base implementation of `setToString` without support for hot loop shorting.
 *
 * @private
 * @param {Function} func The function to modify.
 * @param {Function} string The `toString` result.
 * @returns {Function} Returns `func`.
 */ var baseSetToString = !_defineProperty.default ? _identity.default : function(func, string) {
    return (0, _defineProperty.default)(func, 'toString', {
        'configurable': true,
        'enumerable': false,
        'value': (0, _constant.default)(string),
        'writable': true
    });
};
const _default = baseSetToString;

},
"813cc28a": function(module, exports, farmRequire, farmDynamicRequire) {
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
    ColorWrap: function() {
        return ColorWrap;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _debounce = /*#__PURE__*/ _interop_require_default._(farmRequire("bda4f991"));
const _color = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("889a06b2"));
var _extends = Object.assign || function(target) {
    for(var i = 1; i < arguments.length; i++){
        var source = arguments[i];
        for(var key in source){
            if (Object.prototype.hasOwnProperty.call(source, key)) {
                target[key] = source[key];
            }
        }
    }
    return target;
};
var _createClass = function() {
    function defineProperties(target, props) {
        for(var i = 0; i < props.length; i++){
            var descriptor = props[i];
            descriptor.enumerable = descriptor.enumerable || false;
            descriptor.configurable = true;
            if ("value" in descriptor) descriptor.writable = true;
            Object.defineProperty(target, descriptor.key, descriptor);
        }
    }
    return function(Constructor, protoProps, staticProps) {
        if (protoProps) defineProperties(Constructor.prototype, protoProps);
        if (staticProps) defineProperties(Constructor, staticProps);
        return Constructor;
    };
}();
function _classCallCheck(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
function _possibleConstructorReturn(self, call) {
    if (!self) {
        throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
    }
    return call && (typeof call === "object" || typeof call === "function") ? call : self;
}
function _inherits(subClass, superClass) {
    if (typeof superClass !== "function" && superClass !== null) {
        throw new TypeError("Super expression must either be null or a function, not " + typeof superClass);
    }
    subClass.prototype = Object.create(superClass && superClass.prototype, {
        constructor: {
            value: subClass,
            enumerable: false,
            writable: true,
            configurable: true
        }
    });
    if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass;
}
var ColorWrap = function ColorWrap(Picker) {
    var ColorPicker = function(_ref) {
        _inherits(ColorPicker, _ref);
        function ColorPicker(props) {
            _classCallCheck(this, ColorPicker);
            var _this = _possibleConstructorReturn(this, (ColorPicker.__proto__ || Object.getPrototypeOf(ColorPicker)).call(this));
            _this.handleChange = function(data, event) {
                var isValidColor = _color.simpleCheckForValidColor(data);
                if (isValidColor) {
                    var colors = _color.toState(data, data.h || _this.state.oldHue);
                    _this.setState(colors);
                    _this.props.onChangeComplete && _this.debounce(_this.props.onChangeComplete, colors, event);
                    _this.props.onChange && _this.props.onChange(colors, event);
                }
            };
            _this.handleSwatchHover = function(data, event) {
                var isValidColor = _color.simpleCheckForValidColor(data);
                if (isValidColor) {
                    var colors = _color.toState(data, data.h || _this.state.oldHue);
                    _this.props.onSwatchHover && _this.props.onSwatchHover(colors, event);
                }
            };
            _this.state = _extends({}, _color.toState(props.color, 0));
            _this.debounce = (0, _debounce.default)(function(fn, data, event) {
                fn(data, event);
            }, 100);
            return _this;
        }
        _createClass(ColorPicker, [
            {
                key: 'render',
                value: function render() {
                    var optionalEvents = {};
                    if (this.props.onSwatchHover) {
                        optionalEvents.onSwatchHover = this.handleSwatchHover;
                    }
                    return _react.default.createElement(Picker, _extends({}, this.props, this.state, {
                        onChange: this.handleChange
                    }, optionalEvents));
                }
            }
        ], [
            {
                key: 'getDerivedStateFromProps',
                value: function getDerivedStateFromProps(nextProps, state) {
                    return _extends({}, _color.toState(nextProps.color, state.oldHue));
                }
            }
        ]);
        return ColorPicker;
    }(_react.PureComponent || _react.Component);
    ColorPicker.propTypes = _extends({}, Picker.propTypes);
    ColorPicker.defaultProps = _extends({}, Picker.defaultProps, {
        color: {
            h: 250,
            s: 0.50,
            l: 0.20,
            a: 1
        }
    });
    return ColorPicker;
};
const _default = ColorWrap;

},
"83168a07": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Checkboard: function() {
        return Checkboard;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _checkboard = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("7d8013fd"));
var _extends = Object.assign || function(target) {
    for(var i = 1; i < arguments.length; i++){
        var source = arguments[i];
        for(var key in source){
            if (Object.prototype.hasOwnProperty.call(source, key)) {
                target[key] = source[key];
            }
        }
    }
    return target;
};
var Checkboard = function Checkboard(_ref) {
    var white = _ref.white, grey = _ref.grey, size = _ref.size, renderers = _ref.renderers, borderRadius = _ref.borderRadius, boxShadow = _ref.boxShadow, children = _ref.children;
    var styles = (0, _reactcss.default)({
        'default': {
            grid: {
                borderRadius: borderRadius,
                boxShadow: boxShadow,
                absolute: '0px 0px 0px 0px',
                background: 'url(' + _checkboard.get(white, grey, size, renderers.canvas) + ') center left'
            }
        }
    });
    return (0, _react.isValidElement)(children) ? _react.default.cloneElement(children, _extends({}, children.props, {
        style: _extends({}, children.props.style, styles.grid)
    })) : _react.default.createElement('div', {
        style: styles.grid
    });
};
Checkboard.defaultProps = {
    size: 8,
    white: 'transparent',
    grey: 'rgba(0,0,0,.08)',
    renderers: {}
};
const _default = Checkboard;

},
"8352887b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _assocIndexOf = /*#__PURE__*/ _interop_require_default._(farmRequire("9270f0e3"));
/**
 * Sets the list cache `key` to `value`.
 *
 * @private
 * @name set
 * @memberOf ListCache
 * @param {string} key The key of the value to set.
 * @param {*} value The value to set.
 * @returns {Object} Returns the list cache instance.
 */ function listCacheSet(key, value) {
    var data = this.__data__, index = (0, _assocIndexOf.default)(data, key);
    if (index < 0) {
        ++this.size;
        data.push([
            key,
            value
        ]);
    } else {
        data[index][1] = value;
    }
    return this;
}
const _default = listCacheSet;

},
"86d49f49": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * This method returns the first argument it receives.
 *
 * @static
 * @since 0.1.0
 * @memberOf _
 * @category Util
 * @param {*} value Any value.
 * @returns {*} Returns `value`.
 * @example
 *
 * var object = { 'a': 1 };
 *
 * console.log(_.identity(object) === object);
 * // => true
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function identity(value) {
    return value;
}
const _default = identity;

},
"889a06b2": function(module, exports, farmRequire, farmDynamicRequire) {
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
    getContrastingColor: function() {
        return getContrastingColor;
    },
    isValidHex: function() {
        return isValidHex;
    },
    isvalidColorString: function() {
        return isvalidColorString;
    },
    red: function() {
        return red;
    },
    simpleCheckForValidColor: function() {
        return simpleCheckForValidColor;
    },
    toState: function() {
        return toState;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _each = /*#__PURE__*/ _interop_require_default._(farmRequire("eb158937"));
const _tinycolor2 = /*#__PURE__*/ _interop_require_default._(farmRequire("5a9e6c9a"));
var simpleCheckForValidColor = function simpleCheckForValidColor(data) {
    var keysToCheck = [
        'r',
        'g',
        'b',
        'a',
        'h',
        's',
        'l',
        'v'
    ];
    var checked = 0;
    var passed = 0;
    (0, _each.default)(keysToCheck, function(letter) {
        if (data[letter]) {
            checked += 1;
            if (!isNaN(data[letter])) {
                passed += 1;
            }
            if (letter === 's' || letter === 'l') {
                var percentPatt = /^\d+%$/;
                if (percentPatt.test(data[letter])) {
                    passed += 1;
                }
            }
        }
    });
    return checked === passed ? data : false;
};
var toState = function toState(data, oldHue) {
    var color = data.hex ? (0, _tinycolor2.default)(data.hex) : (0, _tinycolor2.default)(data);
    var hsl = color.toHsl();
    var hsv = color.toHsv();
    var rgb = color.toRgb();
    var hex = color.toHex();
    if (hsl.s === 0) {
        hsl.h = oldHue || 0;
        hsv.h = oldHue || 0;
    }
    var transparent = hex === '000000' && rgb.a === 0;
    return {
        hsl: hsl,
        hex: transparent ? 'transparent' : '#' + hex,
        rgb: rgb,
        hsv: hsv,
        oldHue: data.h || oldHue || hsl.h,
        source: data.source
    };
};
var isValidHex = function isValidHex(hex) {
    if (hex === 'transparent') {
        return true;
    }
    // disable hex4 and hex8
    var lh = String(hex).charAt(0) === '#' ? 1 : 0;
    return hex.length !== 4 + lh && hex.length < 7 + lh && (0, _tinycolor2.default)(hex).isValid();
};
var getContrastingColor = function getContrastingColor(data) {
    if (!data) {
        return '#fff';
    }
    var col = toState(data);
    if (col.hex === 'transparent') {
        return 'rgba(0,0,0,0.4)';
    }
    var yiq = (col.rgb.r * 299 + col.rgb.g * 587 + col.rgb.b * 114) / 1000;
    return yiq >= 128 ? '#000' : '#fff';
};
var red = {
    hsl: {
        a: 1,
        h: 0,
        l: 0.5,
        s: 1
    },
    hex: '#ff0000',
    rgb: {
        r: 255,
        g: 0,
        b: 0,
        a: 1
    },
    hsv: {
        h: 0,
        s: 1,
        v: 1,
        a: 1
    }
};
var isvalidColorString = function isvalidColorString(string, type) {
    var stringWithoutDegree = string.replace('°', '');
    return (0, _tinycolor2.default)(type + ' (' + stringWithoutDegree + ')')._ok;
};

},
"8d305800": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return invariant;
    }
});
var isProduction = "production" === 'production';
var prefix = 'Invariant failed';
function invariant(condition, message) {
    if (condition) {
        return;
    }
    if (isProduction) {
        throw new Error(prefix);
    }
    var provided = typeof message === 'function' ? message() : message;
    var value = provided ? "".concat(prefix, ": ").concat(provided) : prefix;
    throw new Error(value);
}

},
"8e90d37b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _overArg = /*#__PURE__*/ _interop_require_default._(farmRequire("d7a3b053"));
/* Built-in method references for those with the same name as other `lodash` methods. */ var nativeKeys = (0, _overArg.default)(Object.keys, Object);
const _default = nativeKeys;

},
"8feb9431": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _root = /*#__PURE__*/ _interop_require_default._(farmRequire("6bf7b771"));
/** Detect free variable `exports`. */ var freeExports = typeof exports == 'object' && exports && !exports.nodeType && exports;
/** Detect free variable `module`. */ var freeModule = freeExports && typeof module == 'object' && module && !module.nodeType && module;
/** Detect the popular CommonJS extension `module.exports`. */ var moduleExports = freeModule && freeModule.exports === freeExports;
/** Built-in value references. */ var Buffer = moduleExports ? _root.default.Buffer : undefined, allocUnsafe = Buffer ? Buffer.allocUnsafe : undefined;
/**
 * Creates a clone of  `buffer`.
 *
 * @private
 * @param {Buffer} buffer The buffer to clone.
 * @param {boolean} [isDeep] Specify a deep clone.
 * @returns {Buffer} Returns the cloned buffer.
 */ function cloneBuffer(buffer, isDeep) {
    if (isDeep) {
        return buffer.slice();
    }
    var length = buffer.length, result = allocUnsafe ? allocUnsafe(length) : new buffer.constructor(length);
    buffer.copy(result);
    return result;
}
const _default = cloneBuffer;

},
"91fbac55": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _createBaseFor = /*#__PURE__*/ _interop_require_default._(farmRequire("121032f9"));
/**
 * The base implementation of `baseForOwn` which iterates over `object`
 * properties returned by `keysFunc` and invokes `iteratee` for each property.
 * Iteratee functions may exit iteration early by explicitly returning `false`.
 *
 * @private
 * @param {Object} object The object to iterate over.
 * @param {Function} iteratee The function invoked per iteration.
 * @param {Function} keysFunc The function to get the keys of `object`.
 * @returns {Object} Returns `object`.
 */ var baseFor = (0, _createBaseFor.default)();
const _default = baseFor;

},
"9270f0e3": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _eq = /*#__PURE__*/ _interop_require_default._(farmRequire("3e794592"));
/**
 * Gets the index at which the `key` is found in `array` of key-value pairs.
 *
 * @private
 * @param {Array} array The array to inspect.
 * @param {*} key The key to search for.
 * @returns {number} Returns the index of the matched value, else `-1`.
 */ function assocIndexOf(array, key) {
    var length = array.length;
    while(length--){
        if ((0, _eq.default)(array[length][0], key)) {
            return length;
        }
    }
    return -1;
}
const _default = assocIndexOf;

},
"9fa65b27": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _getMapData = /*#__PURE__*/ _interop_require_default._(farmRequire("d6ffea67"));
/**
 * Removes `key` and its value from the map.
 *
 * @private
 * @name delete
 * @memberOf MapCache
 * @param {string} key The key of the value to remove.
 * @returns {boolean} Returns `true` if the entry was removed, else `false`.
 */ function mapCacheDelete(key) {
    var result = (0, _getMapData.default)(this, key)['delete'](key);
    this.size -= result ? 1 : 0;
    return result;
}
const _default = mapCacheDelete;

},
"a259a82b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _arrayLikeKeys = /*#__PURE__*/ _interop_require_default._(farmRequire("fae4ae0b"));
const _baseKeysIn = /*#__PURE__*/ _interop_require_default._(farmRequire("ce35430e"));
const _isArrayLike = /*#__PURE__*/ _interop_require_default._(farmRequire("65cf6153"));
/**
 * Creates an array of the own and inherited enumerable property names of `object`.
 *
 * **Note:** Non-object values are coerced to objects.
 *
 * @static
 * @memberOf _
 * @since 3.0.0
 * @category Object
 * @param {Object} object The object to query.
 * @returns {Array} Returns the array of property names.
 * @example
 *
 * function Foo() {
 *   this.a = 1;
 *   this.b = 2;
 * }
 *
 * Foo.prototype.c = 3;
 *
 * _.keysIn(new Foo);
 * // => ['a', 'b', 'c'] (iteration order is not guaranteed)
 */ function keysIn(object) {
    return (0, _isArrayLike.default)(object) ? (0, _arrayLikeKeys.default)(object, true) : (0, _baseKeysIn.default)(object);
}
const _default = keysIn;

},
"a374a388": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _identity = /*#__PURE__*/ _interop_require_default._(farmRequire("86d49f49"));
/**
 * Casts `value` to `identity` if it's not a function.
 *
 * @private
 * @param {*} value The value to inspect.
 * @returns {Function} Returns cast function.
 */ function castFunction(value) {
    return typeof value == 'function' ? value : _identity.default;
}
const _default = castFunction;

},
"a55fc3a6": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _cloneArrayBuffer = /*#__PURE__*/ _interop_require_default._(farmRequire("77e9a7bd"));
/**
 * Creates a clone of `typedArray`.
 *
 * @private
 * @param {Object} typedArray The typed array to clone.
 * @param {boolean} [isDeep] Specify a deep clone.
 * @returns {Object} Returns the cloned typed array.
 */ function cloneTypedArray(typedArray, isDeep) {
    var buffer = isDeep ? (0, _cloneArrayBuffer.default)(typedArray.buffer) : typedArray.buffer;
    return new typedArray.constructor(buffer, typedArray.byteOffset, typedArray.length);
}
const _default = cloneTypedArray;

},
"a67f1a66": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Checks if `value` is the
 * [language type](http://www.ecma-international.org/ecma-262/7.0/#sec-ecmascript-language-types)
 * of `Object`. (e.g. arrays, functions, objects, regexes, `new Number(0)`, and `new String('')`)
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is an object, else `false`.
 * @example
 *
 * _.isObject({});
 * // => true
 *
 * _.isObject([1, 2, 3]);
 * // => true
 *
 * _.isObject(_.noop);
 * // => true
 *
 * _.isObject(null);
 * // => false
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function isObject(value) {
    var type = typeof value;
    return value != null && (type == 'object' || type == 'function');
}
const _default = isObject;

},
"a8b5d78d": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseIsTypedArray = /*#__PURE__*/ _interop_require_default._(farmRequire("e7191081"));
const _baseUnary = /*#__PURE__*/ _interop_require_default._(farmRequire("f213122b"));
const _nodeUtil = /*#__PURE__*/ _interop_require_default._(farmRequire("5a62e586"));
/* Node.js helper references. */ var nodeIsTypedArray = _nodeUtil.default && _nodeUtil.default.isTypedArray;
/**
 * Checks if `value` is classified as a typed array.
 *
 * @static
 * @memberOf _
 * @since 3.0.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a typed array, else `false`.
 * @example
 *
 * _.isTypedArray(new Uint8Array);
 * // => true
 *
 * _.isTypedArray([]);
 * // => false
 */ var isTypedArray = nodeIsTypedArray ? (0, _baseUnary.default)(nodeIsTypedArray) : _baseIsTypedArray.default;
const _default = isTypedArray;

},
"ab83cd8f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _copyObject = /*#__PURE__*/ _interop_require_default._(farmRequire("283458f2"));
const _keysIn = /*#__PURE__*/ _interop_require_default._(farmRequire("a259a82b"));
/**
 * Converts `value` to a plain object flattening inherited enumerable string
 * keyed properties of `value` to own properties of the plain object.
 *
 * @static
 * @memberOf _
 * @since 3.0.0
 * @category Lang
 * @param {*} value The value to convert.
 * @returns {Object} Returns the converted plain object.
 * @example
 *
 * function Foo() {
 *   this.b = 2;
 * }
 *
 * Foo.prototype.c = 3;
 *
 * _.assign({ 'a': 1 }, new Foo);
 * // => { 'a': 1, 'b': 2 }
 *
 * _.assign({ 'a': 1 }, _.toPlainObject(new Foo));
 * // => { 'a': 1, 'b': 2, 'c': 3 }
 */ function toPlainObject(value) {
    return (0, _copyObject.default)(value, (0, _keysIn.default)(value));
}
const _default = toPlainObject;

},
"b10d87d6": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isArrayLike = /*#__PURE__*/ _interop_require_default._(farmRequire("65cf6153"));
const _isObjectLike = /*#__PURE__*/ _interop_require_default._(farmRequire("1954b1ef"));
/**
 * This method is like `_.isArrayLike` except that it also checks if `value`
 * is an object.
 *
 * @static
 * @memberOf _
 * @since 4.0.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is an array-like object,
 *  else `false`.
 * @example
 *
 * _.isArrayLikeObject([1, 2, 3]);
 * // => true
 *
 * _.isArrayLikeObject(document.body.children);
 * // => true
 *
 * _.isArrayLikeObject('abc');
 * // => false
 *
 * _.isArrayLikeObject(_.noop);
 * // => false
 */ function isArrayLikeObject(value) {
    return (0, _isObjectLike.default)(value) && (0, _isArrayLike.default)(value);
}
const _default = isArrayLikeObject;

},
"b2779319": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Swatch: function() {
        return Swatch;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _interaction = farmRequire("b5a6d95f");
const _Checkboard = /*#__PURE__*/ _interop_require_default._(farmRequire("83168a07"));
var _extends = Object.assign || function(target) {
    for(var i = 1; i < arguments.length; i++){
        var source = arguments[i];
        for(var key in source){
            if (Object.prototype.hasOwnProperty.call(source, key)) {
                target[key] = source[key];
            }
        }
    }
    return target;
};
var ENTER = 13;
var Swatch = function Swatch(_ref) {
    var color = _ref.color, style = _ref.style, _ref$onClick = _ref.onClick, onClick = _ref$onClick === undefined ? function() {} : _ref$onClick, onHover = _ref.onHover, _ref$title = _ref.title, title = _ref$title === undefined ? color : _ref$title, children = _ref.children, focus = _ref.focus, _ref$focusStyle = _ref.focusStyle, focusStyle = _ref$focusStyle === undefined ? {} : _ref$focusStyle;
    var transparent = color === 'transparent';
    var styles = (0, _reactcss.default)({
        default: {
            swatch: _extends({
                background: color,
                height: '100%',
                width: '100%',
                cursor: 'pointer',
                position: 'relative',
                outline: 'none'
            }, style, focus ? focusStyle : {})
        }
    });
    var handleClick = function handleClick(e) {
        return onClick(color, e);
    };
    var handleKeyDown = function handleKeyDown(e) {
        return e.keyCode === ENTER && onClick(color, e);
    };
    var handleHover = function handleHover(e) {
        return onHover(color, e);
    };
    var optionalEvents = {};
    if (onHover) {
        optionalEvents.onMouseOver = handleHover;
    }
    return _react.default.createElement('div', _extends({
        style: styles.swatch,
        onClick: handleClick,
        title: title,
        tabIndex: 0,
        onKeyDown: handleKeyDown
    }, optionalEvents), children, transparent && _react.default.createElement(_Checkboard.default, {
        borderRadius: styles.swatch.borderRadius,
        boxShadow: 'inset 0 0 0 1px rgba(0,0,0,0.1)'
    }));
};
const _default = (0, _interaction.handleFocus)(Swatch);

},
"b3b7aa28": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "calculateChange", {
    enumerable: true,
    get: function() {
        return calculateChange;
    }
});
var calculateChange = function calculateChange(e, hsl, direction, initialA, container) {
    var containerWidth = container.clientWidth;
    var containerHeight = container.clientHeight;
    var x = typeof e.pageX === 'number' ? e.pageX : e.touches[0].pageX;
    var y = typeof e.pageY === 'number' ? e.pageY : e.touches[0].pageY;
    var left = x - (container.getBoundingClientRect().left + window.pageXOffset);
    var top = y - (container.getBoundingClientRect().top + window.pageYOffset);
    if (direction === 'vertical') {
        var a = void 0;
        if (top < 0) {
            a = 0;
        } else if (top > containerHeight) {
            a = 1;
        } else {
            a = Math.round(top * 100 / containerHeight) / 100;
        }
        if (hsl.a !== a) {
            return {
                h: hsl.h,
                s: hsl.s,
                l: hsl.l,
                a: a,
                source: 'rgb'
            };
        }
    } else {
        var _a = void 0;
        if (left < 0) {
            _a = 0;
        } else if (left > containerWidth) {
            _a = 1;
        } else {
            _a = Math.round(left * 100 / containerWidth) / 100;
        }
        if (initialA !== _a) {
            return {
                h: hsl.h,
                s: hsl.s,
                l: hsl.l,
                a: _a,
                source: 'rgb'
            };
        }
    }
    return null;
};

},
"b437633f": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "calculateChange", {
    enumerable: true,
    get: function() {
        return calculateChange;
    }
});
var calculateChange = function calculateChange(e, hsl, container) {
    var _container$getBoundin = container.getBoundingClientRect(), containerWidth = _container$getBoundin.width, containerHeight = _container$getBoundin.height;
    var x = typeof e.pageX === 'number' ? e.pageX : e.touches[0].pageX;
    var y = typeof e.pageY === 'number' ? e.pageY : e.touches[0].pageY;
    var left = x - (container.getBoundingClientRect().left + window.pageXOffset);
    var top = y - (container.getBoundingClientRect().top + window.pageYOffset);
    if (left < 0) {
        left = 0;
    } else if (left > containerWidth) {
        left = containerWidth;
    }
    if (top < 0) {
        top = 0;
    } else if (top > containerHeight) {
        top = containerHeight;
    }
    var saturation = left / containerWidth;
    var bright = 1 - top / containerHeight;
    return {
        h: hsl.h,
        s: saturation,
        v: bright,
        a: hsl.a,
        source: 'hsv'
    };
};

},
"b4de9912": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseForOwn = /*#__PURE__*/ _interop_require_default._(farmRequire("66b9aa18"));
const _createBaseEach = /*#__PURE__*/ _interop_require_default._(farmRequire("6287afc4"));
/**
 * The base implementation of `_.forEach` without support for iteratee shorthands.
 *
 * @private
 * @param {Array|Object} collection The collection to iterate over.
 * @param {Function} iteratee The function invoked per iteration.
 * @returns {Array|Object} Returns `collection`.
 */ var baseEach = (0, _createBaseEach.default)(_baseForOwn.default);
const _default = baseEach;

},
"b5a6d95f": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "handleFocus", {
    enumerable: true,
    get: function() {
        return handleFocus;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
var _extends = Object.assign || function(target) {
    for(var i = 1; i < arguments.length; i++){
        var source = arguments[i];
        for(var key in source){
            if (Object.prototype.hasOwnProperty.call(source, key)) {
                target[key] = source[key];
            }
        }
    }
    return target;
};
var _createClass = function() {
    function defineProperties(target, props) {
        for(var i = 0; i < props.length; i++){
            var descriptor = props[i];
            descriptor.enumerable = descriptor.enumerable || false;
            descriptor.configurable = true;
            if ("value" in descriptor) descriptor.writable = true;
            Object.defineProperty(target, descriptor.key, descriptor);
        }
    }
    return function(Constructor, protoProps, staticProps) {
        if (protoProps) defineProperties(Constructor.prototype, protoProps);
        if (staticProps) defineProperties(Constructor, staticProps);
        return Constructor;
    };
}();
function _classCallCheck(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
function _possibleConstructorReturn(self, call) {
    if (!self) {
        throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
    }
    return call && (typeof call === "object" || typeof call === "function") ? call : self;
}
function _inherits(subClass, superClass) {
    if (typeof superClass !== "function" && superClass !== null) {
        throw new TypeError("Super expression must either be null or a function, not " + typeof superClass);
    }
    subClass.prototype = Object.create(superClass && superClass.prototype, {
        constructor: {
            value: subClass,
            enumerable: false,
            writable: true,
            configurable: true
        }
    });
    if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass;
}
var handleFocus = function handleFocus(Component) {
    var Span = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : 'span';
    return function(_React$Component) {
        _inherits(Focus, _React$Component);
        function Focus() {
            var _ref;
            var _temp, _this, _ret;
            _classCallCheck(this, Focus);
            for(var _len = arguments.length, args = Array(_len), _key = 0; _key < _len; _key++){
                args[_key] = arguments[_key];
            }
            return _ret = (_temp = (_this = _possibleConstructorReturn(this, (_ref = Focus.__proto__ || Object.getPrototypeOf(Focus)).call.apply(_ref, [
                this
            ].concat(args))), _this), _this.state = {
                focus: false
            }, _this.handleFocus = function() {
                return _this.setState({
                    focus: true
                });
            }, _this.handleBlur = function() {
                return _this.setState({
                    focus: false
                });
            }, _temp), _possibleConstructorReturn(_this, _ret);
        }
        _createClass(Focus, [
            {
                key: 'render',
                value: function render() {
                    return _react.default.createElement(Span, {
                        onFocus: this.handleFocus,
                        onBlur: this.handleBlur
                    }, _react.default.createElement(Component, _extends({}, this.props, this.state)));
                }
            }
        ]);
        return Focus;
    }(_react.default.Component);
};

},
"b93a8038": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseCreate = /*#__PURE__*/ _interop_require_default._(farmRequire("2b1f08c0"));
const _getPrototype = /*#__PURE__*/ _interop_require_default._(farmRequire("e251ea03"));
const _isPrototype = /*#__PURE__*/ _interop_require_default._(farmRequire("ce92b344"));
/**
 * Initializes an object clone.
 *
 * @private
 * @param {Object} object The object to clone.
 * @returns {Object} Returns the initialized clone.
 */ function initCloneObject(object) {
    return typeof object.constructor == 'function' && !(0, _isPrototype.default)(object) ? (0, _baseCreate.default)((0, _getPrototype.default)(object)) : {};
}
const _default = initCloneObject;

},
"b995b8c7": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _getMapData = /*#__PURE__*/ _interop_require_default._(farmRequire("d6ffea67"));
/**
 * Sets the map `key` to `value`.
 *
 * @private
 * @name set
 * @memberOf MapCache
 * @param {string} key The key of the value to set.
 * @param {*} value The value to set.
 * @returns {Object} Returns the map cache instance.
 */ function mapCacheSet(key, value) {
    var data = (0, _getMapData.default)(this, key), size = data.size;
    data.set(key, value);
    this.size += data.size == size ? 0 : 1;
    return this;
}
const _default = mapCacheSet;

},
"bda4f991": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
const _now = /*#__PURE__*/ _interop_require_default._(farmRequire("f37b1f98"));
const _toNumber = /*#__PURE__*/ _interop_require_default._(farmRequire("6c30181d"));
/** Error message constants. */ var FUNC_ERROR_TEXT = 'Expected a function';
/* Built-in method references for those with the same name as other `lodash` methods. */ var nativeMax = Math.max, nativeMin = Math.min;
/**
 * Creates a debounced function that delays invoking `func` until after `wait`
 * milliseconds have elapsed since the last time the debounced function was
 * invoked. The debounced function comes with a `cancel` method to cancel
 * delayed `func` invocations and a `flush` method to immediately invoke them.
 * Provide `options` to indicate whether `func` should be invoked on the
 * leading and/or trailing edge of the `wait` timeout. The `func` is invoked
 * with the last arguments provided to the debounced function. Subsequent
 * calls to the debounced function return the result of the last `func`
 * invocation.
 *
 * **Note:** If `leading` and `trailing` options are `true`, `func` is
 * invoked on the trailing edge of the timeout only if the debounced function
 * is invoked more than once during the `wait` timeout.
 *
 * If `wait` is `0` and `leading` is `false`, `func` invocation is deferred
 * until to the next tick, similar to `setTimeout` with a timeout of `0`.
 *
 * See [David Corbacho's article](https://css-tricks.com/debouncing-throttling-explained-examples/)
 * for details over the differences between `_.debounce` and `_.throttle`.
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @category Function
 * @param {Function} func The function to debounce.
 * @param {number} [wait=0] The number of milliseconds to delay.
 * @param {Object} [options={}] The options object.
 * @param {boolean} [options.leading=false]
 *  Specify invoking on the leading edge of the timeout.
 * @param {number} [options.maxWait]
 *  The maximum time `func` is allowed to be delayed before it's invoked.
 * @param {boolean} [options.trailing=true]
 *  Specify invoking on the trailing edge of the timeout.
 * @returns {Function} Returns the new debounced function.
 * @example
 *
 * // Avoid costly calculations while the window size is in flux.
 * jQuery(window).on('resize', _.debounce(calculateLayout, 150));
 *
 * // Invoke `sendMail` when clicked, debouncing subsequent calls.
 * jQuery(element).on('click', _.debounce(sendMail, 300, {
 *   'leading': true,
 *   'trailing': false
 * }));
 *
 * // Ensure `batchLog` is invoked once after 1 second of debounced calls.
 * var debounced = _.debounce(batchLog, 250, { 'maxWait': 1000 });
 * var source = new EventSource('/stream');
 * jQuery(source).on('message', debounced);
 *
 * // Cancel the trailing debounced invocation.
 * jQuery(window).on('popstate', debounced.cancel);
 */ function debounce(func, wait, options) {
    var lastArgs, lastThis, maxWait, result, timerId, lastCallTime, lastInvokeTime = 0, leading = false, maxing = false, trailing = true;
    if (typeof func != 'function') {
        throw new TypeError(FUNC_ERROR_TEXT);
    }
    wait = (0, _toNumber.default)(wait) || 0;
    if ((0, _isObject.default)(options)) {
        leading = !!options.leading;
        maxing = 'maxWait' in options;
        maxWait = maxing ? nativeMax((0, _toNumber.default)(options.maxWait) || 0, wait) : maxWait;
        trailing = 'trailing' in options ? !!options.trailing : trailing;
    }
    function invokeFunc(time) {
        var args = lastArgs, thisArg = lastThis;
        lastArgs = lastThis = undefined;
        lastInvokeTime = time;
        result = func.apply(thisArg, args);
        return result;
    }
    function leadingEdge(time) {
        // Reset any `maxWait` timer.
        lastInvokeTime = time;
        // Start the timer for the trailing edge.
        timerId = setTimeout(timerExpired, wait);
        // Invoke the leading edge.
        return leading ? invokeFunc(time) : result;
    }
    function remainingWait(time) {
        var timeSinceLastCall = time - lastCallTime, timeSinceLastInvoke = time - lastInvokeTime, timeWaiting = wait - timeSinceLastCall;
        return maxing ? nativeMin(timeWaiting, maxWait - timeSinceLastInvoke) : timeWaiting;
    }
    function shouldInvoke(time) {
        var timeSinceLastCall = time - lastCallTime, timeSinceLastInvoke = time - lastInvokeTime;
        // Either this is the first call, activity has stopped and we're at the
        // trailing edge, the system time has gone backwards and we're treating
        // it as the trailing edge, or we've hit the `maxWait` limit.
        return lastCallTime === undefined || timeSinceLastCall >= wait || timeSinceLastCall < 0 || maxing && timeSinceLastInvoke >= maxWait;
    }
    function timerExpired() {
        var time = (0, _now.default)();
        if (shouldInvoke(time)) {
            return trailingEdge(time);
        }
        // Restart the timer.
        timerId = setTimeout(timerExpired, remainingWait(time));
    }
    function trailingEdge(time) {
        timerId = undefined;
        // Only invoke if we have `lastArgs` which means `func` has been
        // debounced at least once.
        if (trailing && lastArgs) {
            return invokeFunc(time);
        }
        lastArgs = lastThis = undefined;
        return result;
    }
    function cancel() {
        if (timerId !== undefined) {
            clearTimeout(timerId);
        }
        lastInvokeTime = 0;
        lastArgs = lastCallTime = lastThis = timerId = undefined;
    }
    function flush() {
        return timerId === undefined ? result : trailingEdge((0, _now.default)());
    }
    function debounced() {
        var time = (0, _now.default)(), isInvoking = shouldInvoke(time);
        lastArgs = arguments;
        lastThis = this;
        lastCallTime = time;
        if (isInvoking) {
            if (timerId === undefined) {
                return leadingEdge(lastCallTime);
            }
            if (maxing) {
                // Handle invocations in a tight loop.
                clearTimeout(timerId);
                timerId = setTimeout(timerExpired, wait);
                return invokeFunc(lastCallTime);
            }
        }
        if (timerId === undefined) {
            timerId = setTimeout(timerExpired, wait);
        }
        return result;
    }
    debounced.cancel = cancel;
    debounced.flush = flush;
    return debounced;
}
const _default = debounce;

},
"be2a156f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _identity = /*#__PURE__*/ _interop_require_default._(farmRequire("86d49f49"));
const _overRest = /*#__PURE__*/ _interop_require_default._(farmRequire("33c4445f"));
const _setToString = /*#__PURE__*/ _interop_require_default._(farmRequire("c388e270"));
/**
 * The base implementation of `_.rest` which doesn't validate or coerce arguments.
 *
 * @private
 * @param {Function} func The function to apply a rest parameter to.
 * @param {number} [start=func.length-1] The start position of the rest parameter.
 * @returns {Function} Returns the new function.
 */ function baseRest(func, start) {
    return (0, _setToString.default)((0, _overRest.default)(func, start, _identity.default), func + '');
}
const _default = baseRest;

},
"c14d35a7": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * This method returns `false`.
 *
 * @static
 * @memberOf _
 * @since 4.13.0
 * @category Util
 * @returns {boolean} Returns `false`.
 * @example
 *
 * _.times(2, _.stubFalse);
 * // => [false, false]
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function stubFalse() {
    return false;
}
const _default = stubFalse;

},
"c388e270": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseSetToString = /*#__PURE__*/ _interop_require_default._(farmRequire("811d4135"));
const _shortOut = /*#__PURE__*/ _interop_require_default._(farmRequire("14ae416a"));
/**
 * Sets the `toString` method of `func` to return `string`.
 *
 * @private
 * @param {Function} func The function to modify.
 * @param {Function} string The `toString` result.
 * @returns {Function} Returns `func`.
 */ var setToString = (0, _shortOut.default)(_baseSetToString.default);
const _default = setToString;

},
"c5dbc97f": function(module, exports, farmRequire, farmDynamicRequire) {
/** Detect free variable `global` from Node.js. */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var freeGlobal = typeof global == 'object' && global && global.Object === Object && global;
const _default = freeGlobal;

},
"c8075e33": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _assocIndexOf = /*#__PURE__*/ _interop_require_default._(farmRequire("9270f0e3"));
/** Used for built-in method references. */ var arrayProto = Array.prototype;
/** Built-in value references. */ var splice = arrayProto.splice;
/**
 * Removes `key` and its value from the list cache.
 *
 * @private
 * @name delete
 * @memberOf ListCache
 * @param {string} key The key of the value to remove.
 * @returns {boolean} Returns `true` if the entry was removed, else `false`.
 */ function listCacheDelete(key) {
    var data = this.__data__, index = (0, _assocIndexOf.default)(data, key);
    if (index < 0) {
        return false;
    }
    var lastIndex = data.length - 1;
    if (index == lastIndex) {
        data.pop();
    } else {
        splice.call(data, index, 1);
    }
    --this.size;
    return true;
}
const _default = listCacheDelete;

},
"cbe2a7de": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * A faster alternative to `Function#apply`, this function invokes `func`
 * with the `this` binding of `thisArg` and the arguments of `args`.
 *
 * @private
 * @param {Function} func The function to invoke.
 * @param {*} thisArg The `this` binding of `func`.
 * @param {Array} args The arguments to invoke `func` with.
 * @returns {*} Returns the result of `func`.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function apply(func, thisArg, args) {
    switch(args.length){
        case 0:
            return func.call(thisArg);
        case 1:
            return func.call(thisArg, args[0]);
        case 2:
            return func.call(thisArg, args[0], args[1]);
        case 3:
            return func.call(thisArg, args[0], args[1], args[2]);
    }
    return func.apply(thisArg, args);
}
const _default = apply;

},
"cdbeebb3": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _root = /*#__PURE__*/ _interop_require_default._(farmRequire("6bf7b771"));
/** Built-in value references. */ var Uint8Array = _root.default.Uint8Array;
const _default = Uint8Array;

},
"ce35430e": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
const _isPrototype = /*#__PURE__*/ _interop_require_default._(farmRequire("ce92b344"));
const _nativeKeysIn = /*#__PURE__*/ _interop_require_default._(farmRequire("01857a72"));
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/**
 * The base implementation of `_.keysIn` which doesn't treat sparse arrays as dense.
 *
 * @private
 * @param {Object} object The object to query.
 * @returns {Array} Returns the array of property names.
 */ function baseKeysIn(object) {
    if (!(0, _isObject.default)(object)) {
        return (0, _nativeKeysIn.default)(object);
    }
    var isProto = (0, _isPrototype.default)(object), result = [];
    for(var key in object){
        if (!(key == 'constructor' && (isProto || !hasOwnProperty.call(object, key)))) {
            result.push(key);
        }
    }
    return result;
}
const _default = baseKeysIn;

},
"ce92b344": function(module, exports, farmRequire, farmDynamicRequire) {
/** Used for built-in method references. */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var objectProto = Object.prototype;
/**
 * Checks if `value` is likely a prototype object.
 *
 * @private
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a prototype, else `false`.
 */ function isPrototype(value) {
    var Ctor = value && value.constructor, proto = typeof Ctor == 'function' && Ctor.prototype || objectProto;
    return value === proto;
}
const _default = isPrototype;

},
"d01e25ba": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _ListCache = /*#__PURE__*/ _interop_require_default._(farmRequire("47a014c7"));
/**
 * Removes all key-value entries from the stack.
 *
 * @private
 * @name clear
 * @memberOf Stack
 */ function stackClear() {
    this.__data__ = new _ListCache.default;
    this.size = 0;
}
const _default = stackClear;

},
"d0910965": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _arrayEach = /*#__PURE__*/ _interop_require_default._(farmRequire("06f8323b"));
const _baseEach = /*#__PURE__*/ _interop_require_default._(farmRequire("b4de9912"));
const _castFunction = /*#__PURE__*/ _interop_require_default._(farmRequire("a374a388"));
const _isArray = /*#__PURE__*/ _interop_require_default._(farmRequire("f1ceb9be"));
/**
 * Iterates over elements of `collection` and invokes `iteratee` for each element.
 * The iteratee is invoked with three arguments: (value, index|key, collection).
 * Iteratee functions may exit iteration early by explicitly returning `false`.
 *
 * **Note:** As with other "Collections" methods, objects with a "length"
 * property are iterated like arrays. To avoid this behavior use `_.forIn`
 * or `_.forOwn` for object iteration.
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @alias each
 * @category Collection
 * @param {Array|Object} collection The collection to iterate over.
 * @param {Function} [iteratee=_.identity] The function invoked per iteration.
 * @returns {Array|Object} Returns `collection`.
 * @see _.forEachRight
 * @example
 *
 * _.forEach([1, 2], function(value) {
 *   console.log(value);
 * });
 * // => Logs `1` then `2`.
 *
 * _.forEach({ 'a': 1, 'b': 2 }, function(value, key) {
 *   console.log(key);
 * });
 * // => Logs 'a' then 'b' (iteration order is not guaranteed).
 */ function forEach(collection, iteratee) {
    var func = (0, _isArray.default)(collection) ? _arrayEach.default : _baseEach.default;
    return func(collection, (0, _castFunction.default)(iteratee));
}
const _default = forEach;

},
"d35f90df": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Hue: function() {
        return Hue;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _hue = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("70a9c618"));
var _createClass = function() {
    function defineProperties(target, props) {
        for(var i = 0; i < props.length; i++){
            var descriptor = props[i];
            descriptor.enumerable = descriptor.enumerable || false;
            descriptor.configurable = true;
            if ("value" in descriptor) descriptor.writable = true;
            Object.defineProperty(target, descriptor.key, descriptor);
        }
    }
    return function(Constructor, protoProps, staticProps) {
        if (protoProps) defineProperties(Constructor.prototype, protoProps);
        if (staticProps) defineProperties(Constructor, staticProps);
        return Constructor;
    };
}();
function _classCallCheck(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
function _possibleConstructorReturn(self, call) {
    if (!self) {
        throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
    }
    return call && (typeof call === "object" || typeof call === "function") ? call : self;
}
function _inherits(subClass, superClass) {
    if (typeof superClass !== "function" && superClass !== null) {
        throw new TypeError("Super expression must either be null or a function, not " + typeof superClass);
    }
    subClass.prototype = Object.create(superClass && superClass.prototype, {
        constructor: {
            value: subClass,
            enumerable: false,
            writable: true,
            configurable: true
        }
    });
    if (superClass) Object.setPrototypeOf ? Object.setPrototypeOf(subClass, superClass) : subClass.__proto__ = superClass;
}
var Hue = function(_ref) {
    _inherits(Hue, _ref);
    function Hue() {
        var _ref2;
        var _temp, _this, _ret;
        _classCallCheck(this, Hue);
        for(var _len = arguments.length, args = Array(_len), _key = 0; _key < _len; _key++){
            args[_key] = arguments[_key];
        }
        return _ret = (_temp = (_this = _possibleConstructorReturn(this, (_ref2 = Hue.__proto__ || Object.getPrototypeOf(Hue)).call.apply(_ref2, [
            this
        ].concat(args))), _this), _this.handleChange = function(e) {
            var change = _hue.calculateChange(e, _this.props.direction, _this.props.hsl, _this.container);
            change && typeof _this.props.onChange === 'function' && _this.props.onChange(change, e);
        }, _this.handleMouseDown = function(e) {
            _this.handleChange(e);
            window.addEventListener('mousemove', _this.handleChange);
            window.addEventListener('mouseup', _this.handleMouseUp);
        }, _this.handleMouseUp = function() {
            _this.unbindEventListeners();
        }, _temp), _possibleConstructorReturn(_this, _ret);
    }
    _createClass(Hue, [
        {
            key: 'componentWillUnmount',
            value: function componentWillUnmount() {
                this.unbindEventListeners();
            }
        },
        {
            key: 'unbindEventListeners',
            value: function unbindEventListeners() {
                window.removeEventListener('mousemove', this.handleChange);
                window.removeEventListener('mouseup', this.handleMouseUp);
            }
        },
        {
            key: 'render',
            value: function render() {
                var _this2 = this;
                var _props$direction = this.props.direction, direction = _props$direction === undefined ? 'horizontal' : _props$direction;
                var styles = (0, _reactcss.default)({
                    'default': {
                        hue: {
                            absolute: '0px 0px 0px 0px',
                            borderRadius: this.props.radius,
                            boxShadow: this.props.shadow
                        },
                        container: {
                            padding: '0 2px',
                            position: 'relative',
                            height: '100%',
                            borderRadius: this.props.radius
                        },
                        pointer: {
                            position: 'absolute',
                            left: this.props.hsl.h * 100 / 360 + '%'
                        },
                        slider: {
                            marginTop: '1px',
                            width: '4px',
                            borderRadius: '1px',
                            height: '8px',
                            boxShadow: '0 0 2px rgba(0, 0, 0, .6)',
                            background: '#fff',
                            transform: 'translateX(-2px)'
                        }
                    },
                    'vertical': {
                        pointer: {
                            left: '0px',
                            top: -(this.props.hsl.h * 100 / 360) + 100 + '%'
                        }
                    }
                }, {
                    vertical: direction === 'vertical'
                });
                return _react.default.createElement('div', {
                    style: styles.hue
                }, _react.default.createElement('div', {
                    className: 'hue-' + direction,
                    style: styles.container,
                    ref: function ref(container) {
                        return _this2.container = container;
                    },
                    onMouseDown: this.handleMouseDown,
                    onTouchMove: this.handleChange,
                    onTouchStart: this.handleChange
                }, _react.default.createElement('style', null, '\n            .hue-horizontal {\n              background: linear-gradient(to right, #f00 0%, #ff0 17%, #0f0\n                33%, #0ff 50%, #00f 67%, #f0f 83%, #f00 100%);\n              background: -webkit-linear-gradient(to right, #f00 0%, #ff0\n                17%, #0f0 33%, #0ff 50%, #00f 67%, #f0f 83%, #f00 100%);\n            }\n\n            .hue-vertical {\n              background: linear-gradient(to top, #f00 0%, #ff0 17%, #0f0 33%,\n                #0ff 50%, #00f 67%, #f0f 83%, #f00 100%);\n              background: -webkit-linear-gradient(to top, #f00 0%, #ff0 17%,\n                #0f0 33%, #0ff 50%, #00f 67%, #f0f 83%, #f00 100%);\n            }\n          '), _react.default.createElement('div', {
                    style: styles.pointer
                }, this.props.pointer ? _react.default.createElement(this.props.pointer, this.props) : _react.default.createElement('div', {
                    style: styles.slider
                }))));
            }
        }
    ]);
    return Hue;
}(_react.PureComponent || _react.Component);
const _default = Hue;

},
"d6ffea67": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _isKeyable = /*#__PURE__*/ _interop_require_default._(farmRequire("f31c2091"));
/**
 * Gets the data for `map`.
 *
 * @private
 * @param {Object} map The map to query.
 * @param {string} key The reference key.
 * @returns {*} Returns the map data.
 */ function getMapData(map, key) {
    var data = map.__data__;
    return (0, _isKeyable.default)(key) ? data[typeof key == 'string' ? 'string' : 'hash'] : data.map;
}
const _default = getMapData;

},
"d7a3b053": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Creates a unary function that invokes `func` with its argument transformed.
 *
 * @private
 * @param {Function} func The function to wrap.
 * @param {Function} transform The argument transform.
 * @returns {Function} Returns the new function.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function overArg(func, transform) {
    return function(arg) {
        return func(transform(arg));
    };
}
const _default = overArg;

},
"d932635f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _trimmedEndIndex = /*#__PURE__*/ _interop_require_default._(farmRequire("41498ac6"));
/** Used to match leading whitespace. */ var reTrimStart = /^\s+/;
/**
 * The base implementation of `_.trim`.
 *
 * @private
 * @param {string} string The string to trim.
 * @returns {string} Returns the trimmed string.
 */ function baseTrim(string) {
    return string ? string.slice(0, (0, _trimmedEndIndex.default)(string) + 1).replace(reTrimStart, '') : string;
}
const _default = baseTrim;

},
"da2fd2b6": function(module, exports, farmRequire, farmDynamicRequire) {
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
    ChromePointer: function() {
        return ChromePointer;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
var ChromePointer = function ChromePointer() {
    var styles = (0, _reactcss.default)({
        'default': {
            picker: {
                width: '12px',
                height: '12px',
                borderRadius: '6px',
                transform: 'translate(-6px, -1px)',
                backgroundColor: 'rgb(248, 248, 248)',
                boxShadow: '0 1px 4px 0 rgba(0, 0, 0, 0.37)'
            }
        }
    });
    return _react.default.createElement('div', {
        style: styles.picker
    });
};
const _default = ChromePointer;

},
"dc218a61": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _getNative = /*#__PURE__*/ _interop_require_default._(farmRequire("7d9eff84"));
/* Built-in method references that are verified to be native. */ var nativeCreate = (0, _getNative.default)(Object, 'create');
const _default = nativeCreate;

},
"df3b0a5b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _root = /*#__PURE__*/ _interop_require_default._(farmRequire("6bf7b771"));
/** Built-in value references. */ var Symbol = _root.default.Symbol;
const _default = Symbol;

},
"e09a6b9e": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _hashClear = /*#__PURE__*/ _interop_require_default._(farmRequire("35076a58"));
const _hashDelete = /*#__PURE__*/ _interop_require_default._(farmRequire("f27471f3"));
const _hashGet = /*#__PURE__*/ _interop_require_default._(farmRequire("70020fba"));
const _hashHas = /*#__PURE__*/ _interop_require_default._(farmRequire("f29f63ea"));
const _hashSet = /*#__PURE__*/ _interop_require_default._(farmRequire("6d294af6"));
/**
 * Creates a hash object.
 *
 * @private
 * @constructor
 * @param {Array} [entries] The key-value pairs to cache.
 */ function Hash(entries) {
    var index = -1, length = entries == null ? 0 : entries.length;
    this.clear();
    while(++index < length){
        var entry = entries[index];
        this.set(entry[0], entry[1]);
    }
}
// Add methods to `Hash`.
Hash.prototype.clear = _hashClear.default;
Hash.prototype['delete'] = _hashDelete.default;
Hash.prototype.get = _hashGet.default;
Hash.prototype.has = _hashHas.default;
Hash.prototype.set = _hashSet.default;
const _default = Hash;

},
"e251ea03": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _overArg = /*#__PURE__*/ _interop_require_default._(farmRequire("d7a3b053"));
/** Built-in value references. */ var getPrototype = (0, _overArg.default)(Object.getPrototypeOf, Object);
const _default = getPrototype;

},
"e44b5e2d": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Removes `key` and its value from the stack.
 *
 * @private
 * @name delete
 * @memberOf Stack
 * @param {string} key The key of the value to remove.
 * @returns {boolean} Returns `true` if the entry was removed, else `false`.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function stackDelete(key) {
    var data = this.__data__, result = data['delete'](key);
    this.size = data.size;
    return result;
}
const _default = stackDelete;

},
"e46625fb": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseRest = /*#__PURE__*/ _interop_require_default._(farmRequire("be2a156f"));
const _isIterateeCall = /*#__PURE__*/ _interop_require_default._(farmRequire("577cafe7"));
/**
 * Creates a function like `_.assign`.
 *
 * @private
 * @param {Function} assigner The function to assign values.
 * @returns {Function} Returns the new assigner function.
 */ function createAssigner(assigner) {
    return (0, _baseRest.default)(function(object, sources) {
        var index = -1, length = sources.length, customizer = length > 1 ? sources[length - 1] : undefined, guard = length > 2 ? sources[2] : undefined;
        customizer = assigner.length > 3 && typeof customizer == 'function' ? (length--, customizer) : undefined;
        if (guard && (0, _isIterateeCall.default)(sources[0], sources[1], guard)) {
            customizer = length < 3 ? undefined : customizer;
            length = 1;
        }
        object = Object(object);
        while(++index < length){
            var source = sources[index];
            if (source) {
                assigner(object, source, index, customizer);
            }
        }
        return object;
    });
}
const _default = createAssigner;

},
"e4aeffaa": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _Stack = /*#__PURE__*/ _interop_require_default._(farmRequire("f7a29359"));
const _assignMergeValue = /*#__PURE__*/ _interop_require_default._(farmRequire("ec3f319a"));
const _baseFor = /*#__PURE__*/ _interop_require_default._(farmRequire("91fbac55"));
const _baseMergeDeep = /*#__PURE__*/ _interop_require_default._(farmRequire("19eb8ac9"));
const _isObject = /*#__PURE__*/ _interop_require_default._(farmRequire("a67f1a66"));
const _keysIn = /*#__PURE__*/ _interop_require_default._(farmRequire("a259a82b"));
const _safeGet = /*#__PURE__*/ _interop_require_default._(farmRequire("5057e1a2"));
/**
 * The base implementation of `_.merge` without support for multiple sources.
 *
 * @private
 * @param {Object} object The destination object.
 * @param {Object} source The source object.
 * @param {number} srcIndex The index of `source`.
 * @param {Function} [customizer] The function to customize merged values.
 * @param {Object} [stack] Tracks traversed source values and their merged
 *  counterparts.
 */ function baseMerge(object, source, srcIndex, customizer, stack) {
    if (object === source) {
        return;
    }
    (0, _baseFor.default)(source, function(srcValue, key) {
        stack || (stack = new _Stack.default);
        if ((0, _isObject.default)(srcValue)) {
            (0, _baseMergeDeep.default)(object, source, key, srcIndex, baseMerge, customizer, stack);
        } else {
            var newValue = customizer ? customizer((0, _safeGet.default)(object, key), srcValue, key + '', object, source, stack) : undefined;
            if (newValue === undefined) {
                newValue = srcValue;
            }
            (0, _assignMergeValue.default)(object, key, newValue);
        }
    }, _keysIn.default);
}
const _default = baseMerge;

},
"e53a6514": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _root = /*#__PURE__*/ _interop_require_default._(farmRequire("6bf7b771"));
/** Used to detect overreaching core-js shims. */ var coreJsData = _root.default['__core-js_shared__'];
const _default = coreJsData;

},
"e7191081": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseGetTag = /*#__PURE__*/ _interop_require_default._(farmRequire("4ffcc116"));
const _isLength = /*#__PURE__*/ _interop_require_default._(farmRequire("2031bec1"));
const _isObjectLike = /*#__PURE__*/ _interop_require_default._(farmRequire("1954b1ef"));
/** `Object#toString` result references. */ var argsTag = '[object Arguments]', arrayTag = '[object Array]', boolTag = '[object Boolean]', dateTag = '[object Date]', errorTag = '[object Error]', funcTag = '[object Function]', mapTag = '[object Map]', numberTag = '[object Number]', objectTag = '[object Object]', regexpTag = '[object RegExp]', setTag = '[object Set]', stringTag = '[object String]', weakMapTag = '[object WeakMap]';
var arrayBufferTag = '[object ArrayBuffer]', dataViewTag = '[object DataView]', float32Tag = '[object Float32Array]', float64Tag = '[object Float64Array]', int8Tag = '[object Int8Array]', int16Tag = '[object Int16Array]', int32Tag = '[object Int32Array]', uint8Tag = '[object Uint8Array]', uint8ClampedTag = '[object Uint8ClampedArray]', uint16Tag = '[object Uint16Array]', uint32Tag = '[object Uint32Array]';
/** Used to identify `toStringTag` values of typed arrays. */ var typedArrayTags = {};
typedArrayTags[float32Tag] = typedArrayTags[float64Tag] = typedArrayTags[int8Tag] = typedArrayTags[int16Tag] = typedArrayTags[int32Tag] = typedArrayTags[uint8Tag] = typedArrayTags[uint8ClampedTag] = typedArrayTags[uint16Tag] = typedArrayTags[uint32Tag] = true;
typedArrayTags[argsTag] = typedArrayTags[arrayTag] = typedArrayTags[arrayBufferTag] = typedArrayTags[boolTag] = typedArrayTags[dataViewTag] = typedArrayTags[dateTag] = typedArrayTags[errorTag] = typedArrayTags[funcTag] = typedArrayTags[mapTag] = typedArrayTags[numberTag] = typedArrayTags[objectTag] = typedArrayTags[regexpTag] = typedArrayTags[setTag] = typedArrayTags[stringTag] = typedArrayTags[weakMapTag] = false;
/**
 * The base implementation of `_.isTypedArray` without Node.js optimizations.
 *
 * @private
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a typed array, else `false`.
 */ function baseIsTypedArray(value) {
    return (0, _isObjectLike.default)(value) && (0, _isLength.default)(value.length) && !!typedArrayTags[(0, _baseGetTag.default)(value)];
}
const _default = baseIsTypedArray;

},
"e8467ae5": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Removes all key-value entries from the list cache.
 *
 * @private
 * @name clear
 * @memberOf ListCache
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function listCacheClear() {
    this.__data__ = [];
    this.size = 0;
}
const _default = listCacheClear;

},
"ea0fed91": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Checks if `value` is `undefined`.
 *
 * @static
 * @since 0.1.0
 * @memberOf _
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is `undefined`, else `false`.
 * @example
 *
 * _.isUndefined(void 0);
 * // => true
 *
 * _.isUndefined(null);
 * // => false
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function isUndefined(value) {
    return value === undefined;
}
const _default = isUndefined;

},
"eb158937": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _forEach.default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _forEach = /*#__PURE__*/ _interop_require_default._(farmRequire("d0910965"));

},
"ec3f319a": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseAssignValue = /*#__PURE__*/ _interop_require_default._(farmRequire("53dc8b2a"));
const _eq = /*#__PURE__*/ _interop_require_default._(farmRequire("3e794592"));
/**
 * This function is like `assignValue` except that it doesn't assign
 * `undefined` values.
 *
 * @private
 * @param {Object} object The object to modify.
 * @param {string} key The key of the property to assign.
 * @param {*} value The value to assign.
 */ function assignMergeValue(object, key, value) {
    if (value !== undefined && !(0, _eq.default)(object[key], value) || value === undefined && !(key in object)) {
        (0, _baseAssignValue.default)(object, key, value);
    }
}
const _default = assignMergeValue;

},
"f1ceb9be": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Checks if `value` is classified as an `Array` object.
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is an array, else `false`.
 * @example
 *
 * _.isArray([1, 2, 3]);
 * // => true
 *
 * _.isArray(document.body.children);
 * // => false
 *
 * _.isArray('abc');
 * // => false
 *
 * _.isArray(_.noop);
 * // => false
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var isArray = Array.isArray;
const _default = isArray;

},
"f213122b": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * The base implementation of `_.unary` without support for storing metadata.
 *
 * @private
 * @param {Function} func The function to cap arguments for.
 * @returns {Function} Returns the new capped function.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function baseUnary(func) {
    return function(value) {
        return func(value);
    };
}
const _default = baseUnary;

},
"f23e6f4b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _getMapData = /*#__PURE__*/ _interop_require_default._(farmRequire("d6ffea67"));
/**
 * Gets the map value for `key`.
 *
 * @private
 * @name get
 * @memberOf MapCache
 * @param {string} key The key of the value to get.
 * @returns {*} Returns the entry value.
 */ function mapCacheGet(key) {
    return (0, _getMapData.default)(this, key).get(key);
}
const _default = mapCacheGet;

},
"f27471f3": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Removes `key` and its value from the hash.
 *
 * @private
 * @name delete
 * @memberOf Hash
 * @param {Object} hash The hash to modify.
 * @param {string} key The key of the value to remove.
 * @returns {boolean} Returns `true` if the entry was removed, else `false`.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function hashDelete(key) {
    var result = this.has(key) && delete this.__data__[key];
    this.size -= result ? 1 : 0;
    return result;
}
const _default = hashDelete;

},
"f29f63ea": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _nativeCreate = /*#__PURE__*/ _interop_require_default._(farmRequire("dc218a61"));
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/**
 * Checks if a hash value for `key` exists.
 *
 * @private
 * @name has
 * @memberOf Hash
 * @param {string} key The key of the entry to check.
 * @returns {boolean} Returns `true` if an entry for `key` exists, else `false`.
 */ function hashHas(key) {
    var data = this.__data__;
    return _nativeCreate.default ? data[key] !== undefined : hasOwnProperty.call(data, key);
}
const _default = hashHas;

},
"f31c2091": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Checks if `value` is suitable for use as unique object key.
 *
 * @private
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is suitable, else `false`.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function isKeyable(value) {
    var type = typeof value;
    return type == 'string' || type == 'number' || type == 'symbol' || type == 'boolean' ? value !== '__proto__' : value === null;
}
const _default = isKeyable;

},
"f37b1f98": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _root = /*#__PURE__*/ _interop_require_default._(farmRequire("6bf7b771"));
/**
 * Gets the timestamp of the number of milliseconds that have elapsed since
 * the Unix epoch (1 January 1970 00:00:00 UTC).
 *
 * @static
 * @memberOf _
 * @since 2.4.0
 * @category Date
 * @returns {number} Returns the timestamp.
 * @example
 *
 * _.defer(function(stamp) {
 *   console.log(_.now() - stamp);
 * }, _.now());
 * // => Logs the number of milliseconds it took for the deferred invocation.
 */ var now = function() {
    return _root.default.Date.now();
};
const _default = now;

},
"f40f3977": function(module, exports, farmRequire, farmDynamicRequire) {
/* eslint-disable no-param-reassign */ "use strict";
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
    SketchFields: function() {
        return SketchFields;
    },
    default: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactcss = /*#__PURE__*/ _interop_require_default._(farmRequire("c77bba68"));
const _color = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("889a06b2"));
const _common = farmRequire("0c79d683");
var SketchFields = function SketchFields(_ref) {
    var onChange = _ref.onChange, rgb = _ref.rgb, hsl = _ref.hsl, hex = _ref.hex, disableAlpha = _ref.disableAlpha;
    var styles = (0, _reactcss.default)({
        'default': {
            fields: {
                display: 'flex',
                paddingTop: '4px'
            },
            single: {
                flex: '1',
                paddingLeft: '6px'
            },
            alpha: {
                flex: '1',
                paddingLeft: '6px'
            },
            double: {
                flex: '2'
            },
            input: {
                width: '80%',
                padding: '4px 10% 3px',
                border: 'none',
                boxShadow: 'inset 0 0 0 1px #ccc',
                fontSize: '11px'
            },
            label: {
                display: 'block',
                textAlign: 'center',
                fontSize: '11px',
                color: '#222',
                paddingTop: '3px',
                paddingBottom: '4px',
                textTransform: 'capitalize'
            }
        },
        'disableAlpha': {
            alpha: {
                display: 'none'
            }
        }
    }, {
        disableAlpha: disableAlpha
    });
    var handleChange = function handleChange(data, e) {
        if (data.hex) {
            _color.isValidHex(data.hex) && onChange({
                hex: data.hex,
                source: 'hex'
            }, e);
        } else if (data.r || data.g || data.b) {
            onChange({
                r: data.r || rgb.r,
                g: data.g || rgb.g,
                b: data.b || rgb.b,
                a: rgb.a,
                source: 'rgb'
            }, e);
        } else if (data.a) {
            if (data.a < 0) {
                data.a = 0;
            } else if (data.a > 100) {
                data.a = 100;
            }
            data.a /= 100;
            onChange({
                h: hsl.h,
                s: hsl.s,
                l: hsl.l,
                a: data.a,
                source: 'rgb'
            }, e);
        }
    };
    return _react.default.createElement('div', {
        style: styles.fields,
        className: 'flexbox-fix'
    }, _react.default.createElement('div', {
        style: styles.double
    }, _react.default.createElement(_common.EditableInput, {
        style: {
            input: styles.input,
            label: styles.label
        },
        label: 'hex',
        value: hex.replace('#', ''),
        onChange: handleChange
    })), _react.default.createElement('div', {
        style: styles.single
    }, _react.default.createElement(_common.EditableInput, {
        style: {
            input: styles.input,
            label: styles.label
        },
        label: 'r',
        value: rgb.r,
        onChange: handleChange,
        dragLabel: 'true',
        dragMax: '255'
    })), _react.default.createElement('div', {
        style: styles.single
    }, _react.default.createElement(_common.EditableInput, {
        style: {
            input: styles.input,
            label: styles.label
        },
        label: 'g',
        value: rgb.g,
        onChange: handleChange,
        dragLabel: 'true',
        dragMax: '255'
    })), _react.default.createElement('div', {
        style: styles.single
    }, _react.default.createElement(_common.EditableInput, {
        style: {
            input: styles.input,
            label: styles.label
        },
        label: 'b',
        value: rgb.b,
        onChange: handleChange,
        dragLabel: 'true',
        dragMax: '255'
    })), _react.default.createElement('div', {
        style: styles.alpha
    }, _react.default.createElement(_common.EditableInput, {
        style: {
            input: styles.input,
            label: styles.label
        },
        label: 'a',
        value: Math.round(rgb.a * 100),
        onChange: handleChange,
        dragLabel: 'true',
        dragMax: '100'
    })));
};
const _default = SketchFields;

},
"f43107b6": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * Gets the stack value for `key`.
 *
 * @private
 * @name get
 * @memberOf Stack
 * @param {string} key The key of the value to get.
 * @returns {*} Returns the entry value.
 */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function stackGet(key) {
    return this.__data__.get(key);
}
const _default = stackGet;

},
"f7a29359": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _ListCache = /*#__PURE__*/ _interop_require_default._(farmRequire("47a014c7"));
const _stackClear = /*#__PURE__*/ _interop_require_default._(farmRequire("d01e25ba"));
const _stackDelete = /*#__PURE__*/ _interop_require_default._(farmRequire("e44b5e2d"));
const _stackGet = /*#__PURE__*/ _interop_require_default._(farmRequire("f43107b6"));
const _stackHas = /*#__PURE__*/ _interop_require_default._(farmRequire("204518c5"));
const _stackSet = /*#__PURE__*/ _interop_require_default._(farmRequire("5be84108"));
/**
 * Creates a stack cache object to store key-value pairs.
 *
 * @private
 * @constructor
 * @param {Array} [entries] The key-value pairs to cache.
 */ function Stack(entries) {
    var data = this.__data__ = new _ListCache.default(entries);
    this.size = data.size;
}
// Add methods to `Stack`.
Stack.prototype.clear = _stackClear.default;
Stack.prototype['delete'] = _stackDelete.default;
Stack.prototype.get = _stackGet.default;
Stack.prototype.has = _stackHas.default;
Stack.prototype.set = _stackSet.default;
const _default = Stack;

},
"f96f20cf": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _mapCacheClear = /*#__PURE__*/ _interop_require_default._(farmRequire("1144a658"));
const _mapCacheDelete = /*#__PURE__*/ _interop_require_default._(farmRequire("9fa65b27"));
const _mapCacheGet = /*#__PURE__*/ _interop_require_default._(farmRequire("f23e6f4b"));
const _mapCacheHas = /*#__PURE__*/ _interop_require_default._(farmRequire("509c9c80"));
const _mapCacheSet = /*#__PURE__*/ _interop_require_default._(farmRequire("b995b8c7"));
/**
 * Creates a map cache object to store key-value pairs.
 *
 * @private
 * @constructor
 * @param {Array} [entries] The key-value pairs to cache.
 */ function MapCache(entries) {
    var index = -1, length = entries == null ? 0 : entries.length;
    this.clear();
    while(++index < length){
        var entry = entries[index];
        this.set(entry[0], entry[1]);
    }
}
// Add methods to `MapCache`.
MapCache.prototype.clear = _mapCacheClear.default;
MapCache.prototype['delete'] = _mapCacheDelete.default;
MapCache.prototype.get = _mapCacheGet.default;
MapCache.prototype.has = _mapCacheHas.default;
MapCache.prototype.set = _mapCacheSet.default;
const _default = MapCache;

},
"fac7b69f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseGetTag = /*#__PURE__*/ _interop_require_default._(farmRequire("4ffcc116"));
const _getPrototype = /*#__PURE__*/ _interop_require_default._(farmRequire("e251ea03"));
const _isObjectLike = /*#__PURE__*/ _interop_require_default._(farmRequire("1954b1ef"));
/** `Object#toString` result references. */ var objectTag = '[object Object]';
/** Used for built-in method references. */ var funcProto = Function.prototype, objectProto = Object.prototype;
/** Used to resolve the decompiled source of functions. */ var funcToString = funcProto.toString;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/** Used to infer the `Object` constructor. */ var objectCtorString = funcToString.call(Object);
/**
 * Checks if `value` is a plain object, that is, an object created by the
 * `Object` constructor or one with a `[[Prototype]]` of `null`.
 *
 * @static
 * @memberOf _
 * @since 0.8.0
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a plain object, else `false`.
 * @example
 *
 * function Foo() {
 *   this.a = 1;
 * }
 *
 * _.isPlainObject(new Foo);
 * // => false
 *
 * _.isPlainObject([1, 2, 3]);
 * // => false
 *
 * _.isPlainObject({ 'x': 0, 'y': 0 });
 * // => true
 *
 * _.isPlainObject(Object.create(null));
 * // => true
 */ function isPlainObject(value) {
    if (!(0, _isObjectLike.default)(value) || (0, _baseGetTag.default)(value) != objectTag) {
        return false;
    }
    var proto = (0, _getPrototype.default)(value);
    if (proto === null) {
        return true;
    }
    var Ctor = hasOwnProperty.call(proto, 'constructor') && proto.constructor;
    return typeof Ctor == 'function' && Ctor instanceof Ctor && funcToString.call(Ctor) == objectCtorString;
}
const _default = isPlainObject;

},
"fae4ae0b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _baseTimes = /*#__PURE__*/ _interop_require_default._(farmRequire("7bb75378"));
const _isArguments = /*#__PURE__*/ _interop_require_default._(farmRequire("23576120"));
const _isArray = /*#__PURE__*/ _interop_require_default._(farmRequire("f1ceb9be"));
const _isBuffer = /*#__PURE__*/ _interop_require_default._(farmRequire("356fea7f"));
const _isIndex = /*#__PURE__*/ _interop_require_default._(farmRequire("6f06a530"));
const _isTypedArray = /*#__PURE__*/ _interop_require_default._(farmRequire("a8b5d78d"));
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/**
 * Creates an array of the enumerable property names of the array-like `value`.
 *
 * @private
 * @param {*} value The value to query.
 * @param {boolean} inherited Specify returning inherited property names.
 * @returns {Array} Returns the array of property names.
 */ function arrayLikeKeys(value, inherited) {
    var isArr = (0, _isArray.default)(value), isArg = !isArr && (0, _isArguments.default)(value), isBuff = !isArr && !isArg && (0, _isBuffer.default)(value), isType = !isArr && !isArg && !isBuff && (0, _isTypedArray.default)(value), skipIndexes = isArr || isArg || isBuff || isType, result = skipIndexes ? (0, _baseTimes.default)(value.length, String) : [], length = result.length;
    for(var key in value){
        if ((inherited || hasOwnProperty.call(value, key)) && !(skipIndexes && // Safari 9 has enumerable `arguments.length` in strict mode.
        (key == 'length' || // Node.js 0.10 has enumerable non-index properties on buffers.
        isBuff && (key == 'offset' || key == 'parent') || // PhantomJS 2 has enumerable non-index properties on typed arrays.
        isType && (key == 'buffer' || key == 'byteLength' || key == 'byteOffset') || // Skip index properties.
        (0, _isIndex.default)(key, length)))) {
            result.push(key);
        }
    }
    return result;
}
const _default = arrayLikeKeys;

},});