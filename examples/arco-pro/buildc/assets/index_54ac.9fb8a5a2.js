(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_54ac.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"0403dd72": function(module, exports, farmRequire, farmDynamicRequire) {
var baseProperty = farmRequire("88545e9c", true), basePropertyDeep = farmRequire("aa0f4db3", true), isKey = farmRequire("82ab411e", true), toKey = farmRequire("12b97de7", true);
/**
 * Creates a function that returns the value at `path` of a given object.
 *
 * @static
 * @memberOf _
 * @since 2.4.0
 * @category Util
 * @param {Array|string} path The path of the property to get.
 * @returns {Function} Returns the new accessor function.
 * @example
 *
 * var objects = [
 *   { 'a': { 'b': 2 } },
 *   { 'a': { 'b': 1 } }
 * ];
 *
 * _.map(objects, _.property('a.b'));
 * // => [2, 1]
 *
 * _.map(_.sortBy(objects, _.property(['a', 'b'])), 'a.b');
 * // => [1, 2]
 */ function property(path) {
    return isKey(path) ? baseProperty(toKey(path)) : basePropertyDeep(path);
}
module.exports = property;

},
"06db9f0a": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * The base implementation of `_.hasIn` without support for deep paths.
 *
 * @private
 * @param {Object} [object] The object to query.
 * @param {Array|string} key The key to check.
 * @returns {boolean} Returns `true` if `key` exists, else `false`.
 */ function baseHasIn(object, key) {
    return object != null && key in Object(object);
}
module.exports = baseHasIn;

},
"0a59b4ed": function(module, exports, farmRequire, farmDynamicRequire) {
var baseAssignValue = farmRequire("9deec6ca", true), createAggregator = farmRequire("64304e4f", true);
/** Used for built-in method references. */ var objectProto = Object.prototype;
/** Used to check objects for own properties. */ var hasOwnProperty = objectProto.hasOwnProperty;
/**
 * Creates an object composed of keys generated from the results of running
 * each element of `collection` thru `iteratee`. The order of grouped values
 * is determined by the order they occur in `collection`. The corresponding
 * value of each key is an array of elements responsible for generating the
 * key. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @category Collection
 * @param {Array|Object} collection The collection to iterate over.
 * @param {Function} [iteratee=_.identity] The iteratee to transform keys.
 * @returns {Object} Returns the composed aggregate object.
 * @example
 *
 * _.groupBy([6.1, 4.2, 6.3], Math.floor);
 * // => { '4': [4.2], '6': [6.1, 6.3] }
 *
 * // The `_.property` iteratee shorthand.
 * _.groupBy(['one', 'two', 'three'], 'length');
 * // => { '3': ['one', 'two'], '5': ['three'] }
 */ var groupBy = createAggregator(function(result, value, key) {
    if (hasOwnProperty.call(result, key)) {
        result[key].push(value);
    } else {
        baseAssignValue(result, key, [
            value
        ]);
    }
});
module.exports = groupBy;

},
"0ede0ab4": function(module, exports, farmRequire, farmDynamicRequire) {
var arrayMap = farmRequire("e31ac504", true), baseIteratee = farmRequire("248e0b34", true), baseMap = farmRequire("0ffc9518", true), isArray = farmRequire("eff342ff", true);
/**
 * Creates an array of values by running each element in `collection` thru
 * `iteratee`. The iteratee is invoked with three arguments:
 * (value, index|key, collection).
 *
 * Many lodash methods are guarded to work as iteratees for methods like
 * `_.every`, `_.filter`, `_.map`, `_.mapValues`, `_.reject`, and `_.some`.
 *
 * The guarded methods are:
 * `ary`, `chunk`, `curry`, `curryRight`, `drop`, `dropRight`, `every`,
 * `fill`, `invert`, `parseInt`, `random`, `range`, `rangeRight`, `repeat`,
 * `sampleSize`, `slice`, `some`, `sortBy`, `split`, `take`, `takeRight`,
 * `template`, `trim`, `trimEnd`, `trimStart`, and `words`
 *
 * @static
 * @memberOf _
 * @since 0.1.0
 * @category Collection
 * @param {Array|Object} collection The collection to iterate over.
 * @param {Function} [iteratee=_.identity] The function invoked per iteration.
 * @returns {Array} Returns the new mapped array.
 * @example
 *
 * function square(n) {
 *   return n * n;
 * }
 *
 * _.map([4, 8], square);
 * // => [16, 64]
 *
 * _.map({ 'a': 4, 'b': 8 }, square);
 * // => [16, 64] (iteration order is not guaranteed)
 *
 * var users = [
 *   { 'user': 'barney' },
 *   { 'user': 'fred' }
 * ];
 *
 * // The `_.property` iteratee shorthand.
 * _.map(users, 'user');
 * // => ['barney', 'fred']
 */ function map(collection, iteratee) {
    var func = isArray(collection) ? arrayMap : baseMap;
    return func(collection, baseIteratee(iteratee, 3));
}
module.exports = map;

},
"0ffc9518": function(module, exports, farmRequire, farmDynamicRequire) {
var baseEach = farmRequire("4c7d0c51", true), isArrayLike = farmRequire("15a95643", true);
/**
 * The base implementation of `_.map` without support for iteratee shorthands.
 *
 * @private
 * @param {Array|Object} collection The collection to iterate over.
 * @param {Function} iteratee The function invoked per iteration.
 * @returns {Array} Returns the new mapped array.
 */ function baseMap(collection, iteratee) {
    var index = -1, result = isArrayLike(collection) ? Array(collection.length) : [];
    baseEach(collection, function(value, key, collection) {
        result[++index] = iteratee(value, key, collection);
    });
    return result;
}
module.exports = baseMap;

},
"13614e07": function(module, exports, farmRequire, farmDynamicRequire) {
var isArrayLike = farmRequire("15a95643", true);
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
        if (!isArrayLike(collection)) {
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
module.exports = createBaseEach;

},
"1b3a60c0": function(module, exports, farmRequire, farmDynamicRequire) {
'use strict';
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.autoprefix = undefined;
var _forOwn2 = farmRequire("74cb39e4", true);
var _forOwn3 = _interopRequireDefault(_forOwn2);
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
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
var transforms = {
    borderRadius: function borderRadius(value) {
        return {
            msBorderRadius: value,
            MozBorderRadius: value,
            OBorderRadius: value,
            WebkitBorderRadius: value,
            borderRadius: value
        };
    },
    boxShadow: function boxShadow(value) {
        return {
            msBoxShadow: value,
            MozBoxShadow: value,
            OBoxShadow: value,
            WebkitBoxShadow: value,
            boxShadow: value
        };
    },
    userSelect: function userSelect(value) {
        return {
            WebkitTouchCallout: value,
            KhtmlUserSelect: value,
            MozUserSelect: value,
            msUserSelect: value,
            WebkitUserSelect: value,
            userSelect: value
        };
    },
    flex: function flex(value) {
        return {
            WebkitBoxFlex: value,
            MozBoxFlex: value,
            WebkitFlex: value,
            msFlex: value,
            flex: value
        };
    },
    flexBasis: function flexBasis(value) {
        return {
            WebkitFlexBasis: value,
            flexBasis: value
        };
    },
    justifyContent: function justifyContent(value) {
        return {
            WebkitJustifyContent: value,
            justifyContent: value
        };
    },
    transition: function transition(value) {
        return {
            msTransition: value,
            MozTransition: value,
            OTransition: value,
            WebkitTransition: value,
            transition: value
        };
    },
    transform: function transform(value) {
        return {
            msTransform: value,
            MozTransform: value,
            OTransform: value,
            WebkitTransform: value,
            transform: value
        };
    },
    absolute: function absolute(value) {
        var direction = value && value.split(' ');
        return {
            position: 'absolute',
            top: direction && direction[0],
            right: direction && direction[1],
            bottom: direction && direction[2],
            left: direction && direction[3]
        };
    },
    extend: function extend(name, otherElementStyles) {
        var otherStyle = otherElementStyles[name];
        if (otherStyle) {
            return otherStyle;
        }
        return {
            'extend': name
        };
    }
};
var autoprefix = exports.autoprefix = function autoprefix(elements) {
    var prefixed = {};
    (0, _forOwn3.default)(elements, function(styles, element) {
        var expanded = {};
        (0, _forOwn3.default)(styles, function(value, key) {
            var transform = transforms[key];
            if (transform) {
                expanded = _extends({}, expanded, transform(value));
            } else {
                expanded[key] = value;
            }
        });
        prefixed[element] = expanded;
    });
    return prefixed;
};
exports.default = autoprefix;

},
"1e09a1df": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * A specialized version of `matchesProperty` for source values suitable
 * for strict equality comparisons, i.e. `===`.
 *
 * @private
 * @param {string} key The key of the property to get.
 * @param {*} srcValue The value to match.
 * @returns {Function} Returns the new spec function.
 */ function matchesStrictComparable(key, srcValue) {
    return function(object) {
        if (object == null) {
            return false;
        }
        return object[key] === srcValue && (srcValue !== undefined || key in Object(object));
    };
}
module.exports = matchesStrictComparable;

},
"219b7102": function(module, exports, farmRequire, farmDynamicRequire) {
var baseFor = farmRequire("fbfede77", true), keys = farmRequire("ed28e463", true);
/**
 * The base implementation of `_.forOwn` without support for iteratee shorthands.
 *
 * @private
 * @param {Object} object The object to iterate over.
 * @param {Function} iteratee The function invoked per iteration.
 * @returns {Object} Returns `object`.
 */ function baseForOwn(object, iteratee) {
    return object && baseFor(object, iteratee, keys);
}
module.exports = baseForOwn;

},
"248e0b34": function(module, exports, farmRequire, farmDynamicRequire) {
var baseMatches = farmRequire("fc636423", true), baseMatchesProperty = farmRequire("6959fd39", true), identity = farmRequire("c6fbe0a8", true), isArray = farmRequire("eff342ff", true), property = farmRequire("0403dd72", true);
/**
 * The base implementation of `_.iteratee`.
 *
 * @private
 * @param {*} [value=_.identity] The value to convert to an iteratee.
 * @returns {Function} Returns the iteratee.
 */ function baseIteratee(value) {
    // Don't store the `typeof` result in a variable to avoid a JIT bug in Safari 9.
    // See https://bugs.webkit.org/show_bug.cgi?id=156034 for more details.
    if (typeof value == 'function') {
        return value;
    }
    if (value == null) {
        return identity;
    }
    if (typeof value == 'object') {
        return isArray(value) ? baseMatchesProperty(value[0], value[1]) : baseMatches(value);
    }
    return property(value);
}
module.exports = baseIteratee;

},
"28569ddf": function(module, exports, farmRequire, farmDynamicRequire) {
var baseGetTag = farmRequire("5bdd1813", true), isArray = farmRequire("eff342ff", true), isObjectLike = farmRequire("5d2483c0", true);
/** `Object#toString` result references. */ var stringTag = '[object String]';
/**
 * Checks if `value` is classified as a `String` primitive or object.
 *
 * @static
 * @since 0.1.0
 * @memberOf _
 * @category Lang
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` is a string, else `false`.
 * @example
 *
 * _.isString('abc');
 * // => true
 *
 * _.isString(1);
 * // => false
 */ function isString(value) {
    return typeof value == 'string' || !isArray(value) && isObjectLike(value) && baseGetTag(value) == stringTag;
}
module.exports = isString;

},
"355763f4": function(module, exports, farmRequire, farmDynamicRequire) {
'use strict';
var colorString = farmRequire("86bb81b8", true);
var convert = farmRequire("f14759ef", true);
var _slice = [].slice;
var skippedModels = [
    // to be honest, I don't really feel like keyword belongs in color convert, but eh.
    'keyword',
    // gray conflicts with some method names, and has its own method defined.
    'gray',
    // shouldn't really be in color-convert either...
    'hex'
];
var hashedModelKeys = {};
Object.keys(convert).forEach(function(model) {
    hashedModelKeys[_slice.call(convert[model].labels).sort().join('')] = model;
});
var limiters = {};
function Color(obj, model) {
    if (!(this instanceof Color)) {
        return new Color(obj, model);
    }
    if (model && model in skippedModels) {
        model = null;
    }
    if (model && !(model in convert)) {
        throw new Error('Unknown model: ' + model);
    }
    var i;
    var channels;
    if (obj == null) {
        this.model = 'rgb';
        this.color = [
            0,
            0,
            0
        ];
        this.valpha = 1;
    } else if (obj instanceof Color) {
        this.model = obj.model;
        this.color = obj.color.slice();
        this.valpha = obj.valpha;
    } else if (typeof obj === 'string') {
        var result = colorString.get(obj);
        if (result === null) {
            throw new Error('Unable to parse color from string: ' + obj);
        }
        this.model = result.model;
        channels = convert[this.model].channels;
        this.color = result.value.slice(0, channels);
        this.valpha = typeof result.value[channels] === 'number' ? result.value[channels] : 1;
    } else if (obj.length) {
        this.model = model || 'rgb';
        channels = convert[this.model].channels;
        var newArr = _slice.call(obj, 0, channels);
        this.color = zeroArray(newArr, channels);
        this.valpha = typeof obj[channels] === 'number' ? obj[channels] : 1;
    } else if (typeof obj === 'number') {
        // this is always RGB - can be converted later on.
        obj &= 0xFFFFFF;
        this.model = 'rgb';
        this.color = [
            obj >> 16 & 0xFF,
            obj >> 8 & 0xFF,
            obj & 0xFF
        ];
        this.valpha = 1;
    } else {
        this.valpha = 1;
        var keys = Object.keys(obj);
        if ('alpha' in obj) {
            keys.splice(keys.indexOf('alpha'), 1);
            this.valpha = typeof obj.alpha === 'number' ? obj.alpha : 0;
        }
        var hashedKeys = keys.sort().join('');
        if (!(hashedKeys in hashedModelKeys)) {
            throw new Error('Unable to parse color from object: ' + JSON.stringify(obj));
        }
        this.model = hashedModelKeys[hashedKeys];
        var labels = convert[this.model].labels;
        var color = [];
        for(i = 0; i < labels.length; i++){
            color.push(obj[labels[i]]);
        }
        this.color = zeroArray(color);
    }
    // perform limitations (clamping, etc.)
    if (limiters[this.model]) {
        channels = convert[this.model].channels;
        for(i = 0; i < channels; i++){
            var limit = limiters[this.model][i];
            if (limit) {
                this.color[i] = limit(this.color[i]);
            }
        }
    }
    this.valpha = Math.max(0, Math.min(1, this.valpha));
    if (Object.freeze) {
        Object.freeze(this);
    }
}
Color.prototype = {
    toString: function() {
        return this.string();
    },
    toJSON: function() {
        return this[this.model]();
    },
    string: function(places) {
        var self = this.model in colorString.to ? this : this.rgb();
        self = self.round(typeof places === 'number' ? places : 1);
        var args = self.valpha === 1 ? self.color : self.color.concat(this.valpha);
        return colorString.to[self.model](args);
    },
    percentString: function(places) {
        var self = this.rgb().round(typeof places === 'number' ? places : 1);
        var args = self.valpha === 1 ? self.color : self.color.concat(this.valpha);
        return colorString.to.rgb.percent(args);
    },
    array: function() {
        return this.valpha === 1 ? this.color.slice() : this.color.concat(this.valpha);
    },
    object: function() {
        var result = {};
        var channels = convert[this.model].channels;
        var labels = convert[this.model].labels;
        for(var i = 0; i < channels; i++){
            result[labels[i]] = this.color[i];
        }
        if (this.valpha !== 1) {
            result.alpha = this.valpha;
        }
        return result;
    },
    unitArray: function() {
        var rgb = this.rgb().color;
        rgb[0] /= 255;
        rgb[1] /= 255;
        rgb[2] /= 255;
        if (this.valpha !== 1) {
            rgb.push(this.valpha);
        }
        return rgb;
    },
    unitObject: function() {
        var rgb = this.rgb().object();
        rgb.r /= 255;
        rgb.g /= 255;
        rgb.b /= 255;
        if (this.valpha !== 1) {
            rgb.alpha = this.valpha;
        }
        return rgb;
    },
    round: function(places) {
        places = Math.max(places || 0, 0);
        return new Color(this.color.map(roundToPlace(places)).concat(this.valpha), this.model);
    },
    alpha: function(val) {
        if (arguments.length) {
            return new Color(this.color.concat(Math.max(0, Math.min(1, val))), this.model);
        }
        return this.valpha;
    },
    // rgb
    red: getset('rgb', 0, maxfn(255)),
    green: getset('rgb', 1, maxfn(255)),
    blue: getset('rgb', 2, maxfn(255)),
    hue: getset([
        'hsl',
        'hsv',
        'hsl',
        'hwb',
        'hcg'
    ], 0, function(val) {
        return (val % 360 + 360) % 360;
    }),
    saturationl: getset('hsl', 1, maxfn(100)),
    lightness: getset('hsl', 2, maxfn(100)),
    saturationv: getset('hsv', 1, maxfn(100)),
    value: getset('hsv', 2, maxfn(100)),
    chroma: getset('hcg', 1, maxfn(100)),
    gray: getset('hcg', 2, maxfn(100)),
    white: getset('hwb', 1, maxfn(100)),
    wblack: getset('hwb', 2, maxfn(100)),
    cyan: getset('cmyk', 0, maxfn(100)),
    magenta: getset('cmyk', 1, maxfn(100)),
    yellow: getset('cmyk', 2, maxfn(100)),
    black: getset('cmyk', 3, maxfn(100)),
    x: getset('xyz', 0, maxfn(100)),
    y: getset('xyz', 1, maxfn(100)),
    z: getset('xyz', 2, maxfn(100)),
    l: getset('lab', 0, maxfn(100)),
    a: getset('lab', 1),
    b: getset('lab', 2),
    keyword: function(val) {
        if (arguments.length) {
            return new Color(val);
        }
        return convert[this.model].keyword(this.color);
    },
    hex: function(val) {
        if (arguments.length) {
            return new Color(val);
        }
        return colorString.to.hex(this.rgb().round().color);
    },
    rgbNumber: function() {
        var rgb = this.rgb().color;
        return (rgb[0] & 0xFF) << 16 | (rgb[1] & 0xFF) << 8 | rgb[2] & 0xFF;
    },
    luminosity: function() {
        // http://www.w3.org/TR/WCAG20/#relativeluminancedef
        var rgb = this.rgb().color;
        var lum = [];
        for(var i = 0; i < rgb.length; i++){
            var chan = rgb[i] / 255;
            lum[i] = chan <= 0.03928 ? chan / 12.92 : Math.pow((chan + 0.055) / 1.055, 2.4);
        }
        return 0.2126 * lum[0] + 0.7152 * lum[1] + 0.0722 * lum[2];
    },
    contrast: function(color2) {
        // http://www.w3.org/TR/WCAG20/#contrast-ratiodef
        var lum1 = this.luminosity();
        var lum2 = color2.luminosity();
        if (lum1 > lum2) {
            return (lum1 + 0.05) / (lum2 + 0.05);
        }
        return (lum2 + 0.05) / (lum1 + 0.05);
    },
    level: function(color2) {
        var contrastRatio = this.contrast(color2);
        if (contrastRatio >= 7.1) {
            return 'AAA';
        }
        return contrastRatio >= 4.5 ? 'AA' : '';
    },
    isDark: function() {
        // YIQ equation from http://24ways.org/2010/calculating-color-contrast
        var rgb = this.rgb().color;
        var yiq = (rgb[0] * 299 + rgb[1] * 587 + rgb[2] * 114) / 1000;
        return yiq < 128;
    },
    isLight: function() {
        return !this.isDark();
    },
    negate: function() {
        var rgb = this.rgb();
        for(var i = 0; i < 3; i++){
            rgb.color[i] = 255 - rgb.color[i];
        }
        return rgb;
    },
    lighten: function(ratio) {
        var hsl = this.hsl();
        hsl.color[2] += hsl.color[2] * ratio;
        return hsl;
    },
    darken: function(ratio) {
        var hsl = this.hsl();
        hsl.color[2] -= hsl.color[2] * ratio;
        return hsl;
    },
    saturate: function(ratio) {
        var hsl = this.hsl();
        hsl.color[1] += hsl.color[1] * ratio;
        return hsl;
    },
    desaturate: function(ratio) {
        var hsl = this.hsl();
        hsl.color[1] -= hsl.color[1] * ratio;
        return hsl;
    },
    whiten: function(ratio) {
        var hwb = this.hwb();
        hwb.color[1] += hwb.color[1] * ratio;
        return hwb;
    },
    blacken: function(ratio) {
        var hwb = this.hwb();
        hwb.color[2] += hwb.color[2] * ratio;
        return hwb;
    },
    grayscale: function() {
        // http://en.wikipedia.org/wiki/Grayscale#Converting_color_to_grayscale
        var rgb = this.rgb().color;
        var val = rgb[0] * 0.3 + rgb[1] * 0.59 + rgb[2] * 0.11;
        return Color.rgb(val, val, val);
    },
    fade: function(ratio) {
        return this.alpha(this.valpha - this.valpha * ratio);
    },
    opaquer: function(ratio) {
        return this.alpha(this.valpha + this.valpha * ratio);
    },
    rotate: function(degrees) {
        var hsl = this.hsl();
        var hue = hsl.color[0];
        hue = (hue + degrees) % 360;
        hue = hue < 0 ? 360 + hue : hue;
        hsl.color[0] = hue;
        return hsl;
    },
    mix: function(mixinColor, weight) {
        // ported from sass implementation in C
        // https://github.com/sass/libsass/blob/0e6b4a2850092356aa3ece07c6b249f0221caced/functions.cpp#L209
        if (!mixinColor || !mixinColor.rgb) {
            throw new Error('Argument to "mix" was not a Color instance, but rather an instance of ' + typeof mixinColor);
        }
        var color1 = mixinColor.rgb();
        var color2 = this.rgb();
        var p = weight === undefined ? 0.5 : weight;
        var w = 2 * p - 1;
        var a = color1.alpha() - color2.alpha();
        var w1 = ((w * a === -1 ? w : (w + a) / (1 + w * a)) + 1) / 2.0;
        var w2 = 1 - w1;
        return Color.rgb(w1 * color1.red() + w2 * color2.red(), w1 * color1.green() + w2 * color2.green(), w1 * color1.blue() + w2 * color2.blue(), color1.alpha() * p + color2.alpha() * (1 - p));
    }
};
// model conversion methods and static constructors
Object.keys(convert).forEach(function(model) {
    if (skippedModels.indexOf(model) !== -1) {
        return;
    }
    var channels = convert[model].channels;
    // conversion methods
    Color.prototype[model] = function() {
        if (this.model === model) {
            return new Color(this);
        }
        if (arguments.length) {
            return new Color(arguments, model);
        }
        var newAlpha = typeof arguments[channels] === 'number' ? channels : this.valpha;
        return new Color(assertArray(convert[this.model][model].raw(this.color)).concat(newAlpha), model);
    };
    // 'static' construction methods
    Color[model] = function(color) {
        if (typeof color === 'number') {
            color = zeroArray(_slice.call(arguments), channels);
        }
        return new Color(color, model);
    };
});
function roundTo(num, places) {
    return Number(num.toFixed(places));
}
function roundToPlace(places) {
    return function(num) {
        return roundTo(num, places);
    };
}
function getset(model, channel, modifier) {
    model = Array.isArray(model) ? model : [
        model
    ];
    model.forEach(function(m) {
        (limiters[m] || (limiters[m] = []))[channel] = modifier;
    });
    model = model[0];
    return function(val) {
        var result;
        if (arguments.length) {
            if (modifier) {
                val = modifier(val);
            }
            result = this[model]();
            result.color[channel] = val;
            return result;
        }
        result = this[model]().color[channel];
        if (modifier) {
            result = modifier(result);
        }
        return result;
    };
}
function maxfn(max) {
    return function(v) {
        return Math.max(0, Math.min(max, v));
    };
}
function assertArray(val) {
    return Array.isArray(val) ? val : [
        val
    ];
}
function zeroArray(arr, length) {
    for(var i = 0; i < length; i++){
        if (typeof arr[i] !== 'number') {
            arr[i] = 0;
        }
    }
    return arr;
}
module.exports = Color;

},
"4c0d486a": function(module, exports, farmRequire, farmDynamicRequire) {
'use strict';
Object.defineProperty(exports, "__esModule", {
    value: true
});
var loopable = function loopable(i, length) {
    var props = {};
    var setProp = function setProp(name) {
        var value = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : true;
        props[name] = value;
    };
    i === 0 && setProp('first-child');
    i === length - 1 && setProp('last-child');
    (i === 0 || i % 2 === 0) && setProp('even');
    Math.abs(i % 2) === 1 && setProp('odd');
    setProp('nth-child', i);
    return props;
};
exports.default = loopable;

},
"4c7d0c51": function(module, exports, farmRequire, farmDynamicRequire) {
var baseForOwn = farmRequire("219b7102", true), createBaseEach = farmRequire("13614e07", true);
/**
 * The base implementation of `_.forEach` without support for iteratee shorthands.
 *
 * @private
 * @param {Array|Object} collection The collection to iterate over.
 * @param {Function} iteratee The function invoked per iteration.
 * @returns {Array|Object} Returns `collection`.
 */ var baseEach = createBaseEach(baseForOwn);
module.exports = baseEach;

},
"4d74d49a": function(module, exports, farmRequire, farmDynamicRequire) {
var isObject = farmRequire("419b048f", true);
/**
 * Checks if `value` is suitable for strict equality comparisons, i.e. `===`.
 *
 * @private
 * @param {*} value The value to check.
 * @returns {boolean} Returns `true` if `value` if suitable for strict
 *  equality comparisons, else `false`.
 */ function isStrictComparable(value) {
    return value === value && !isObject(value);
}
module.exports = isStrictComparable;

},
"5b59c548": function(module, exports, farmRequire, farmDynamicRequire) {
'use strict';
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.flattenNames = undefined;
var _isString2 = farmRequire("28569ddf", true);
var _isString3 = _interopRequireDefault(_isString2);
var _forOwn2 = farmRequire("74cb39e4", true);
var _forOwn3 = _interopRequireDefault(_forOwn2);
var _isPlainObject2 = farmRequire("1ffd1d0c", true);
var _isPlainObject3 = _interopRequireDefault(_isPlainObject2);
var _map2 = farmRequire("0ede0ab4", true);
var _map3 = _interopRequireDefault(_map2);
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
var flattenNames = exports.flattenNames = function flattenNames() {
    var things = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : [];
    var names = [];
    (0, _map3.default)(things, function(thing) {
        if (Array.isArray(thing)) {
            flattenNames(thing).map(function(name) {
                return names.push(name);
            });
        } else if ((0, _isPlainObject3.default)(thing)) {
            (0, _forOwn3.default)(thing, function(value, key) {
                value === true && names.push(key);
                names.push(key + '-' + value);
            });
        } else if ((0, _isString3.default)(thing)) {
            names.push(thing);
        }
    });
    return names;
};
exports.default = flattenNames;

},
"64304e4f": function(module, exports, farmRequire, farmDynamicRequire) {
var arrayAggregator = farmRequire("84c4e179", true), baseAggregator = farmRequire("da8dec8d", true), baseIteratee = farmRequire("248e0b34", true), isArray = farmRequire("eff342ff", true);
/**
 * Creates a function like `_.groupBy`.
 *
 * @private
 * @param {Function} setter The function to set accumulator values.
 * @param {Function} [initializer] The accumulator object initializer.
 * @returns {Function} Returns the new aggregator function.
 */ function createAggregator(setter, initializer) {
    return function(collection, iteratee) {
        var func = isArray(collection) ? arrayAggregator : baseAggregator, accumulator = initializer ? initializer() : {};
        return func(collection, setter, baseIteratee(iteratee, 2), accumulator);
    };
}
module.exports = createAggregator;

},
"6959fd39": function(module, exports, farmRequire, farmDynamicRequire) {
var baseIsEqual = farmRequire("06efa494", true), get = farmRequire("d2f53123", true), hasIn = farmRequire("dea22812", true), isKey = farmRequire("82ab411e", true), isStrictComparable = farmRequire("4d74d49a", true), matchesStrictComparable = farmRequire("1e09a1df", true), toKey = farmRequire("12b97de7", true);
/** Used to compose bitmasks for value comparisons. */ var COMPARE_PARTIAL_FLAG = 1, COMPARE_UNORDERED_FLAG = 2;
/**
 * The base implementation of `_.matchesProperty` which doesn't clone `srcValue`.
 *
 * @private
 * @param {string} path The path of the property to get.
 * @param {*} srcValue The value to match.
 * @returns {Function} Returns the new spec function.
 */ function baseMatchesProperty(path, srcValue) {
    if (isKey(path) && isStrictComparable(srcValue)) {
        return matchesStrictComparable(toKey(path), srcValue);
    }
    return function(object) {
        var objValue = get(object, path);
        return objValue === undefined && objValue === srcValue ? hasIn(object, path) : baseIsEqual(srcValue, objValue, COMPARE_PARTIAL_FLAG | COMPARE_UNORDERED_FLAG);
    };
}
module.exports = baseMatchesProperty;

},
"74cb39e4": function(module, exports, farmRequire, farmDynamicRequire) {
var baseForOwn = farmRequire("219b7102", true), castFunction = farmRequire("7575f7fd", true);
/**
 * Iterates over own enumerable string keyed properties of an object and
 * invokes `iteratee` for each property. The iteratee is invoked with three
 * arguments: (value, key, object). Iteratee functions may exit iteration
 * early by explicitly returning `false`.
 *
 * @static
 * @memberOf _
 * @since 0.3.0
 * @category Object
 * @param {Object} object The object to iterate over.
 * @param {Function} [iteratee=_.identity] The function invoked per iteration.
 * @returns {Object} Returns `object`.
 * @see _.forOwnRight
 * @example
 *
 * function Foo() {
 *   this.a = 1;
 *   this.b = 2;
 * }
 *
 * Foo.prototype.c = 3;
 *
 * _.forOwn(new Foo, function(value, key) {
 *   console.log(key);
 * });
 * // => Logs 'a' then 'b' (iteration order is not guaranteed).
 */ function forOwn(object, iteratee) {
    return object && baseForOwn(object, castFunction(iteratee));
}
module.exports = forOwn;

},
"7575f7fd": function(module, exports, farmRequire, farmDynamicRequire) {
var identity = farmRequire("c6fbe0a8", true);
/**
 * Casts `value` to `identity` if it's not a function.
 *
 * @private
 * @param {*} value The value to inspect.
 * @returns {Function} Returns cast function.
 */ function castFunction(value) {
    return typeof value == 'function' ? value : identity;
}
module.exports = castFunction;

},
"80963b0b": function(module, exports, farmRequire, farmDynamicRequire) {
;
(function(root, factory) {
    if (typeof define === 'function' && define.amd) {
        define(factory);
    } else if (typeof exports === 'object') {
        module.exports = factory();
    } else {
        root.NProgress = factory();
    }
})(this, function() {
    var NProgress = {};
    NProgress.version = '0.2.0';
    var Settings = NProgress.settings = {
        minimum: 0.08,
        easing: 'ease',
        positionUsing: '',
        speed: 200,
        trickle: true,
        trickleRate: 0.02,
        trickleSpeed: 800,
        showSpinner: true,
        barSelector: '[role="bar"]',
        spinnerSelector: '[role="spinner"]',
        parent: 'body',
        template: '<div class="bar" role="bar"><div class="peg"></div></div><div class="spinner" role="spinner"><div class="spinner-icon"></div></div>'
    };
    /**
   * Updates configuration.
   *
   *     NProgress.configure({
   *       minimum: 0.1
   *     });
   */ NProgress.configure = function(options) {
        var key, value;
        for(key in options){
            value = options[key];
            if (value !== undefined && options.hasOwnProperty(key)) Settings[key] = value;
        }
        return this;
    };
    /**
   * Last number.
   */ NProgress.status = null;
    /**
   * Sets the progress bar status, where `n` is a number from `0.0` to `1.0`.
   *
   *     NProgress.set(0.4);
   *     NProgress.set(1.0);
   */ NProgress.set = function(n) {
        var started = NProgress.isStarted();
        n = clamp(n, Settings.minimum, 1);
        NProgress.status = n === 1 ? null : n;
        var progress = NProgress.render(!started), bar = progress.querySelector(Settings.barSelector), speed = Settings.speed, ease = Settings.easing;
        progress.offsetWidth; /* Repaint */ 
        queue(function(next) {
            // Set positionUsing if it hasn't already been set
            if (Settings.positionUsing === '') Settings.positionUsing = NProgress.getPositioningCSS();
            // Add transition
            css(bar, barPositionCSS(n, speed, ease));
            if (n === 1) {
                // Fade out
                css(progress, {
                    transition: 'none',
                    opacity: 1
                });
                progress.offsetWidth; /* Repaint */ 
                setTimeout(function() {
                    css(progress, {
                        transition: 'all ' + speed + 'ms linear',
                        opacity: 0
                    });
                    setTimeout(function() {
                        NProgress.remove();
                        next();
                    }, speed);
                }, speed);
            } else {
                setTimeout(next, speed);
            }
        });
        return this;
    };
    NProgress.isStarted = function() {
        return typeof NProgress.status === 'number';
    };
    /**
   * Shows the progress bar.
   * This is the same as setting the status to 0%, except that it doesn't go backwards.
   *
   *     NProgress.start();
   *
   */ NProgress.start = function() {
        if (!NProgress.status) NProgress.set(0);
        var work = function() {
            setTimeout(function() {
                if (!NProgress.status) return;
                NProgress.trickle();
                work();
            }, Settings.trickleSpeed);
        };
        if (Settings.trickle) work();
        return this;
    };
    /**
   * Hides the progress bar.
   * This is the *sort of* the same as setting the status to 100%, with the
   * difference being `done()` makes some placebo effect of some realistic motion.
   *
   *     NProgress.done();
   *
   * If `true` is passed, it will show the progress bar even if its hidden.
   *
   *     NProgress.done(true);
   */ NProgress.done = function(force) {
        if (!force && !NProgress.status) return this;
        return NProgress.inc(0.3 + 0.5 * Math.random()).set(1);
    };
    /**
   * Increments by a random amount.
   */ NProgress.inc = function(amount) {
        var n = NProgress.status;
        if (!n) {
            return NProgress.start();
        } else {
            if (typeof amount !== 'number') {
                amount = (1 - n) * clamp(Math.random() * n, 0.1, 0.95);
            }
            n = clamp(n + amount, 0, 0.994);
            return NProgress.set(n);
        }
    };
    NProgress.trickle = function() {
        return NProgress.inc(Math.random() * Settings.trickleRate);
    };
    /**
   * Waits for all supplied jQuery promises and
   * increases the progress as the promises resolve.
   *
   * @param $promise jQUery Promise
   */ (function() {
        var initial = 0, current = 0;
        NProgress.promise = function($promise) {
            if (!$promise || $promise.state() === "resolved") {
                return this;
            }
            if (current === 0) {
                NProgress.start();
            }
            initial++;
            current++;
            $promise.always(function() {
                current--;
                if (current === 0) {
                    initial = 0;
                    NProgress.done();
                } else {
                    NProgress.set((initial - current) / initial);
                }
            });
            return this;
        };
    })();
    /**
   * (Internal) renders the progress bar markup based on the `template`
   * setting.
   */ NProgress.render = function(fromStart) {
        if (NProgress.isRendered()) return document.getElementById('nprogress');
        addClass(document.documentElement, 'nprogress-busy');
        var progress = document.createElement('div');
        progress.id = 'nprogress';
        progress.innerHTML = Settings.template;
        var bar = progress.querySelector(Settings.barSelector), perc = fromStart ? '-100' : toBarPerc(NProgress.status || 0), parent = document.querySelector(Settings.parent), spinner;
        css(bar, {
            transition: 'all 0 linear',
            transform: 'translate3d(' + perc + '%,0,0)'
        });
        if (!Settings.showSpinner) {
            spinner = progress.querySelector(Settings.spinnerSelector);
            spinner && removeElement(spinner);
        }
        if (parent != document.body) {
            addClass(parent, 'nprogress-custom-parent');
        }
        parent.appendChild(progress);
        return progress;
    };
    /**
   * Removes the element. Opposite of render().
   */ NProgress.remove = function() {
        removeClass(document.documentElement, 'nprogress-busy');
        removeClass(document.querySelector(Settings.parent), 'nprogress-custom-parent');
        var progress = document.getElementById('nprogress');
        progress && removeElement(progress);
    };
    /**
   * Checks if the progress bar is rendered.
   */ NProgress.isRendered = function() {
        return !!document.getElementById('nprogress');
    };
    /**
   * Determine which positioning CSS rule to use.
   */ NProgress.getPositioningCSS = function() {
        // Sniff on document.body.style
        var bodyStyle = document.body.style;
        // Sniff prefixes
        var vendorPrefix = 'WebkitTransform' in bodyStyle ? 'Webkit' : 'MozTransform' in bodyStyle ? 'Moz' : 'msTransform' in bodyStyle ? 'ms' : 'OTransform' in bodyStyle ? 'O' : '';
        if (vendorPrefix + 'Perspective' in bodyStyle) {
            // Modern browsers with 3D support, e.g. Webkit, IE10
            return 'translate3d';
        } else if (vendorPrefix + 'Transform' in bodyStyle) {
            // Browsers without 3D support, e.g. IE9
            return 'translate';
        } else {
            // Browsers without translate() support, e.g. IE7-8
            return 'margin';
        }
    };
    /**
   * Helpers
   */ function clamp(n, min, max) {
        if (n < min) return min;
        if (n > max) return max;
        return n;
    }
    /**
   * (Internal) converts a percentage (`0..1`) to a bar translateX
   * percentage (`-100%..0%`).
   */ function toBarPerc(n) {
        return (-1 + n) * 100;
    }
    /**
   * (Internal) returns the correct CSS for changing the bar's
   * position given an n percentage, and speed and ease from Settings
   */ function barPositionCSS(n, speed, ease) {
        var barCSS;
        if (Settings.positionUsing === 'translate3d') {
            barCSS = {
                transform: 'translate3d(' + toBarPerc(n) + '%,0,0)'
            };
        } else if (Settings.positionUsing === 'translate') {
            barCSS = {
                transform: 'translate(' + toBarPerc(n) + '%,0)'
            };
        } else {
            barCSS = {
                'margin-left': toBarPerc(n) + '%'
            };
        }
        barCSS.transition = 'all ' + speed + 'ms ' + ease;
        return barCSS;
    }
    /**
   * (Internal) Queues a function to be executed.
   */ var queue = function() {
        var pending = [];
        function next() {
            var fn = pending.shift();
            if (fn) {
                fn(next);
            }
        }
        return function(fn) {
            pending.push(fn);
            if (pending.length == 1) next();
        };
    }();
    /**
   * (Internal) Applies css properties to an element, similar to the jQuery 
   * css method.
   *
   * While this helper does assist with vendor prefixed property names, it 
   * does not perform any manipulation of values prior to setting styles.
   */ var css = function() {
        var cssPrefixes = [
            'Webkit',
            'O',
            'Moz',
            'ms'
        ], cssProps = {};
        function camelCase(string) {
            return string.replace(/^-ms-/, 'ms-').replace(/-([\da-z])/gi, function(match, letter) {
                return letter.toUpperCase();
            });
        }
        function getVendorProp(name) {
            var style = document.body.style;
            if (name in style) return name;
            var i = cssPrefixes.length, capName = name.charAt(0).toUpperCase() + name.slice(1), vendorName;
            while(i--){
                vendorName = cssPrefixes[i] + capName;
                if (vendorName in style) return vendorName;
            }
            return name;
        }
        function getStyleProp(name) {
            name = camelCase(name);
            return cssProps[name] || (cssProps[name] = getVendorProp(name));
        }
        function applyCss(element, prop, value) {
            prop = getStyleProp(prop);
            element.style[prop] = value;
        }
        return function(element, properties) {
            var args = arguments, prop, value;
            if (args.length == 2) {
                for(prop in properties){
                    value = properties[prop];
                    if (value !== undefined && properties.hasOwnProperty(prop)) applyCss(element, prop, value);
                }
            } else {
                applyCss(element, args[1], args[2]);
            }
        };
    }();
    /**
   * (Internal) Determines if an element or space separated list of class names contains a class name.
   */ function hasClass(element, name) {
        var list = typeof element == 'string' ? element : classList(element);
        return list.indexOf(' ' + name + ' ') >= 0;
    }
    /**
   * (Internal) Adds a class to an element.
   */ function addClass(element, name) {
        var oldList = classList(element), newList = oldList + name;
        if (hasClass(oldList, name)) return;
        // Trim the opening space.
        element.className = newList.substring(1);
    }
    /**
   * (Internal) Removes a class from an element.
   */ function removeClass(element, name) {
        var oldList = classList(element), newList;
        if (!hasClass(element, name)) return;
        // Replace the class name.
        newList = oldList.replace(' ' + name + ' ', ' ');
        // Trim the opening and closing spaces.
        element.className = newList.substring(1, newList.length - 1);
    }
    /**
   * (Internal) Gets a space separated list of the class names on the element. 
   * The list is wrapped with a single space on each end to facilitate finding 
   * matches within the list.
   */ function classList(element) {
        return (' ' + (element.className || '') + ' ').replace(/\s+/gi, ' ');
    }
    /**
   * (Internal) Removes an element from the DOM.
   */ function removeElement(element) {
        element && element.parentNode && element.parentNode.removeChild(element);
    }
    return NProgress;
});

},
"84c4e179": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * A specialized version of `baseAggregator` for arrays.
 *
 * @private
 * @param {Array} [array] The array to iterate over.
 * @param {Function} setter The function to set `accumulator` values.
 * @param {Function} iteratee The iteratee to transform keys.
 * @param {Object} accumulator The initial aggregated object.
 * @returns {Function} Returns `accumulator`.
 */ function arrayAggregator(array, setter, iteratee, accumulator) {
    var index = -1, length = array == null ? 0 : array.length;
    while(++index < length){
        var value = array[index];
        setter(accumulator, value, iteratee(value), array);
    }
    return accumulator;
}
module.exports = arrayAggregator;

},
"87119331": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _objectWithoutPropertiesLoose = /*#__PURE__*/ _interop_require_default._(farmRequire("4fc31380"));
const _extends = /*#__PURE__*/ _interop_require_default._(farmRequire("857a4e6a"));
const _assertThisInitialized = /*#__PURE__*/ _interop_require_default._(farmRequire("c1f23455"));
const _inheritsLoose = /*#__PURE__*/ _interop_require_default._(farmRequire("45a778e2"));
const _reactis = farmRequire("949ddefe");
const _hoistnonreactstatics = /*#__PURE__*/ _interop_require_default._(farmRequire("34917ff4"));
/* eslint-disable import/prefer-default-export */ function invariant(condition, message) {
    if (condition) return;
    var error = new Error("loadable: " + message);
    error.framesToPop = 1;
    error.name = 'Invariant Violation';
    throw error;
}
var Context = /*#__PURE__*/ _react.default.createContext();
var LOADABLE_REQUIRED_CHUNKS_KEY = '__LOADABLE_REQUIRED_CHUNKS__';
function getRequiredChunkKey(namespace) {
    return "" + namespace + LOADABLE_REQUIRED_CHUNKS_KEY;
}
var sharedInternals = /*#__PURE__*/ Object.freeze({
    __proto__: null,
    getRequiredChunkKey: getRequiredChunkKey,
    invariant: invariant,
    Context: Context
});
var LOADABLE_SHARED = {
    initialChunks: {}
};
var STATUS_PENDING = 'PENDING';
var STATUS_RESOLVED = 'RESOLVED';
var STATUS_REJECTED = 'REJECTED';
function resolveConstructor(ctor) {
    if (typeof ctor === 'function') {
        return {
            requireAsync: ctor,
            resolve: function resolve() {
                return undefined;
            },
            chunkName: function chunkName() {
                return undefined;
            }
        };
    }
    return ctor;
}
var withChunkExtractor = function withChunkExtractor(Component) {
    var LoadableWithChunkExtractor = function LoadableWithChunkExtractor(props) {
        return _react.default.createElement(Context.Consumer, null, function(extractor) {
            return _react.default.createElement(Component, Object.assign({
                __chunkExtractor: extractor
            }, props));
        });
    };
    if (Component.displayName) {
        LoadableWithChunkExtractor.displayName = Component.displayName + "WithChunkExtractor";
    }
    return LoadableWithChunkExtractor;
};
var identity = function identity(v) {
    return v;
};
function createLoadable(_ref) {
    var _ref$defaultResolveCo = _ref.defaultResolveComponent, defaultResolveComponent = _ref$defaultResolveCo === void 0 ? identity : _ref$defaultResolveCo, _render = _ref.render, onLoad = _ref.onLoad;
    function loadable(loadableConstructor, options) {
        if (options === void 0) {
            options = {};
        }
        var ctor = resolveConstructor(loadableConstructor);
        var cache = {};
        /**
     * Cachekey represents the component to be loaded
     * if key changes - component has to be reloaded
     * @param props
     * @returns {null|Component}
     */ function _getCacheKey(props) {
            if (options.cacheKey) {
                return options.cacheKey(props);
            }
            if (ctor.resolve) {
                return ctor.resolve(props);
            }
            return 'static';
        }
        /**
     * Resolves loaded `module` to a specific `Component
     * @param module
     * @param props
     * @param Loadable
     * @returns Component
     */ function resolve(module, props, Loadable) {
            var Component = options.resolveComponent ? options.resolveComponent(module, props) : defaultResolveComponent(module);
            if (options.resolveComponent && !(0, _reactis.isValidElementType)(Component)) {
                throw new Error("resolveComponent returned something that is not a React component!");
            }
            (0, _hoistnonreactstatics.default)(Loadable, Component, {
                preload: true
            });
            return Component;
        }
        var cachedLoad = function cachedLoad(props) {
            var cacheKey = _getCacheKey(props);
            var promise = cache[cacheKey];
            if (!promise || promise.status === STATUS_REJECTED) {
                promise = ctor.requireAsync(props);
                promise.status = STATUS_PENDING;
                cache[cacheKey] = promise;
                promise.then(function() {
                    promise.status = STATUS_RESOLVED;
                }, function(error) {
                    console.error('loadable-components: failed to asynchronously load component', {
                        fileName: ctor.resolve(props),
                        chunkName: ctor.chunkName(props),
                        error: error ? error.message : error
                    });
                    promise.status = STATUS_REJECTED;
                });
            }
            return promise;
        };
        var InnerLoadable = /*#__PURE__*/ function(_React$Component) {
            (0, _inheritsLoose.default)(InnerLoadable, _React$Component);
            InnerLoadable.getDerivedStateFromProps = function getDerivedStateFromProps(props, state) {
                var cacheKey = _getCacheKey(props);
                return (0, _extends.default)({}, state, {
                    cacheKey: cacheKey,
                    // change of a key triggers loading state automatically
                    loading: state.loading || state.cacheKey !== cacheKey
                });
            };
            function InnerLoadable(props) {
                var _this;
                _this = _React$Component.call(this, props) || this;
                _this.state = {
                    result: null,
                    error: null,
                    loading: true,
                    cacheKey: _getCacheKey(props)
                };
                invariant(!props.__chunkExtractor || ctor.requireSync, 'SSR requires `@loadable/babel-plugin`, please install it'); // Server-side
                if (props.__chunkExtractor) {
                    // This module has been marked with no SSR
                    if (options.ssr === false) {
                        return (0, _assertThisInitialized.default)(_this);
                    } // We run load function, we assume that it won't fail and that it
                    // triggers a synchronous loading of the module
                    ctor.requireAsync(props)["catch"](function() {
                        return null;
                    }); // So we can require now the module synchronously
                    _this.loadSync();
                    props.__chunkExtractor.addChunk(ctor.chunkName(props));
                    return (0, _assertThisInitialized.default)(_this);
                } // Client-side with `isReady` method present (SSR probably)
                // If module is already loaded, we use a synchronous loading
                // Only perform this synchronous loading if the component has not
                // been marked with no SSR, else we risk hydration mismatches
                if (options.ssr !== false && (ctor.isReady && ctor.isReady(props) || // is ready - was loaded during SSR process
                ctor.chunkName && LOADABLE_SHARED.initialChunks[ctor.chunkName(props)])) {
                    _this.loadSync();
                }
                return _this;
            }
            var _proto = InnerLoadable.prototype;
            _proto.componentDidMount = function componentDidMount() {
                this.mounted = true; // retrieve loading promise from a global cache
                var cachedPromise = this.getCache(); // if promise exists, but rejected - clear cache
                if (cachedPromise && cachedPromise.status === STATUS_REJECTED) {
                    this.setCache();
                } // component might be resolved synchronously in the constructor
                if (this.state.loading) {
                    this.loadAsync();
                }
            };
            _proto.componentDidUpdate = function componentDidUpdate(prevProps, prevState) {
                // Component has to be reloaded on cacheKey change
                if (prevState.cacheKey !== this.state.cacheKey) {
                    this.loadAsync();
                }
            };
            _proto.componentWillUnmount = function componentWillUnmount() {
                this.mounted = false;
            };
            _proto.safeSetState = function safeSetState(nextState, callback) {
                if (this.mounted) {
                    this.setState(nextState, callback);
                }
            } /**
       * returns a cache key for the current props
       * @returns {Component|string}
       */ ;
            _proto.getCacheKey = function getCacheKey() {
                return _getCacheKey(this.props);
            } /**
       * access the persistent cache
       */ ;
            _proto.getCache = function getCache() {
                return cache[this.getCacheKey()];
            } /**
       * sets the cache value. If called without value sets it as undefined
       */ ;
            _proto.setCache = function setCache(value) {
                if (value === void 0) {
                    value = undefined;
                }
                cache[this.getCacheKey()] = value;
            };
            _proto.triggerOnLoad = function triggerOnLoad() {
                var _this2 = this;
                if (onLoad) {
                    setTimeout(function() {
                        onLoad(_this2.state.result, _this2.props);
                    });
                }
            } /**
       * Synchronously loads component
       * target module is expected to already exists in the module cache
       * or be capable to resolve synchronously (webpack target=node)
       */ ;
            _proto.loadSync = function loadSync() {
                // load sync is expecting component to be in the "loading" state already
                // sounds weird, but loading=true is the initial state of InnerLoadable
                if (!this.state.loading) return;
                try {
                    var loadedModule = ctor.requireSync(this.props);
                    var result = resolve(loadedModule, this.props, Loadable);
                    this.state.result = result;
                    this.state.loading = false;
                } catch (error) {
                    console.error('loadable-components: failed to synchronously load component, which expected to be available', {
                        fileName: ctor.resolve(this.props),
                        chunkName: ctor.chunkName(this.props),
                        error: error ? error.message : error
                    });
                    this.state.error = error;
                }
            } /**
       * Asynchronously loads a component.
       */ ;
            _proto.loadAsync = function loadAsync() {
                var _this3 = this;
                var promise = this.resolveAsync();
                promise.then(function(loadedModule) {
                    var result = resolve(loadedModule, _this3.props, Loadable);
                    _this3.safeSetState({
                        result: result,
                        loading: false
                    }, function() {
                        return _this3.triggerOnLoad();
                    });
                })["catch"](function(error) {
                    return _this3.safeSetState({
                        error: error,
                        loading: false
                    });
                });
                return promise;
            } /**
       * Asynchronously resolves(not loads) a component.
       * Note - this function does not change the state
       */ ;
            _proto.resolveAsync = function resolveAsync() {
                var _this$props = this.props, __chunkExtractor = _this$props.__chunkExtractor, forwardedRef = _this$props.forwardedRef, props = (0, _objectWithoutPropertiesLoose.default)(_this$props, [
                    "__chunkExtractor",
                    "forwardedRef"
                ]);
                return cachedLoad(props);
            };
            _proto.render = function render() {
                var _this$props2 = this.props, forwardedRef = _this$props2.forwardedRef, propFallback = _this$props2.fallback, __chunkExtractor = _this$props2.__chunkExtractor, props = (0, _objectWithoutPropertiesLoose.default)(_this$props2, [
                    "forwardedRef",
                    "fallback",
                    "__chunkExtractor"
                ]);
                var _this$state = this.state, error = _this$state.error, loading = _this$state.loading, result = _this$state.result;
                if (options.suspense) {
                    var cachedPromise = this.getCache() || this.loadAsync();
                    if (cachedPromise.status === STATUS_PENDING) {
                        throw this.loadAsync();
                    }
                }
                if (error) {
                    throw error;
                }
                var fallback = propFallback || options.fallback || null;
                if (loading) {
                    return fallback;
                }
                return _render({
                    fallback: fallback,
                    result: result,
                    options: options,
                    props: (0, _extends.default)({}, props, {
                        ref: forwardedRef
                    })
                });
            };
            return InnerLoadable;
        }(_react.default.Component);
        var EnhancedInnerLoadable = withChunkExtractor(InnerLoadable);
        var Loadable = _react.default.forwardRef(function(props, ref) {
            return _react.default.createElement(EnhancedInnerLoadable, Object.assign({
                forwardedRef: ref
            }, props));
        });
        Loadable.displayName = 'Loadable'; // In future, preload could use `<link rel="preload">`
        Loadable.preload = function(props) {
            Loadable.load(props);
        };
        Loadable.load = function(props) {
            return cachedLoad(props);
        };
        return Loadable;
    }
    function lazy(ctor, options) {
        return loadable(ctor, (0, _extends.default)({}, options, {
            suspense: true
        }));
    }
    return {
        loadable: loadable,
        lazy: lazy
    };
}
function defaultResolveComponent(loadedModule) {
    // eslint-disable-next-line no-underscore-dangle
    return loadedModule.__esModule ? loadedModule["default"] : loadedModule["default"] || loadedModule;
}
/* eslint-disable no-use-before-define, react/no-multi-comp */ var _createLoadable = /*#__PURE__*/ createLoadable({
    defaultResolveComponent: defaultResolveComponent,
    render: function render(_ref) {
        var Component = _ref.result, props = _ref.props;
        return _react.default.createElement(Component, props);
    }
}), loadable = _createLoadable.loadable, lazy = _createLoadable.lazy;
/* eslint-disable no-use-before-define, react/no-multi-comp */ var _createLoadable$1 = /*#__PURE__*/ createLoadable({
    onLoad: function onLoad(result, props) {
        if (result && props.forwardedRef) {
            if (typeof props.forwardedRef === 'function') {
                props.forwardedRef(result);
            } else {
                props.forwardedRef.current = result;
            }
        }
    },
    render: function render(_ref) {
        var result = _ref.result, props = _ref.props;
        if (props.children) {
            return props.children(result);
        }
        return null;
    }
}), loadable$1 = _createLoadable$1.loadable, lazy$1 = _createLoadable$1.lazy;
/* eslint-disable no-underscore-dangle, camelcase */ var BROWSER = typeof window !== 'undefined';
/* eslint-disable no-underscore-dangle */ var loadable$2 = loadable;
loadable$2.lib = loadable$1;
const _default = loadable$2;

},
"88545e9c": function(module, exports, farmRequire, farmDynamicRequire) {
/**
 * The base implementation of `_.property` without support for deep paths.
 *
 * @private
 * @param {string} key The key of the property to get.
 * @returns {Function} Returns the new accessor function.
 */ function baseProperty(key) {
    return function(object) {
        return object == null ? undefined : object[key];
    };
}
module.exports = baseProperty;

},
"896f638f": function(module, exports, farmRequire, farmDynamicRequire) {
'use strict';
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.hover = undefined;
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
var _react = farmRequire("a0fc9dfd", true);
var _react2 = _interopRequireDefault(_react);
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
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
var hover = exports.hover = function hover(Component) {
    var Span = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : 'span';
    return function(_React$Component) {
        _inherits(Hover, _React$Component);
        function Hover() {
            var _ref;
            var _temp, _this, _ret;
            _classCallCheck(this, Hover);
            for(var _len = arguments.length, args = Array(_len), _key = 0; _key < _len; _key++){
                args[_key] = arguments[_key];
            }
            return _ret = (_temp = (_this = _possibleConstructorReturn(this, (_ref = Hover.__proto__ || Object.getPrototypeOf(Hover)).call.apply(_ref, [
                this
            ].concat(args))), _this), _this.state = {
                hover: false
            }, _this.handleMouseOver = function() {
                return _this.setState({
                    hover: true
                });
            }, _this.handleMouseOut = function() {
                return _this.setState({
                    hover: false
                });
            }, _this.render = function() {
                return _react2.default.createElement(Span, {
                    onMouseOver: _this.handleMouseOver,
                    onMouseOut: _this.handleMouseOut
                }, _react2.default.createElement(Component, _extends({}, _this.props, _this.state)));
            }, _temp), _possibleConstructorReturn(_this, _ret);
        }
        return Hover;
    }(_react2.default.Component);
};
exports.default = hover;

},
"898d9be6": function(module, exports, farmRequire, farmDynamicRequire) {
'use strict';
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.active = undefined;
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
var _react = farmRequire("a0fc9dfd", true);
var _react2 = _interopRequireDefault(_react);
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
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
var active = exports.active = function active(Component) {
    var Span = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : 'span';
    return function(_React$Component) {
        _inherits(Active, _React$Component);
        function Active() {
            var _ref;
            var _temp, _this, _ret;
            _classCallCheck(this, Active);
            for(var _len = arguments.length, args = Array(_len), _key = 0; _key < _len; _key++){
                args[_key] = arguments[_key];
            }
            return _ret = (_temp = (_this = _possibleConstructorReturn(this, (_ref = Active.__proto__ || Object.getPrototypeOf(Active)).call.apply(_ref, [
                this
            ].concat(args))), _this), _this.state = {
                active: false
            }, _this.handleMouseDown = function() {
                return _this.setState({
                    active: true
                });
            }, _this.handleMouseUp = function() {
                return _this.setState({
                    active: false
                });
            }, _this.render = function() {
                return _react2.default.createElement(Span, {
                    onMouseDown: _this.handleMouseDown,
                    onMouseUp: _this.handleMouseUp
                }, _react2.default.createElement(Component, _extends({}, _this.props, _this.state)));
            }, _temp), _possibleConstructorReturn(_this, _ret);
        }
        return Active;
    }(_react2.default.Component);
};
exports.default = active;

},
"9509615b": function(module, exports, farmRequire, farmDynamicRequire) {
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
    Redirect: function() {
        return Redirect;
    },
    Route: function() {
        return Route;
    },
    Router: function() {
        return Router;
    },
    Switch: function() {
        return Switch;
    },
    __RouterContext: function() {
        return context;
    },
    matchPath: function() {
        return matchPath;
    },
    useHistory: function() {
        return useHistory;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _inheritsLoose = /*#__PURE__*/ _interop_require_default._(farmRequire("45a778e2"));
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _proptypes = /*#__PURE__*/ _interop_require_default._(farmRequire("75a1a40c"));
const _history = farmRequire("e57f3bb0");
const _tinywarning = /*#__PURE__*/ _interop_require_default._(farmRequire("386b0d68"));
const _tinyinvariant = /*#__PURE__*/ _interop_require_default._(farmRequire("8d305800"));
const _extends = /*#__PURE__*/ _interop_require_default._(farmRequire("857a4e6a"));
const _pathtoregexp = /*#__PURE__*/ _interop_require_default._(farmRequire("d4532a2a"));
const _reactis = farmRequire("949ddefe");
const _objectWithoutPropertiesLoose = /*#__PURE__*/ _interop_require_default._(farmRequire("4fc31380"));
var MAX_SIGNED_31_BIT_INT = 1073741823;
var commonjsGlobal = typeof globalThis !== "undefined" // 'global proper'
 ? globalThis : typeof window !== "undefined" ? window // Browser
 : typeof global !== "undefined" ? global // node.js
 : {};
function getUniqueId() {
    var key = "__global_unique_id__";
    return commonjsGlobal[key] = (commonjsGlobal[key] || 0) + 1;
} // Inlined Object.is polyfill.
// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/is
function objectIs(x, y) {
    if (x === y) {
        return x !== 0 || 1 / x === 1 / y;
    } else {
        // eslint-disable-next-line no-self-compare
        return x !== x && y !== y;
    }
}
function createEventEmitter(value) {
    var handlers = [];
    return {
        on: function on(handler) {
            handlers.push(handler);
        },
        off: function off(handler) {
            handlers = handlers.filter(function(h) {
                return h !== handler;
            });
        },
        get: function get() {
            return value;
        },
        set: function set(newValue, changedBits) {
            value = newValue;
            handlers.forEach(function(handler) {
                return handler(value, changedBits);
            });
        }
    };
}
function onlyChild(children) {
    return Array.isArray(children) ? children[0] : children;
}
function createReactContext(defaultValue, calculateChangedBits) {
    var _Provider$childContex, _Consumer$contextType;
    var contextProp = "__create-react-context-" + getUniqueId() + "__";
    var Provider = /*#__PURE__*/ function(_React$Component) {
        (0, _inheritsLoose.default)(Provider, _React$Component);
        function Provider() {
            var _this;
            for(var _len = arguments.length, args = new Array(_len), _key = 0; _key < _len; _key++){
                args[_key] = arguments[_key];
            }
            _this = _React$Component.call.apply(_React$Component, [
                this
            ].concat(args)) || this;
            _this.emitter = createEventEmitter(_this.props.value);
            return _this;
        }
        var _proto = Provider.prototype;
        _proto.getChildContext = function getChildContext() {
            var _ref;
            return _ref = {}, _ref[contextProp] = this.emitter, _ref;
        };
        _proto.componentWillReceiveProps = function componentWillReceiveProps(nextProps) {
            if (this.props.value !== nextProps.value) {
                var oldValue = this.props.value;
                var newValue = nextProps.value;
                var changedBits;
                if (objectIs(oldValue, newValue)) {
                    changedBits = 0; // No change
                } else {
                    changedBits = typeof calculateChangedBits === "function" ? calculateChangedBits(oldValue, newValue) : MAX_SIGNED_31_BIT_INT;
                    if ("production" !== "production") {
                        "production" !== "production" ? (0, _tinywarning.default)((changedBits & MAX_SIGNED_31_BIT_INT) === changedBits, "calculateChangedBits: Expected the return value to be a " + "31-bit integer. Instead received: " + changedBits) : void 0;
                    }
                    changedBits |= 0;
                    if (changedBits !== 0) {
                        this.emitter.set(nextProps.value, changedBits);
                    }
                }
            }
        };
        _proto.render = function render() {
            return this.props.children;
        };
        return Provider;
    }(_react.default.Component);
    Provider.childContextTypes = (_Provider$childContex = {}, _Provider$childContex[contextProp] = _proptypes.default.object.isRequired, _Provider$childContex);
    var Consumer = /*#__PURE__*/ function(_React$Component2) {
        (0, _inheritsLoose.default)(Consumer, _React$Component2);
        function Consumer() {
            var _this2;
            for(var _len2 = arguments.length, args = new Array(_len2), _key2 = 0; _key2 < _len2; _key2++){
                args[_key2] = arguments[_key2];
            }
            _this2 = _React$Component2.call.apply(_React$Component2, [
                this
            ].concat(args)) || this;
            _this2.observedBits = void 0;
            _this2.state = {
                value: _this2.getValue()
            };
            _this2.onUpdate = function(newValue, changedBits) {
                var observedBits = _this2.observedBits | 0;
                if ((observedBits & changedBits) !== 0) {
                    _this2.setState({
                        value: _this2.getValue()
                    });
                }
            };
            return _this2;
        }
        var _proto2 = Consumer.prototype;
        _proto2.componentWillReceiveProps = function componentWillReceiveProps(nextProps) {
            var observedBits = nextProps.observedBits;
            this.observedBits = observedBits === undefined || observedBits === null ? MAX_SIGNED_31_BIT_INT // Subscribe to all changes by default
             : observedBits;
        };
        _proto2.componentDidMount = function componentDidMount() {
            if (this.context[contextProp]) {
                this.context[contextProp].on(this.onUpdate);
            }
            var observedBits = this.props.observedBits;
            this.observedBits = observedBits === undefined || observedBits === null ? MAX_SIGNED_31_BIT_INT // Subscribe to all changes by default
             : observedBits;
        };
        _proto2.componentWillUnmount = function componentWillUnmount() {
            if (this.context[contextProp]) {
                this.context[contextProp].off(this.onUpdate);
            }
        };
        _proto2.getValue = function getValue() {
            if (this.context[contextProp]) {
                return this.context[contextProp].get();
            } else {
                return defaultValue;
            }
        };
        _proto2.render = function render() {
            return onlyChild(this.props.children)(this.state.value);
        };
        return Consumer;
    }(_react.default.Component);
    Consumer.contextTypes = (_Consumer$contextType = {}, _Consumer$contextType[contextProp] = _proptypes.default.object, _Consumer$contextType);
    return {
        Provider: Provider,
        Consumer: Consumer
    };
}
// MIT License
var createContext = _react.default.createContext || createReactContext;
// TODO: Replace with React.createContext once we can assume React 16+
var createNamedContext = function createNamedContext(name) {
    var context = createContext();
    context.displayName = name;
    return context;
};
var historyContext = /*#__PURE__*/ createNamedContext("Router-History");
var context = /*#__PURE__*/ createNamedContext("Router");
/**
 * The public API for putting history on context.
 */ var Router = /*#__PURE__*/ function(_React$Component) {
    (0, _inheritsLoose.default)(Router, _React$Component);
    Router.computeRootMatch = function computeRootMatch(pathname) {
        return {
            path: "/",
            url: "/",
            params: {},
            isExact: pathname === "/"
        };
    };
    function Router(props) {
        var _this;
        _this = _React$Component.call(this, props) || this;
        _this.state = {
            location: props.history.location
        }; // This is a bit of a hack. We have to start listening for location
        // changes here in the constructor in case there are any <Redirect>s
        // on the initial render. If there are, they will replace/push when
        // they mount and since cDM fires in children before parents, we may
        // get a new location before the <Router> is mounted.
        _this._isMounted = false;
        _this._pendingLocation = null;
        if (!props.staticContext) {
            _this.unlisten = props.history.listen(function(location) {
                _this._pendingLocation = location;
            });
        }
        return _this;
    }
    var _proto = Router.prototype;
    _proto.componentDidMount = function componentDidMount() {
        var _this2 = this;
        this._isMounted = true;
        if (this.unlisten) {
            // Any pre-mount location changes have been captured at
            // this point, so unregister the listener.
            this.unlisten();
        }
        if (!this.props.staticContext) {
            this.unlisten = this.props.history.listen(function(location) {
                if (_this2._isMounted) {
                    _this2.setState({
                        location: location
                    });
                }
            });
        }
        if (this._pendingLocation) {
            this.setState({
                location: this._pendingLocation
            });
        }
    };
    _proto.componentWillUnmount = function componentWillUnmount() {
        if (this.unlisten) {
            this.unlisten();
            this._isMounted = false;
            this._pendingLocation = null;
        }
    };
    _proto.render = function render() {
        return /*#__PURE__*/ _react.default.createElement(context.Provider, {
            value: {
                history: this.props.history,
                location: this.state.location,
                match: Router.computeRootMatch(this.state.location.pathname),
                staticContext: this.props.staticContext
            }
        }, /*#__PURE__*/ _react.default.createElement(historyContext.Provider, {
            children: this.props.children || null,
            value: this.props.history
        }));
    };
    return Router;
}(_react.default.Component);
if ("production" !== "production") {
    Router.propTypes = {
        children: _proptypes.default.node,
        history: _proptypes.default.object.isRequired,
        staticContext: _proptypes.default.object
    };
    Router.prototype.componentDidUpdate = function(prevProps) {
        "production" !== "production" ? (0, _tinywarning.default)(prevProps.history === this.props.history, "You cannot change <Router history>") : void 0;
    };
}
/**
 * The public API for a <Router> that stores location in memory.
 */ var MemoryRouter = /*#__PURE__*/ function(_React$Component) {
    (0, _inheritsLoose.default)(MemoryRouter, _React$Component);
    function MemoryRouter() {
        var _this;
        for(var _len = arguments.length, args = new Array(_len), _key = 0; _key < _len; _key++){
            args[_key] = arguments[_key];
        }
        _this = _React$Component.call.apply(_React$Component, [
            this
        ].concat(args)) || this;
        _this.history = (0, _history.createMemoryHistory)(_this.props);
        return _this;
    }
    var _proto = MemoryRouter.prototype;
    _proto.render = function render() {
        return /*#__PURE__*/ _react.default.createElement(Router, {
            history: this.history,
            children: this.props.children
        });
    };
    return MemoryRouter;
}(_react.default.Component);
if ("production" !== "production") {
    MemoryRouter.propTypes = {
        initialEntries: _proptypes.default.array,
        initialIndex: _proptypes.default.number,
        getUserConfirmation: _proptypes.default.func,
        keyLength: _proptypes.default.number,
        children: _proptypes.default.node
    };
    MemoryRouter.prototype.componentDidMount = function() {
        "production" !== "production" ? (0, _tinywarning.default)(!this.props.history, "<MemoryRouter> ignores the history prop. To use a custom history, " + "use `import { Router }` instead of `import { MemoryRouter as Router }`.") : void 0;
    };
}
var Lifecycle = /*#__PURE__*/ function(_React$Component) {
    (0, _inheritsLoose.default)(Lifecycle, _React$Component);
    function Lifecycle() {
        return _React$Component.apply(this, arguments) || this;
    }
    var _proto = Lifecycle.prototype;
    _proto.componentDidMount = function componentDidMount() {
        if (this.props.onMount) this.props.onMount.call(this, this);
    };
    _proto.componentDidUpdate = function componentDidUpdate(prevProps) {
        if (this.props.onUpdate) this.props.onUpdate.call(this, this, prevProps);
    };
    _proto.componentWillUnmount = function componentWillUnmount() {
        if (this.props.onUnmount) this.props.onUnmount.call(this, this);
    };
    _proto.render = function render() {
        return null;
    };
    return Lifecycle;
}(_react.default.Component);
/**
 * The public API for prompting the user before navigating away from a screen.
 */ function Prompt(_ref) {
    var message = _ref.message, _ref$when = _ref.when, when = _ref$when === void 0 ? true : _ref$when;
    return /*#__PURE__*/ _react.default.createElement(context.Consumer, null, function(context) {
        !context ? "production" !== "production" ? (0, _tinyinvariant.default)(false, "You should not use <Prompt> outside a <Router>") : (0, _tinyinvariant.default)(false) : void 0;
        if (!when || context.staticContext) return null;
        var method = context.history.block;
        return /*#__PURE__*/ _react.default.createElement(Lifecycle, {
            onMount: function onMount(self) {
                self.release = method(message);
            },
            onUpdate: function onUpdate(self, prevProps) {
                if (prevProps.message !== message) {
                    self.release();
                    self.release = method(message);
                }
            },
            onUnmount: function onUnmount(self) {
                self.release();
            },
            message: message
        });
    });
}
if ("production" !== "production") {
    var messageType = _proptypes.default.oneOfType([
        _proptypes.default.func,
        _proptypes.default.string
    ]);
    Prompt.propTypes = {
        when: _proptypes.default.bool,
        message: messageType.isRequired
    };
}
var cache = {};
var cacheLimit = 10000;
var cacheCount = 0;
function compilePath(path) {
    if (cache[path]) return cache[path];
    var generator = _pathtoregexp.default.compile(path);
    if (cacheCount < cacheLimit) {
        cache[path] = generator;
        cacheCount++;
    }
    return generator;
}
/**
 * Public API for generating a URL pathname from a path and parameters.
 */ function generatePath(path, params) {
    if (path === void 0) {
        path = "/";
    }
    if (params === void 0) {
        params = {};
    }
    return path === "/" ? path : compilePath(path)(params, {
        pretty: true
    });
}
/**
 * The public API for navigating programmatically with a component.
 */ function Redirect(_ref) {
    var computedMatch = _ref.computedMatch, to = _ref.to, _ref$push = _ref.push, push = _ref$push === void 0 ? false : _ref$push;
    return /*#__PURE__*/ _react.default.createElement(context.Consumer, null, function(context) {
        !context ? "production" !== "production" ? (0, _tinyinvariant.default)(false, "You should not use <Redirect> outside a <Router>") : (0, _tinyinvariant.default)(false) : void 0;
        var history = context.history, staticContext = context.staticContext;
        var method = push ? history.push : history.replace;
        var location = (0, _history.createLocation)(computedMatch ? typeof to === "string" ? generatePath(to, computedMatch.params) : (0, _extends.default)({}, to, {
            pathname: generatePath(to.pathname, computedMatch.params)
        }) : to); // When rendering in a static context,
        // set the new location immediately.
        if (staticContext) {
            method(location);
            return null;
        }
        return /*#__PURE__*/ _react.default.createElement(Lifecycle, {
            onMount: function onMount() {
                method(location);
            },
            onUpdate: function onUpdate(self, prevProps) {
                var prevLocation = (0, _history.createLocation)(prevProps.to);
                if (!(0, _history.locationsAreEqual)(prevLocation, (0, _extends.default)({}, location, {
                    key: prevLocation.key
                }))) {
                    method(location);
                }
            },
            to: to
        });
    });
}
if ("production" !== "production") {
    Redirect.propTypes = {
        push: _proptypes.default.bool,
        from: _proptypes.default.string,
        to: _proptypes.default.oneOfType([
            _proptypes.default.string,
            _proptypes.default.object
        ]).isRequired
    };
}
var cache$1 = {};
var cacheLimit$1 = 10000;
var cacheCount$1 = 0;
function compilePath$1(path, options) {
    var cacheKey = "" + options.end + options.strict + options.sensitive;
    var pathCache = cache$1[cacheKey] || (cache$1[cacheKey] = {});
    if (pathCache[path]) return pathCache[path];
    var keys = [];
    var regexp = (0, _pathtoregexp.default)(path, keys, options);
    var result = {
        regexp: regexp,
        keys: keys
    };
    if (cacheCount$1 < cacheLimit$1) {
        pathCache[path] = result;
        cacheCount$1++;
    }
    return result;
}
/**
 * Public API for matching a URL pathname to a path.
 */ function matchPath(pathname, options) {
    if (options === void 0) {
        options = {};
    }
    if (typeof options === "string" || Array.isArray(options)) {
        options = {
            path: options
        };
    }
    var _options = options, path = _options.path, _options$exact = _options.exact, exact = _options$exact === void 0 ? false : _options$exact, _options$strict = _options.strict, strict = _options$strict === void 0 ? false : _options$strict, _options$sensitive = _options.sensitive, sensitive = _options$sensitive === void 0 ? false : _options$sensitive;
    var paths = [].concat(path);
    return paths.reduce(function(matched, path) {
        if (!path && path !== "") return null;
        if (matched) return matched;
        var _compilePath = compilePath$1(path, {
            end: exact,
            strict: strict,
            sensitive: sensitive
        }), regexp = _compilePath.regexp, keys = _compilePath.keys;
        var match = regexp.exec(pathname);
        if (!match) return null;
        var url = match[0], values = match.slice(1);
        var isExact = pathname === url;
        if (exact && !isExact) return null;
        return {
            path: path,
            // the path used to match
            url: path === "/" && url === "" ? "/" : url,
            // the matched portion of the URL
            isExact: isExact,
            // whether or not we matched exactly
            params: keys.reduce(function(memo, key, index) {
                memo[key.name] = values[index];
                return memo;
            }, {})
        };
    }, null);
}
function isEmptyChildren(children) {
    return _react.default.Children.count(children) === 0;
}
function evalChildrenDev(children, props, path) {
    var value = children(props);
    "production" !== "production" ? (0, _tinywarning.default)(value !== undefined, "You returned `undefined` from the `children` function of " + ("<Route" + (path ? " path=\"" + path + "\"" : "") + ">, but you ") + "should have returned a React element or `null`") : void 0;
    return value || null;
}
/**
 * The public API for matching a single path and rendering.
 */ var Route = /*#__PURE__*/ function(_React$Component) {
    (0, _inheritsLoose.default)(Route, _React$Component);
    function Route() {
        return _React$Component.apply(this, arguments) || this;
    }
    var _proto = Route.prototype;
    _proto.render = function render() {
        var _this = this;
        return /*#__PURE__*/ _react.default.createElement(context.Consumer, null, function(context$1) {
            !context$1 ? "production" !== "production" ? (0, _tinyinvariant.default)(false, "You should not use <Route> outside a <Router>") : (0, _tinyinvariant.default)(false) : void 0;
            var location = _this.props.location || context$1.location;
            var match = _this.props.computedMatch ? _this.props.computedMatch // <Switch> already computed the match for us
             : _this.props.path ? matchPath(location.pathname, _this.props) : context$1.match;
            var props = (0, _extends.default)({}, context$1, {
                location: location,
                match: match
            });
            var _this$props = _this.props, children = _this$props.children, component = _this$props.component, render = _this$props.render; // Preact uses an empty array as children by
            // default, so use null if that's the case.
            if (Array.isArray(children) && isEmptyChildren(children)) {
                children = null;
            }
            return /*#__PURE__*/ _react.default.createElement(context.Provider, {
                value: props
            }, props.match ? children ? typeof children === "function" ? "production" !== "production" ? evalChildrenDev(children, props, _this.props.path) : children(props) : children : component ? /*#__PURE__*/ _react.default.createElement(component, props) : render ? render(props) : null : typeof children === "function" ? "production" !== "production" ? evalChildrenDev(children, props, _this.props.path) : children(props) : null);
        });
    };
    return Route;
}(_react.default.Component);
if ("production" !== "production") {
    Route.propTypes = {
        children: _proptypes.default.oneOfType([
            _proptypes.default.func,
            _proptypes.default.node
        ]),
        component: function component(props, propName) {
            if (props[propName] && !(0, _reactis.isValidElementType)(props[propName])) {
                return new Error("Invalid prop 'component' supplied to 'Route': the prop is not a valid React component");
            }
        },
        exact: _proptypes.default.bool,
        location: _proptypes.default.object,
        path: _proptypes.default.oneOfType([
            _proptypes.default.string,
            _proptypes.default.arrayOf(_proptypes.default.string)
        ]),
        render: _proptypes.default.func,
        sensitive: _proptypes.default.bool,
        strict: _proptypes.default.bool
    };
    Route.prototype.componentDidMount = function() {
        "production" !== "production" ? (0, _tinywarning.default)(!(this.props.children && !isEmptyChildren(this.props.children) && this.props.component), "You should not use <Route component> and <Route children> in the same route; <Route component> will be ignored") : void 0;
        "production" !== "production" ? (0, _tinywarning.default)(!(this.props.children && !isEmptyChildren(this.props.children) && this.props.render), "You should not use <Route render> and <Route children> in the same route; <Route render> will be ignored") : void 0;
        "production" !== "production" ? (0, _tinywarning.default)(!(this.props.component && this.props.render), "You should not use <Route component> and <Route render> in the same route; <Route render> will be ignored") : void 0;
    };
    Route.prototype.componentDidUpdate = function(prevProps) {
        "production" !== "production" ? (0, _tinywarning.default)(!(this.props.location && !prevProps.location), '<Route> elements should not change from uncontrolled to controlled (or vice versa). You initially used no "location" prop and then provided one on a subsequent render.') : void 0;
        "production" !== "production" ? (0, _tinywarning.default)(!(!this.props.location && prevProps.location), '<Route> elements should not change from controlled to uncontrolled (or vice versa). You provided a "location" prop initially but omitted it on a subsequent render.') : void 0;
    };
}
function addLeadingSlash(path) {
    return path.charAt(0) === "/" ? path : "/" + path;
}
function addBasename(basename, location) {
    if (!basename) return location;
    return (0, _extends.default)({}, location, {
        pathname: addLeadingSlash(basename) + location.pathname
    });
}
function stripBasename(basename, location) {
    if (!basename) return location;
    var base = addLeadingSlash(basename);
    if (location.pathname.indexOf(base) !== 0) return location;
    return (0, _extends.default)({}, location, {
        pathname: location.pathname.substr(base.length)
    });
}
function createURL(location) {
    return typeof location === "string" ? location : (0, _history.createPath)(location);
}
function staticHandler(methodName) {
    return function() {
        "production" !== "production" ? (0, _tinyinvariant.default)(false, "You cannot %s with <StaticRouter>", methodName) : (0, _tinyinvariant.default)(false);
    };
}
function noop() {}
/**
 * The public top-level API for a "static" <Router>, so-called because it
 * can't actually change the current location. Instead, it just records
 * location changes in a context object. Useful mainly in testing and
 * server-rendering scenarios.
 */ var StaticRouter = /*#__PURE__*/ function(_React$Component) {
    (0, _inheritsLoose.default)(StaticRouter, _React$Component);
    function StaticRouter() {
        var _this;
        for(var _len = arguments.length, args = new Array(_len), _key = 0; _key < _len; _key++){
            args[_key] = arguments[_key];
        }
        _this = _React$Component.call.apply(_React$Component, [
            this
        ].concat(args)) || this;
        _this.handlePush = function(location) {
            return _this.navigateTo(location, "PUSH");
        };
        _this.handleReplace = function(location) {
            return _this.navigateTo(location, "REPLACE");
        };
        _this.handleListen = function() {
            return noop;
        };
        _this.handleBlock = function() {
            return noop;
        };
        return _this;
    }
    var _proto = StaticRouter.prototype;
    _proto.navigateTo = function navigateTo(location, action) {
        var _this$props = this.props, _this$props$basename = _this$props.basename, basename = _this$props$basename === void 0 ? "" : _this$props$basename, _this$props$context = _this$props.context, context = _this$props$context === void 0 ? {} : _this$props$context;
        context.action = action;
        context.location = addBasename(basename, (0, _history.createLocation)(location));
        context.url = createURL(context.location);
    };
    _proto.render = function render() {
        var _this$props2 = this.props, _this$props2$basename = _this$props2.basename, basename = _this$props2$basename === void 0 ? "" : _this$props2$basename, _this$props2$context = _this$props2.context, context = _this$props2$context === void 0 ? {} : _this$props2$context, _this$props2$location = _this$props2.location, location = _this$props2$location === void 0 ? "/" : _this$props2$location, rest = (0, _objectWithoutPropertiesLoose.default)(_this$props2, [
            "basename",
            "context",
            "location"
        ]);
        var history = {
            createHref: function createHref(path) {
                return addLeadingSlash(basename + createURL(path));
            },
            action: "POP",
            location: stripBasename(basename, (0, _history.createLocation)(location)),
            push: this.handlePush,
            replace: this.handleReplace,
            go: staticHandler("go"),
            goBack: staticHandler("goBack"),
            goForward: staticHandler("goForward"),
            listen: this.handleListen,
            block: this.handleBlock
        };
        return /*#__PURE__*/ _react.default.createElement(Router, (0, _extends.default)({}, rest, {
            history: history,
            staticContext: context
        }));
    };
    return StaticRouter;
}(_react.default.Component);
if ("production" !== "production") {
    StaticRouter.propTypes = {
        basename: _proptypes.default.string,
        context: _proptypes.default.object,
        location: _proptypes.default.oneOfType([
            _proptypes.default.string,
            _proptypes.default.object
        ])
    };
    StaticRouter.prototype.componentDidMount = function() {
        "production" !== "production" ? (0, _tinywarning.default)(!this.props.history, "<StaticRouter> ignores the history prop. To use a custom history, " + "use `import { Router }` instead of `import { StaticRouter as Router }`.") : void 0;
    };
}
/**
 * The public API for rendering the first <Route> that matches.
 */ var Switch = /*#__PURE__*/ function(_React$Component) {
    (0, _inheritsLoose.default)(Switch, _React$Component);
    function Switch() {
        return _React$Component.apply(this, arguments) || this;
    }
    var _proto = Switch.prototype;
    _proto.render = function render() {
        var _this = this;
        return /*#__PURE__*/ _react.default.createElement(context.Consumer, null, function(context) {
            !context ? "production" !== "production" ? (0, _tinyinvariant.default)(false, "You should not use <Switch> outside a <Router>") : (0, _tinyinvariant.default)(false) : void 0;
            var location = _this.props.location || context.location;
            var element, match; // We use React.Children.forEach instead of React.Children.toArray().find()
            // here because toArray adds keys to all child elements and we do not want
            // to trigger an unmount/remount for two <Route>s that render the same
            // component at different URLs.
            _react.default.Children.forEach(_this.props.children, function(child) {
                if (match == null && /*#__PURE__*/ _react.default.isValidElement(child)) {
                    element = child;
                    var path = child.props.path || child.props.from;
                    match = path ? matchPath(location.pathname, (0, _extends.default)({}, child.props, {
                        path: path
                    })) : context.match;
                }
            });
            return match ? /*#__PURE__*/ _react.default.cloneElement(element, {
                location: location,
                computedMatch: match
            }) : null;
        });
    };
    return Switch;
}(_react.default.Component);
if ("production" !== "production") {
    Switch.propTypes = {
        children: _proptypes.default.node,
        location: _proptypes.default.object
    };
    Switch.prototype.componentDidUpdate = function(prevProps) {
        "production" !== "production" ? (0, _tinywarning.default)(!(this.props.location && !prevProps.location), '<Switch> elements should not change from uncontrolled to controlled (or vice versa). You initially used no "location" prop and then provided one on a subsequent render.') : void 0;
        "production" !== "production" ? (0, _tinywarning.default)(!(!this.props.location && prevProps.location), '<Switch> elements should not change from controlled to uncontrolled (or vice versa). You provided a "location" prop initially but omitted it on a subsequent render.') : void 0;
    };
}
var useContext = _react.default.useContext;
function useHistory() {
    if ("production" !== "production") {
        !(typeof useContext === "function") ? "production" !== "production" ? (0, _tinyinvariant.default)(false, "You must use React >= 16.8 in order to use useHistory()") : (0, _tinyinvariant.default)(false) : void 0;
    }
    return useContext(historyContext);
}
if ("production" !== "production") {
    if (typeof window !== "undefined") {
        var global$1 = window;
        var key = "__react_router_build__";
        var buildNames = {
            cjs: "CommonJS",
            esm: "ES modules",
            umd: "UMD"
        };
        if (global$1[key] && global$1[key] !== "esm") {
            var initialBuildName = buildNames[global$1[key]];
            var secondaryBuildName = buildNames["esm"]; // TODO: Add link to article that explains in detail how to avoid
            // loading 2 different builds.
            throw new Error("You are loading the " + secondaryBuildName + " build of React Router " + ("on a page that is already running the " + initialBuildName + " ") + "build, so things won't work right.");
        }
        global$1[key] = "esm";
    }
}
 //# sourceMappingURL=react-router.js.map

},
"aa0f4db3": function(module, exports, farmRequire, farmDynamicRequire) {
var baseGet = farmRequire("e97bdcac", true);
/**
 * A specialized version of `baseProperty` which supports deep paths.
 *
 * @private
 * @param {Array|string} path The path of the property to get.
 * @returns {Function} Returns the new accessor function.
 */ function basePropertyDeep(path) {
    return function(object) {
        return baseGet(object, path);
    };
}
module.exports = basePropertyDeep;

},
"ab2b5ee0": function(module, exports, farmRequire, farmDynamicRequire) {
var baseClone = farmRequire("1ed1673b", true);
/** Used to compose bitmasks for cloning. */ var CLONE_DEEP_FLAG = 1, CLONE_SYMBOLS_FLAG = 4;
/**
 * This method is like `_.clone` except that it recursively clones `value`.
 *
 * @static
 * @memberOf _
 * @since 1.0.0
 * @category Lang
 * @param {*} value The value to recursively clone.
 * @returns {*} Returns the deep cloned value.
 * @see _.clone
 * @example
 *
 * var objects = [{ 'a': 1 }, { 'b': 2 }];
 *
 * var deep = _.cloneDeep(objects);
 * console.log(deep[0] === objects[0]);
 * // => false
 */ function cloneDeep(value) {
    return baseClone(value, CLONE_DEEP_FLAG | CLONE_SYMBOLS_FLAG);
}
module.exports = cloneDeep;

},
"bcd6de30": function(module, exports, farmRequire, farmDynamicRequire) {
var conversions = farmRequire("e687b7f8", true);
/*
	this function routes a model to all other models.

	all functions that are routed have a property `.conversion` attached
	to the returned synthetic function. This property is an array
	of strings, each with the steps in between the 'from' and 'to'
	color models (inclusive).

	conversions that are not possible simply are not included.
*/ function buildGraph() {
    var graph = {};
    // https://jsperf.com/object-keys-vs-for-in-with-closure/3
    var models = Object.keys(conversions);
    for(var len = models.length, i = 0; i < len; i++){
        graph[models[i]] = {
            // http://jsperf.com/1-vs-infinity
            // micro-opt, but this is simple.
            distance: -1,
            parent: null
        };
    }
    return graph;
}
// https://en.wikipedia.org/wiki/Breadth-first_search
function deriveBFS(fromModel) {
    var graph = buildGraph();
    var queue = [
        fromModel
    ]; // unshift -> queue -> pop
    graph[fromModel].distance = 0;
    while(queue.length){
        var current = queue.pop();
        var adjacents = Object.keys(conversions[current]);
        for(var len = adjacents.length, i = 0; i < len; i++){
            var adjacent = adjacents[i];
            var node = graph[adjacent];
            if (node.distance === -1) {
                node.distance = graph[current].distance + 1;
                node.parent = current;
                queue.unshift(adjacent);
            }
        }
    }
    return graph;
}
function link(from, to) {
    return function(args) {
        return to(from(args));
    };
}
function wrapConversion(toModel, graph) {
    var path = [
        graph[toModel].parent,
        toModel
    ];
    var fn = conversions[graph[toModel].parent][toModel];
    var cur = graph[toModel].parent;
    while(graph[cur].parent){
        path.unshift(graph[cur].parent);
        fn = link(conversions[graph[cur].parent][cur], fn);
        cur = graph[cur].parent;
    }
    fn.conversion = path;
    return fn;
}
module.exports = function(fromModel) {
    var graph = deriveBFS(fromModel);
    var conversion = {};
    var models = Object.keys(graph);
    for(var len = models.length, i = 0; i < len; i++){
        var toModel = models[i];
        var node = graph[toModel];
        if (node.parent === null) {
            continue;
        }
        conversion[toModel] = wrapConversion(toModel, graph);
    }
    return conversion;
};

},
"c77bba68": function(module, exports, farmRequire, farmDynamicRequire) {
'use strict';
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.ReactCSS = exports.loop = exports.handleActive = exports.handleHover = exports.hover = undefined;
var _flattenNames = farmRequire("5b59c548", true);
var _flattenNames2 = _interopRequireDefault(_flattenNames);
var _mergeClasses = farmRequire("da6823cb", true);
var _mergeClasses2 = _interopRequireDefault(_mergeClasses);
var _autoprefix = farmRequire("1b3a60c0", true);
var _autoprefix2 = _interopRequireDefault(_autoprefix);
var _hover2 = farmRequire("896f638f", true);
var _hover3 = _interopRequireDefault(_hover2);
var _active = farmRequire("898d9be6", true);
var _active2 = _interopRequireDefault(_active);
var _loop2 = farmRequire("4c0d486a", true);
var _loop3 = _interopRequireDefault(_loop2);
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
exports.hover = _hover3.default;
exports.handleHover = _hover3.default;
exports.handleActive = _active2.default;
exports.loop = _loop3.default;
var ReactCSS = exports.ReactCSS = function ReactCSS(classes) {
    for(var _len = arguments.length, activations = Array(_len > 1 ? _len - 1 : 0), _key = 1; _key < _len; _key++){
        activations[_key - 1] = arguments[_key];
    }
    var activeNames = (0, _flattenNames2.default)(activations);
    var merged = (0, _mergeClasses2.default)(classes, activeNames);
    return (0, _autoprefix2.default)(merged);
};
exports.default = ReactCSS;

},
"c894d30e": function(module, exports, farmRequire, farmDynamicRequire) {
var isStrictComparable = farmRequire("4d74d49a", true), keys = farmRequire("ed28e463", true);
/**
 * Gets the property names, values, and compare flags of `object`.
 *
 * @private
 * @param {Object} object The object to query.
 * @returns {Array} Returns the match data of `object`.
 */ function getMatchData(object) {
    var result = keys(object), length = result.length;
    while(length--){
        var key = result[length], value = object[key];
        result[length] = [
            key,
            value,
            isStrictComparable(value)
        ];
    }
    return result;
}
module.exports = getMatchData;

},
"ca993b72": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "createStore", {
    enumerable: true,
    get: function() {
        return createStore;
    }
});
/**
 * Adapted from React: https://github.com/facebook/react/blob/master/packages/shared/formatProdErrorMessage.js
 *
 * Do not require this module directly! Use normal throw error calls. These messages will be replaced with error codes
 * during build.
 * @param {number} code
 */ function formatProdErrorMessage(code) {
    return "Minified Redux error #" + code + "; visit https://redux.js.org/Errors?code=" + code + " for the full message or " + 'use the non-minified dev environment for full errors. ';
}
// Inlined version of the `symbol-observable` polyfill
var $$observable = function() {
    return typeof Symbol === 'function' && Symbol.observable || '@@observable';
}();
/**
 * These are private action types reserved by Redux.
 * For any unknown actions, you must return the current state.
 * If the current state is undefined, you must return the initial state.
 * Do not reference these action types directly in your code.
 */ var randomString = function randomString() {
    return Math.random().toString(36).substring(7).split('').join('.');
};
var ActionTypes = {
    INIT: "@@redux/INIT" + randomString(),
    REPLACE: "@@redux/REPLACE" + randomString(),
    PROBE_UNKNOWN_ACTION: function PROBE_UNKNOWN_ACTION() {
        return "@@redux/PROBE_UNKNOWN_ACTION" + randomString();
    }
};
/**
 * @param {any} obj The object to inspect.
 * @returns {boolean} True if the argument appears to be a plain object.
 */ function isPlainObject(obj) {
    if (typeof obj !== 'object' || obj === null) return false;
    var proto = obj;
    while(Object.getPrototypeOf(proto) !== null){
        proto = Object.getPrototypeOf(proto);
    }
    return Object.getPrototypeOf(obj) === proto;
}
// Inlined / shortened version of `kindOf` from https://github.com/jonschlinkert/kind-of
function miniKindOf(val) {
    if (val === void 0) return 'undefined';
    if (val === null) return 'null';
    var type = typeof val;
    switch(type){
        case 'boolean':
        case 'string':
        case 'number':
        case 'symbol':
        case 'function':
            {
                return type;
            }
    }
    if (Array.isArray(val)) return 'array';
    if (isDate(val)) return 'date';
    if (isError(val)) return 'error';
    var constructorName = ctorName(val);
    switch(constructorName){
        case 'Symbol':
        case 'Promise':
        case 'WeakMap':
        case 'WeakSet':
        case 'Map':
        case 'Set':
            return constructorName;
    } // other
    return type.slice(8, -1).toLowerCase().replace(/\s/g, '');
}
function ctorName(val) {
    return typeof val.constructor === 'function' ? val.constructor.name : null;
}
function isError(val) {
    return val instanceof Error || typeof val.message === 'string' && val.constructor && typeof val.constructor.stackTraceLimit === 'number';
}
function isDate(val) {
    if (val instanceof Date) return true;
    return typeof val.toDateString === 'function' && typeof val.getDate === 'function' && typeof val.setDate === 'function';
}
function kindOf(val) {
    var typeOfVal = typeof val;
    if ("production" !== 'production') {
        typeOfVal = miniKindOf(val);
    }
    return typeOfVal;
}
/**
 * @deprecated
 *
 * **We recommend using the `configureStore` method
 * of the `@reduxjs/toolkit` package**, which replaces `createStore`.
 *
 * Redux Toolkit is our recommended approach for writing Redux logic today,
 * including store setup, reducers, data fetching, and more.
 *
 * **For more details, please read this Redux docs page:**
 * **https://redux.js.org/introduction/why-rtk-is-redux-today**
 *
 * `configureStore` from Redux Toolkit is an improved version of `createStore` that
 * simplifies setup and helps avoid common bugs.
 *
 * You should not be using the `redux` core package by itself today, except for learning purposes.
 * The `createStore` method from the core `redux` package will not be removed, but we encourage
 * all users to migrate to using Redux Toolkit for all Redux code.
 *
 * If you want to use `createStore` without this visual deprecation warning, use
 * the `legacy_createStore` import instead:
 *
 * `import { legacy_createStore as createStore} from 'redux'`
 *
 */ function createStore(reducer, preloadedState, enhancer) {
    var _ref2;
    if (typeof preloadedState === 'function' && typeof enhancer === 'function' || typeof enhancer === 'function' && typeof arguments[3] === 'function') {
        throw new Error("production" === "production" ? formatProdErrorMessage(0) : 'It looks like you are passing several store enhancers to ' + 'createStore(). This is not supported. Instead, compose them ' + 'together to a single function. See https://redux.js.org/tutorials/fundamentals/part-4-store#creating-a-store-with-enhancers for an example.');
    }
    if (typeof preloadedState === 'function' && typeof enhancer === 'undefined') {
        enhancer = preloadedState;
        preloadedState = undefined;
    }
    if (typeof enhancer !== 'undefined') {
        if (typeof enhancer !== 'function') {
            throw new Error("production" === "production" ? formatProdErrorMessage(1) : "Expected the enhancer to be a function. Instead, received: '" + kindOf(enhancer) + "'");
        }
        return enhancer(createStore)(reducer, preloadedState);
    }
    if (typeof reducer !== 'function') {
        throw new Error("production" === "production" ? formatProdErrorMessage(2) : "Expected the root reducer to be a function. Instead, received: '" + kindOf(reducer) + "'");
    }
    var currentReducer = reducer;
    var currentState = preloadedState;
    var currentListeners = [];
    var nextListeners = currentListeners;
    var isDispatching = false;
    /**
   * This makes a shallow copy of currentListeners so we can use
   * nextListeners as a temporary list while dispatching.
   *
   * This prevents any bugs around consumers calling
   * subscribe/unsubscribe in the middle of a dispatch.
   */ function ensureCanMutateNextListeners() {
        if (nextListeners === currentListeners) {
            nextListeners = currentListeners.slice();
        }
    }
    /**
   * Reads the state tree managed by the store.
   *
   * @returns {any} The current state tree of your application.
   */ function getState() {
        if (isDispatching) {
            throw new Error("production" === "production" ? formatProdErrorMessage(3) : 'You may not call store.getState() while the reducer is executing. ' + 'The reducer has already received the state as an argument. ' + 'Pass it down from the top reducer instead of reading it from the store.');
        }
        return currentState;
    }
    /**
   * Adds a change listener. It will be called any time an action is dispatched,
   * and some part of the state tree may potentially have changed. You may then
   * call `getState()` to read the current state tree inside the callback.
   *
   * You may call `dispatch()` from a change listener, with the following
   * caveats:
   *
   * 1. The subscriptions are snapshotted just before every `dispatch()` call.
   * If you subscribe or unsubscribe while the listeners are being invoked, this
   * will not have any effect on the `dispatch()` that is currently in progress.
   * However, the next `dispatch()` call, whether nested or not, will use a more
   * recent snapshot of the subscription list.
   *
   * 2. The listener should not expect to see all state changes, as the state
   * might have been updated multiple times during a nested `dispatch()` before
   * the listener is called. It is, however, guaranteed that all subscribers
   * registered before the `dispatch()` started will be called with the latest
   * state by the time it exits.
   *
   * @param {Function} listener A callback to be invoked on every dispatch.
   * @returns {Function} A function to remove this change listener.
   */ function subscribe(listener) {
        if (typeof listener !== 'function') {
            throw new Error("production" === "production" ? formatProdErrorMessage(4) : "Expected the listener to be a function. Instead, received: '" + kindOf(listener) + "'");
        }
        if (isDispatching) {
            throw new Error("production" === "production" ? formatProdErrorMessage(5) : 'You may not call store.subscribe() while the reducer is executing. ' + 'If you would like to be notified after the store has been updated, subscribe from a ' + 'component and invoke store.getState() in the callback to access the latest state. ' + 'See https://redux.js.org/api/store#subscribelistener for more details.');
        }
        var isSubscribed = true;
        ensureCanMutateNextListeners();
        nextListeners.push(listener);
        return function unsubscribe() {
            if (!isSubscribed) {
                return;
            }
            if (isDispatching) {
                throw new Error("production" === "production" ? formatProdErrorMessage(6) : 'You may not unsubscribe from a store listener while the reducer is executing. ' + 'See https://redux.js.org/api/store#subscribelistener for more details.');
            }
            isSubscribed = false;
            ensureCanMutateNextListeners();
            var index = nextListeners.indexOf(listener);
            nextListeners.splice(index, 1);
            currentListeners = null;
        };
    }
    /**
   * Dispatches an action. It is the only way to trigger a state change.
   *
   * The `reducer` function, used to create the store, will be called with the
   * current state tree and the given `action`. Its return value will
   * be considered the **next** state of the tree, and the change listeners
   * will be notified.
   *
   * The base implementation only supports plain object actions. If you want to
   * dispatch a Promise, an Observable, a thunk, or something else, you need to
   * wrap your store creating function into the corresponding middleware. For
   * example, see the documentation for the `redux-thunk` package. Even the
   * middleware will eventually dispatch plain object actions using this method.
   *
   * @param {Object} action A plain object representing “what changed”. It is
   * a good idea to keep actions serializable so you can record and replay user
   * sessions, or use the time travelling `redux-devtools`. An action must have
   * a `type` property which may not be `undefined`. It is a good idea to use
   * string constants for action types.
   *
   * @returns {Object} For convenience, the same action object you dispatched.
   *
   * Note that, if you use a custom middleware, it may wrap `dispatch()` to
   * return something else (for example, a Promise you can await).
   */ function dispatch(action) {
        if (!isPlainObject(action)) {
            throw new Error("production" === "production" ? formatProdErrorMessage(7) : "Actions must be plain objects. Instead, the actual type was: '" + kindOf(action) + "'. You may need to add middleware to your store setup to handle dispatching other values, such as 'redux-thunk' to handle dispatching functions. See https://redux.js.org/tutorials/fundamentals/part-4-store#middleware and https://redux.js.org/tutorials/fundamentals/part-6-async-logic#using-the-redux-thunk-middleware for examples.");
        }
        if (typeof action.type === 'undefined') {
            throw new Error("production" === "production" ? formatProdErrorMessage(8) : 'Actions may not have an undefined "type" property. You may have misspelled an action type string constant.');
        }
        if (isDispatching) {
            throw new Error("production" === "production" ? formatProdErrorMessage(9) : 'Reducers may not dispatch actions.');
        }
        try {
            isDispatching = true;
            currentState = currentReducer(currentState, action);
        } finally{
            isDispatching = false;
        }
        var listeners = currentListeners = nextListeners;
        for(var i = 0; i < listeners.length; i++){
            var listener = listeners[i];
            listener();
        }
        return action;
    }
    /**
   * Replaces the reducer currently used by the store to calculate the state.
   *
   * You might need this if your app implements code splitting and you want to
   * load some of the reducers dynamically. You might also need this if you
   * implement a hot reloading mechanism for Redux.
   *
   * @param {Function} nextReducer The reducer for the store to use instead.
   * @returns {void}
   */ function replaceReducer(nextReducer) {
        if (typeof nextReducer !== 'function') {
            throw new Error("production" === "production" ? formatProdErrorMessage(10) : "Expected the nextReducer to be a function. Instead, received: '" + kindOf(nextReducer));
        }
        currentReducer = nextReducer; // This action has a similiar effect to ActionTypes.INIT.
        // Any reducers that existed in both the new and old rootReducer
        // will receive the previous state. This effectively populates
        // the new state tree with any relevant data from the old one.
        dispatch({
            type: ActionTypes.REPLACE
        });
    }
    /**
   * Interoperability point for observable/reactive libraries.
   * @returns {observable} A minimal observable of state changes.
   * For more information, see the observable proposal:
   * https://github.com/tc39/proposal-observable
   */ function observable() {
        var _ref;
        var outerSubscribe = subscribe;
        return _ref = {
            /**
       * The minimal observable subscription method.
       * @param {Object} observer Any object that can be used as an observer.
       * The observer object should have a `next` method.
       * @returns {subscription} An object with an `unsubscribe` method that can
       * be used to unsubscribe the observable from the store, and prevent further
       * emission of values from the observable.
       */ subscribe: function subscribe(observer) {
                if (typeof observer !== 'object' || observer === null) {
                    throw new Error("production" === "production" ? formatProdErrorMessage(11) : "Expected the observer to be an object. Instead, received: '" + kindOf(observer) + "'");
                }
                function observeState() {
                    if (observer.next) {
                        observer.next(getState());
                    }
                }
                observeState();
                var unsubscribe = outerSubscribe(observeState);
                return {
                    unsubscribe: unsubscribe
                };
            }
        }, _ref[$$observable] = function() {
            return this;
        }, _ref;
    } // When a store is created, an "INIT" action is dispatched so that every
    // reducer returns their initial state. This effectively populates
    // the initial state tree.
    dispatch({
        type: ActionTypes.INIT
    });
    return _ref2 = {
        dispatch: dispatch,
        subscribe: subscribe,
        getState: getState,
        replaceReducer: replaceReducer
    }, _ref2[$$observable] = observable, _ref2;
}

},
"d30c21d7": function(module, exports, farmRequire, farmDynamicRequire) {
var Stack = farmRequire("75e40c36", true), baseIsEqual = farmRequire("06efa494", true);
/** Used to compose bitmasks for value comparisons. */ var COMPARE_PARTIAL_FLAG = 1, COMPARE_UNORDERED_FLAG = 2;
/**
 * The base implementation of `_.isMatch` without support for iteratee shorthands.
 *
 * @private
 * @param {Object} object The object to inspect.
 * @param {Object} source The object of property values to match.
 * @param {Array} matchData The property names, values, and compare flags to match.
 * @param {Function} [customizer] The function to customize comparisons.
 * @returns {boolean} Returns `true` if `object` is a match, else `false`.
 */ function baseIsMatch(object, source, matchData, customizer) {
    var index = matchData.length, length = index, noCustomizer = !customizer;
    if (object == null) {
        return !length;
    }
    object = Object(object);
    while(index--){
        var data = matchData[index];
        if (noCustomizer && data[2] ? data[1] !== object[data[0]] : !(data[0] in object)) {
            return false;
        }
    }
    while(++index < length){
        data = matchData[index];
        var key = data[0], objValue = object[key], srcValue = data[1];
        if (noCustomizer && data[2]) {
            if (objValue === undefined && !(key in object)) {
                return false;
            }
        } else {
            var stack = new Stack;
            if (customizer) {
                var result = customizer(objValue, srcValue, key, object, source, stack);
            }
            if (!(result === undefined ? baseIsEqual(srcValue, objValue, COMPARE_PARTIAL_FLAG | COMPARE_UNORDERED_FLAG, customizer, stack) : result)) {
                return false;
            }
        }
    }
    return true;
}
module.exports = baseIsMatch;

},
"d4532a2a": function(module, exports, farmRequire, farmDynamicRequire) {
var isarray = farmRequire("b28429bc", true);
/**
 * Expose `pathToRegexp`.
 */ module.exports = pathToRegexp;
module.exports.parse = parse;
module.exports.compile = compile;
module.exports.tokensToFunction = tokensToFunction;
module.exports.tokensToRegExp = tokensToRegExp;
/**
 * The main path matching regexp utility.
 *
 * @type {RegExp}
 */ var PATH_REGEXP = new RegExp([
    // Match escaped characters that would otherwise appear in future matches.
    // This allows the user to escape special characters that won't transform.
    '(\\\\.)',
    // Match Express-style parameters and un-named parameters with a prefix
    // and optional suffixes. Matches appear as:
    //
    // "/:test(\\d+)?" => ["/", "test", "\d+", undefined, "?", undefined]
    // "/route(\\d+)"  => [undefined, undefined, undefined, "\d+", undefined, undefined]
    // "/*"            => ["/", undefined, undefined, undefined, undefined, "*"]
    '([\\/.])?(?:(?:\\:(\\w+)(?:\\(((?:\\\\.|[^\\\\()])+)\\))?|\\(((?:\\\\.|[^\\\\()])+)\\))([+*?])?|(\\*))'
].join('|'), 'g');
/**
 * Parse a string for the raw tokens.
 *
 * @param  {string}  str
 * @param  {Object=} options
 * @return {!Array}
 */ function parse(str, options) {
    var tokens = [];
    var key = 0;
    var index = 0;
    var path = '';
    var defaultDelimiter = options && options.delimiter || '/';
    var res;
    while((res = PATH_REGEXP.exec(str)) != null){
        var m = res[0];
        var escaped = res[1];
        var offset = res.index;
        path += str.slice(index, offset);
        index = offset + m.length;
        // Ignore already escaped sequences.
        if (escaped) {
            path += escaped[1];
            continue;
        }
        var next = str[index];
        var prefix = res[2];
        var name = res[3];
        var capture = res[4];
        var group = res[5];
        var modifier = res[6];
        var asterisk = res[7];
        // Push the current path onto the tokens.
        if (path) {
            tokens.push(path);
            path = '';
        }
        var partial = prefix != null && next != null && next !== prefix;
        var repeat = modifier === '+' || modifier === '*';
        var optional = modifier === '?' || modifier === '*';
        var delimiter = res[2] || defaultDelimiter;
        var pattern = capture || group;
        tokens.push({
            name: name || key++,
            prefix: prefix || '',
            delimiter: delimiter,
            optional: optional,
            repeat: repeat,
            partial: partial,
            asterisk: !!asterisk,
            pattern: pattern ? escapeGroup(pattern) : asterisk ? '.*' : '[^' + escapeString(delimiter) + ']+?'
        });
    }
    // Match any characters still remaining.
    if (index < str.length) {
        path += str.substr(index);
    }
    // If the path exists, push it onto the end.
    if (path) {
        tokens.push(path);
    }
    return tokens;
}
/**
 * Compile a string to a template function for the path.
 *
 * @param  {string}             str
 * @param  {Object=}            options
 * @return {!function(Object=, Object=)}
 */ function compile(str, options) {
    return tokensToFunction(parse(str, options), options);
}
/**
 * Prettier encoding of URI path segments.
 *
 * @param  {string}
 * @return {string}
 */ function encodeURIComponentPretty(str) {
    return encodeURI(str).replace(/[\/?#]/g, function(c) {
        return '%' + c.charCodeAt(0).toString(16).toUpperCase();
    });
}
/**
 * Encode the asterisk parameter. Similar to `pretty`, but allows slashes.
 *
 * @param  {string}
 * @return {string}
 */ function encodeAsterisk(str) {
    return encodeURI(str).replace(/[?#]/g, function(c) {
        return '%' + c.charCodeAt(0).toString(16).toUpperCase();
    });
}
/**
 * Expose a method for transforming tokens into the path function.
 */ function tokensToFunction(tokens, options) {
    // Compile all the tokens into regexps.
    var matches = new Array(tokens.length);
    // Compile all the patterns before compilation.
    for(var i = 0; i < tokens.length; i++){
        if (typeof tokens[i] === 'object') {
            matches[i] = new RegExp('^(?:' + tokens[i].pattern + ')$', flags(options));
        }
    }
    return function(obj, opts) {
        var path = '';
        var data = obj || {};
        var options = opts || {};
        var encode = options.pretty ? encodeURIComponentPretty : encodeURIComponent;
        for(var i = 0; i < tokens.length; i++){
            var token = tokens[i];
            if (typeof token === 'string') {
                path += token;
                continue;
            }
            var value = data[token.name];
            var segment;
            if (value == null) {
                if (token.optional) {
                    // Prepend partial segment prefixes.
                    if (token.partial) {
                        path += token.prefix;
                    }
                    continue;
                } else {
                    throw new TypeError('Expected "' + token.name + '" to be defined');
                }
            }
            if (isarray(value)) {
                if (!token.repeat) {
                    throw new TypeError('Expected "' + token.name + '" to not repeat, but received `' + JSON.stringify(value) + '`');
                }
                if (value.length === 0) {
                    if (token.optional) {
                        continue;
                    } else {
                        throw new TypeError('Expected "' + token.name + '" to not be empty');
                    }
                }
                for(var j = 0; j < value.length; j++){
                    segment = encode(value[j]);
                    if (!matches[i].test(segment)) {
                        throw new TypeError('Expected all "' + token.name + '" to match "' + token.pattern + '", but received `' + JSON.stringify(segment) + '`');
                    }
                    path += (j === 0 ? token.prefix : token.delimiter) + segment;
                }
                continue;
            }
            segment = token.asterisk ? encodeAsterisk(value) : encode(value);
            if (!matches[i].test(segment)) {
                throw new TypeError('Expected "' + token.name + '" to match "' + token.pattern + '", but received "' + segment + '"');
            }
            path += token.prefix + segment;
        }
        return path;
    };
}
/**
 * Escape a regular expression string.
 *
 * @param  {string} str
 * @return {string}
 */ function escapeString(str) {
    return str.replace(/([.+*?=^!:${}()[\]|\/\\])/g, '\\$1');
}
/**
 * Escape the capturing group by escaping special characters and meaning.
 *
 * @param  {string} group
 * @return {string}
 */ function escapeGroup(group) {
    return group.replace(/([=!:$\/()])/g, '\\$1');
}
/**
 * Attach the keys as a property of the regexp.
 *
 * @param  {!RegExp} re
 * @param  {Array}   keys
 * @return {!RegExp}
 */ function attachKeys(re, keys) {
    re.keys = keys;
    return re;
}
/**
 * Get the flags for a regexp from the options.
 *
 * @param  {Object} options
 * @return {string}
 */ function flags(options) {
    return options && options.sensitive ? '' : 'i';
}
/**
 * Pull out keys from a regexp.
 *
 * @param  {!RegExp} path
 * @param  {!Array}  keys
 * @return {!RegExp}
 */ function regexpToRegexp(path, keys) {
    // Use a negative lookahead to match only capturing groups.
    var groups = path.source.match(/\((?!\?)/g);
    if (groups) {
        for(var i = 0; i < groups.length; i++){
            keys.push({
                name: i,
                prefix: null,
                delimiter: null,
                optional: false,
                repeat: false,
                partial: false,
                asterisk: false,
                pattern: null
            });
        }
    }
    return attachKeys(path, keys);
}
/**
 * Transform an array into a regexp.
 *
 * @param  {!Array}  path
 * @param  {Array}   keys
 * @param  {!Object} options
 * @return {!RegExp}
 */ function arrayToRegexp(path, keys, options) {
    var parts = [];
    for(var i = 0; i < path.length; i++){
        parts.push(pathToRegexp(path[i], keys, options).source);
    }
    var regexp = new RegExp('(?:' + parts.join('|') + ')', flags(options));
    return attachKeys(regexp, keys);
}
/**
 * Create a path regexp from string input.
 *
 * @param  {string}  path
 * @param  {!Array}  keys
 * @param  {!Object} options
 * @return {!RegExp}
 */ function stringToRegexp(path, keys, options) {
    return tokensToRegExp(parse(path, options), keys, options);
}
/**
 * Expose a function for taking tokens and returning a RegExp.
 *
 * @param  {!Array}          tokens
 * @param  {(Array|Object)=} keys
 * @param  {Object=}         options
 * @return {!RegExp}
 */ function tokensToRegExp(tokens, keys, options) {
    if (!isarray(keys)) {
        options = /** @type {!Object} */ keys || options;
        keys = [];
    }
    options = options || {};
    var strict = options.strict;
    var end = options.end !== false;
    var route = '';
    // Iterate over the tokens and create our regexp string.
    for(var i = 0; i < tokens.length; i++){
        var token = tokens[i];
        if (typeof token === 'string') {
            route += escapeString(token);
        } else {
            var prefix = escapeString(token.prefix);
            var capture = '(?:' + token.pattern + ')';
            keys.push(token);
            if (token.repeat) {
                capture += '(?:' + prefix + capture + ')*';
            }
            if (token.optional) {
                if (!token.partial) {
                    capture = '(?:' + prefix + '(' + capture + '))?';
                } else {
                    capture = prefix + '(' + capture + ')?';
                }
            } else {
                capture = prefix + '(' + capture + ')';
            }
            route += capture;
        }
    }
    var delimiter = escapeString(options.delimiter || '/');
    var endsWithDelimiter = route.slice(-delimiter.length) === delimiter;
    // In non-strict mode we allow a slash at the end of match. If the path to
    // match already ends with a slash, we remove it for consistency. The slash
    // is valid at the end of a path match, not in the middle. This is important
    // in non-ending mode, where "/test/" shouldn't match "/test//route".
    if (!strict) {
        route = (endsWithDelimiter ? route.slice(0, -delimiter.length) : route) + '(?:' + delimiter + '(?=$))?';
    }
    if (end) {
        route += '$';
    } else {
        // In non-ending mode, we need the capturing groups to match as much as
        // possible by using a positive lookahead to the end or next path segment.
        route += strict && endsWithDelimiter ? '' : '(?=' + delimiter + '|$)';
    }
    return attachKeys(new RegExp('^' + route, flags(options)), keys);
}
/**
 * Normalize the given path string, returning a regular expression.
 *
 * An empty array can be passed in for the keys, which will hold the
 * placeholder key descriptions. For example, using `/user/:id`, `keys` will
 * contain `[{ name: 'id', delimiter: '/', optional: false, repeat: false }]`.
 *
 * @param  {(string|RegExp|Array)} path
 * @param  {(Array|Object)=}       keys
 * @param  {Object=}               options
 * @return {!RegExp}
 */ function pathToRegexp(path, keys, options) {
    if (!isarray(keys)) {
        options = /** @type {!Object} */ keys || options;
        keys = [];
    }
    options = options || {};
    if (path instanceof RegExp) {
        return regexpToRegexp(path, /** @type {!Array} */ keys);
    }
    if (isarray(path)) {
        return arrayToRegexp(/** @type {!Array} */ path, /** @type {!Array} */ keys, options);
    }
    return stringToRegexp(/** @type {string} */ path, /** @type {!Array} */ keys, options);
}

},
"da6823cb": function(module, exports, farmRequire, farmDynamicRequire) {
'use strict';
Object.defineProperty(exports, "__esModule", {
    value: true
});
exports.mergeClasses = undefined;
var _forOwn2 = farmRequire("74cb39e4", true);
var _forOwn3 = _interopRequireDefault(_forOwn2);
var _cloneDeep2 = farmRequire("ab2b5ee0", true);
var _cloneDeep3 = _interopRequireDefault(_cloneDeep2);
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
function _interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
var mergeClasses = exports.mergeClasses = function mergeClasses(classes) {
    var activeNames = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : [];
    var styles = classes.default && (0, _cloneDeep3.default)(classes.default) || {};
    activeNames.map(function(name) {
        var toMerge = classes[name];
        if (toMerge) {
            (0, _forOwn3.default)(toMerge, function(value, key) {
                if (!styles[key]) {
                    styles[key] = {};
                }
                styles[key] = _extends({}, styles[key], toMerge[key]);
            });
        }
        return name;
    });
    return styles;
};
exports.default = mergeClasses;

},
"da8dec8d": function(module, exports, farmRequire, farmDynamicRequire) {
var baseEach = farmRequire("4c7d0c51", true);
/**
 * Aggregates elements of `collection` on `accumulator` with keys transformed
 * by `iteratee` and values set by `setter`.
 *
 * @private
 * @param {Array|Object} collection The collection to iterate over.
 * @param {Function} setter The function to set `accumulator` values.
 * @param {Function} iteratee The iteratee to transform keys.
 * @param {Object} accumulator The initial aggregated object.
 * @returns {Function} Returns `accumulator`.
 */ function baseAggregator(collection, setter, iteratee, accumulator) {
    baseEach(collection, function(value, key, collection) {
        setter(accumulator, value, iteratee(value), collection);
    });
    return accumulator;
}
module.exports = baseAggregator;

},
"dea22812": function(module, exports, farmRequire, farmDynamicRequire) {
var baseHasIn = farmRequire("06db9f0a", true), hasPath = farmRequire("d9d3d3fa", true);
/**
 * Checks if `path` is a direct or inherited property of `object`.
 *
 * @static
 * @memberOf _
 * @since 4.0.0
 * @category Object
 * @param {Object} object The object to query.
 * @param {Array|string} path The path to check.
 * @returns {boolean} Returns `true` if `path` exists, else `false`.
 * @example
 *
 * var object = _.create({ 'a': _.create({ 'b': 2 }) });
 *
 * _.hasIn(object, 'a');
 * // => true
 *
 * _.hasIn(object, 'a.b');
 * // => true
 *
 * _.hasIn(object, ['a', 'b']);
 * // => true
 *
 * _.hasIn(object, 'b');
 * // => false
 */ function hasIn(object, path) {
    return object != null && hasPath(object, path, baseHasIn);
}
module.exports = hasIn;

},
"e57f3bb0": function(module, exports, farmRequire, farmDynamicRequire) {
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
    createBrowserHistory: function() {
        return createBrowserHistory;
    },
    createHashHistory: function() {
        return createHashHistory;
    },
    createLocation: function() {
        return createLocation;
    },
    createMemoryHistory: function() {
        return createMemoryHistory;
    },
    createPath: function() {
        return createPath;
    },
    locationsAreEqual: function() {
        return locationsAreEqual;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _extends = /*#__PURE__*/ _interop_require_default._(farmRequire("857a4e6a"));
const _resolvepathname = /*#__PURE__*/ _interop_require_default._(farmRequire("106ae2c0"));
const _valueequal = /*#__PURE__*/ _interop_require_default._(farmRequire("15b3e282"));
const _tinywarning = /*#__PURE__*/ _interop_require_default._(farmRequire("386b0d68"));
const _tinyinvariant = /*#__PURE__*/ _interop_require_default._(farmRequire("8d305800"));
function addLeadingSlash(path) {
    return path.charAt(0) === '/' ? path : '/' + path;
}
function stripLeadingSlash(path) {
    return path.charAt(0) === '/' ? path.substr(1) : path;
}
function hasBasename(path, prefix) {
    return path.toLowerCase().indexOf(prefix.toLowerCase()) === 0 && '/?#'.indexOf(path.charAt(prefix.length)) !== -1;
}
function stripBasename(path, prefix) {
    return hasBasename(path, prefix) ? path.substr(prefix.length) : path;
}
function stripTrailingSlash(path) {
    return path.charAt(path.length - 1) === '/' ? path.slice(0, -1) : path;
}
function parsePath(path) {
    var pathname = path || '/';
    var search = '';
    var hash = '';
    var hashIndex = pathname.indexOf('#');
    if (hashIndex !== -1) {
        hash = pathname.substr(hashIndex);
        pathname = pathname.substr(0, hashIndex);
    }
    var searchIndex = pathname.indexOf('?');
    if (searchIndex !== -1) {
        search = pathname.substr(searchIndex);
        pathname = pathname.substr(0, searchIndex);
    }
    return {
        pathname: pathname,
        search: search === '?' ? '' : search,
        hash: hash === '#' ? '' : hash
    };
}
function createPath(location) {
    var pathname = location.pathname, search = location.search, hash = location.hash;
    var path = pathname || '/';
    if (search && search !== '?') path += search.charAt(0) === '?' ? search : "?" + search;
    if (hash && hash !== '#') path += hash.charAt(0) === '#' ? hash : "#" + hash;
    return path;
}
function createLocation(path, state, key, currentLocation) {
    var location;
    if (typeof path === 'string') {
        // Two-arg form: push(path, state)
        location = parsePath(path);
        location.state = state;
    } else {
        // One-arg form: push(location)
        location = (0, _extends.default)({}, path);
        if (location.pathname === undefined) location.pathname = '';
        if (location.search) {
            if (location.search.charAt(0) !== '?') location.search = '?' + location.search;
        } else {
            location.search = '';
        }
        if (location.hash) {
            if (location.hash.charAt(0) !== '#') location.hash = '#' + location.hash;
        } else {
            location.hash = '';
        }
        if (state !== undefined && location.state === undefined) location.state = state;
    }
    try {
        location.pathname = decodeURI(location.pathname);
    } catch (e) {
        if (e instanceof URIError) {
            throw new URIError('Pathname "' + location.pathname + '" could not be decoded. ' + 'This is likely caused by an invalid percent-encoding.');
        } else {
            throw e;
        }
    }
    if (key) location.key = key;
    if (currentLocation) {
        // Resolve incomplete/relative pathname relative to current location.
        if (!location.pathname) {
            location.pathname = currentLocation.pathname;
        } else if (location.pathname.charAt(0) !== '/') {
            location.pathname = (0, _resolvepathname.default)(location.pathname, currentLocation.pathname);
        }
    } else {
        // When there is no prior location and pathname is empty, set it to /
        if (!location.pathname) {
            location.pathname = '/';
        }
    }
    return location;
}
function locationsAreEqual(a, b) {
    return a.pathname === b.pathname && a.search === b.search && a.hash === b.hash && a.key === b.key && (0, _valueequal.default)(a.state, b.state);
}
function createTransitionManager() {
    var prompt = null;
    function setPrompt(nextPrompt) {
        "production" !== "production" ? (0, _tinywarning.default)(prompt == null, 'A history supports only one prompt at a time') : void 0;
        prompt = nextPrompt;
        return function() {
            if (prompt === nextPrompt) prompt = null;
        };
    }
    function confirmTransitionTo(location, action, getUserConfirmation, callback) {
        // TODO: If another transition starts while we're still confirming
        // the previous one, we may end up in a weird state. Figure out the
        // best way to handle this.
        if (prompt != null) {
            var result = typeof prompt === 'function' ? prompt(location, action) : prompt;
            if (typeof result === 'string') {
                if (typeof getUserConfirmation === 'function') {
                    getUserConfirmation(result, callback);
                } else {
                    "production" !== "production" ? (0, _tinywarning.default)(false, 'A history needs a getUserConfirmation function in order to use a prompt message') : void 0;
                    callback(true);
                }
            } else {
                // Return false from a transition hook to cancel the transition.
                callback(result !== false);
            }
        } else {
            callback(true);
        }
    }
    var listeners = [];
    function appendListener(fn) {
        var isActive = true;
        function listener() {
            if (isActive) fn.apply(void 0, arguments);
        }
        listeners.push(listener);
        return function() {
            isActive = false;
            listeners = listeners.filter(function(item) {
                return item !== listener;
            });
        };
    }
    function notifyListeners() {
        for(var _len = arguments.length, args = new Array(_len), _key = 0; _key < _len; _key++){
            args[_key] = arguments[_key];
        }
        listeners.forEach(function(listener) {
            return listener.apply(void 0, args);
        });
    }
    return {
        setPrompt: setPrompt,
        confirmTransitionTo: confirmTransitionTo,
        appendListener: appendListener,
        notifyListeners: notifyListeners
    };
}
var canUseDOM = !!(typeof window !== 'undefined' && window.document && window.document.createElement);
function getConfirmation(message, callback) {
    callback(window.confirm(message)); // eslint-disable-line no-alert
}
/**
 * Returns true if the HTML5 history API is supported. Taken from Modernizr.
 *
 * https://github.com/Modernizr/Modernizr/blob/master/LICENSE
 * https://github.com/Modernizr/Modernizr/blob/master/feature-detects/history.js
 * changed to avoid false negatives for Windows Phones: https://github.com/reactjs/react-router/issues/586
 */ function supportsHistory() {
    var ua = window.navigator.userAgent;
    if ((ua.indexOf('Android 2.') !== -1 || ua.indexOf('Android 4.0') !== -1) && ua.indexOf('Mobile Safari') !== -1 && ua.indexOf('Chrome') === -1 && ua.indexOf('Windows Phone') === -1) return false;
    return window.history && 'pushState' in window.history;
}
/**
 * Returns true if browser fires popstate on hash change.
 * IE10 and IE11 do not.
 */ function supportsPopStateOnHashChange() {
    return window.navigator.userAgent.indexOf('Trident') === -1;
}
/**
 * Returns false if using go(n) with hash history causes a full page reload.
 */ function supportsGoWithoutReloadUsingHash() {
    return window.navigator.userAgent.indexOf('Firefox') === -1;
}
/**
 * Returns true if a given popstate event is an extraneous WebKit event.
 * Accounts for the fact that Chrome on iOS fires real popstate events
 * containing undefined state when pressing the back button.
 */ function isExtraneousPopstateEvent(event) {
    return event.state === undefined && navigator.userAgent.indexOf('CriOS') === -1;
}
var PopStateEvent = 'popstate';
var HashChangeEvent = 'hashchange';
function getHistoryState() {
    try {
        return window.history.state || {};
    } catch (e) {
        // IE 11 sometimes throws when accessing window.history.state
        // See https://github.com/ReactTraining/history/pull/289
        return {};
    }
}
/**
 * Creates a history object that uses the HTML5 history API including
 * pushState, replaceState, and the popstate event.
 */ function createBrowserHistory(props) {
    if (props === void 0) {
        props = {};
    }
    !canUseDOM ? "production" !== "production" ? (0, _tinyinvariant.default)(false, 'Browser history needs a DOM') : (0, _tinyinvariant.default)(false) : void 0;
    var globalHistory = window.history;
    var canUseHistory = supportsHistory();
    var needsHashChangeListener = !supportsPopStateOnHashChange();
    var _props = props, _props$forceRefresh = _props.forceRefresh, forceRefresh = _props$forceRefresh === void 0 ? false : _props$forceRefresh, _props$getUserConfirm = _props.getUserConfirmation, getUserConfirmation = _props$getUserConfirm === void 0 ? getConfirmation : _props$getUserConfirm, _props$keyLength = _props.keyLength, keyLength = _props$keyLength === void 0 ? 6 : _props$keyLength;
    var basename = props.basename ? stripTrailingSlash(addLeadingSlash(props.basename)) : '';
    function getDOMLocation(historyState) {
        var _ref = historyState || {}, key = _ref.key, state = _ref.state;
        var _window$location = window.location, pathname = _window$location.pathname, search = _window$location.search, hash = _window$location.hash;
        var path = pathname + search + hash;
        "production" !== "production" ? (0, _tinywarning.default)(!basename || hasBasename(path, basename), 'You are attempting to use a basename on a page whose URL path does not begin ' + 'with the basename. Expected path "' + path + '" to begin with "' + basename + '".') : void 0;
        if (basename) path = stripBasename(path, basename);
        return createLocation(path, state, key);
    }
    function createKey() {
        return Math.random().toString(36).substr(2, keyLength);
    }
    var transitionManager = createTransitionManager();
    function setState(nextState) {
        (0, _extends.default)(history, nextState);
        history.length = globalHistory.length;
        transitionManager.notifyListeners(history.location, history.action);
    }
    function handlePopState(event) {
        // Ignore extraneous popstate events in WebKit.
        if (isExtraneousPopstateEvent(event)) return;
        handlePop(getDOMLocation(event.state));
    }
    function handleHashChange() {
        handlePop(getDOMLocation(getHistoryState()));
    }
    var forceNextPop = false;
    function handlePop(location) {
        if (forceNextPop) {
            forceNextPop = false;
            setState();
        } else {
            var action = 'POP';
            transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
                if (ok) {
                    setState({
                        action: action,
                        location: location
                    });
                } else {
                    revertPop(location);
                }
            });
        }
    }
    function revertPop(fromLocation) {
        var toLocation = history.location; // TODO: We could probably make this more reliable by
        // keeping a list of keys we've seen in sessionStorage.
        // Instead, we just default to 0 for keys we don't know.
        var toIndex = allKeys.indexOf(toLocation.key);
        if (toIndex === -1) toIndex = 0;
        var fromIndex = allKeys.indexOf(fromLocation.key);
        if (fromIndex === -1) fromIndex = 0;
        var delta = toIndex - fromIndex;
        if (delta) {
            forceNextPop = true;
            go(delta);
        }
    }
    var initialLocation = getDOMLocation(getHistoryState());
    var allKeys = [
        initialLocation.key
    ]; // Public interface
    function createHref(location) {
        return basename + createPath(location);
    }
    function push(path, state) {
        "production" !== "production" ? (0, _tinywarning.default)(!(typeof path === 'object' && path.state !== undefined && state !== undefined), 'You should avoid providing a 2nd state argument to push when the 1st ' + 'argument is a location-like object that already has state; it is ignored') : void 0;
        var action = 'PUSH';
        var location = createLocation(path, state, createKey(), history.location);
        transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
            if (!ok) return;
            var href = createHref(location);
            var key = location.key, state = location.state;
            if (canUseHistory) {
                globalHistory.pushState({
                    key: key,
                    state: state
                }, null, href);
                if (forceRefresh) {
                    window.location.href = href;
                } else {
                    var prevIndex = allKeys.indexOf(history.location.key);
                    var nextKeys = allKeys.slice(0, prevIndex + 1);
                    nextKeys.push(location.key);
                    allKeys = nextKeys;
                    setState({
                        action: action,
                        location: location
                    });
                }
            } else {
                "production" !== "production" ? (0, _tinywarning.default)(state === undefined, 'Browser history cannot push state in browsers that do not support HTML5 history') : void 0;
                window.location.href = href;
            }
        });
    }
    function replace(path, state) {
        "production" !== "production" ? (0, _tinywarning.default)(!(typeof path === 'object' && path.state !== undefined && state !== undefined), 'You should avoid providing a 2nd state argument to replace when the 1st ' + 'argument is a location-like object that already has state; it is ignored') : void 0;
        var action = 'REPLACE';
        var location = createLocation(path, state, createKey(), history.location);
        transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
            if (!ok) return;
            var href = createHref(location);
            var key = location.key, state = location.state;
            if (canUseHistory) {
                globalHistory.replaceState({
                    key: key,
                    state: state
                }, null, href);
                if (forceRefresh) {
                    window.location.replace(href);
                } else {
                    var prevIndex = allKeys.indexOf(history.location.key);
                    if (prevIndex !== -1) allKeys[prevIndex] = location.key;
                    setState({
                        action: action,
                        location: location
                    });
                }
            } else {
                "production" !== "production" ? (0, _tinywarning.default)(state === undefined, 'Browser history cannot replace state in browsers that do not support HTML5 history') : void 0;
                window.location.replace(href);
            }
        });
    }
    function go(n) {
        globalHistory.go(n);
    }
    function goBack() {
        go(-1);
    }
    function goForward() {
        go(1);
    }
    var listenerCount = 0;
    function checkDOMListeners(delta) {
        listenerCount += delta;
        if (listenerCount === 1 && delta === 1) {
            window.addEventListener(PopStateEvent, handlePopState);
            if (needsHashChangeListener) window.addEventListener(HashChangeEvent, handleHashChange);
        } else if (listenerCount === 0) {
            window.removeEventListener(PopStateEvent, handlePopState);
            if (needsHashChangeListener) window.removeEventListener(HashChangeEvent, handleHashChange);
        }
    }
    var isBlocked = false;
    function block(prompt) {
        if (prompt === void 0) {
            prompt = false;
        }
        var unblock = transitionManager.setPrompt(prompt);
        if (!isBlocked) {
            checkDOMListeners(1);
            isBlocked = true;
        }
        return function() {
            if (isBlocked) {
                isBlocked = false;
                checkDOMListeners(-1);
            }
            return unblock();
        };
    }
    function listen(listener) {
        var unlisten = transitionManager.appendListener(listener);
        checkDOMListeners(1);
        return function() {
            checkDOMListeners(-1);
            unlisten();
        };
    }
    var history = {
        length: globalHistory.length,
        action: 'POP',
        location: initialLocation,
        createHref: createHref,
        push: push,
        replace: replace,
        go: go,
        goBack: goBack,
        goForward: goForward,
        block: block,
        listen: listen
    };
    return history;
}
var HashChangeEvent$1 = 'hashchange';
var HashPathCoders = {
    hashbang: {
        encodePath: function encodePath(path) {
            return path.charAt(0) === '!' ? path : '!/' + stripLeadingSlash(path);
        },
        decodePath: function decodePath(path) {
            return path.charAt(0) === '!' ? path.substr(1) : path;
        }
    },
    noslash: {
        encodePath: stripLeadingSlash,
        decodePath: addLeadingSlash
    },
    slash: {
        encodePath: addLeadingSlash,
        decodePath: addLeadingSlash
    }
};
function stripHash(url) {
    var hashIndex = url.indexOf('#');
    return hashIndex === -1 ? url : url.slice(0, hashIndex);
}
function getHashPath() {
    // We can't use window.location.hash here because it's not
    // consistent across browsers - Firefox will pre-decode it!
    var href = window.location.href;
    var hashIndex = href.indexOf('#');
    return hashIndex === -1 ? '' : href.substring(hashIndex + 1);
}
function pushHashPath(path) {
    window.location.hash = path;
}
function replaceHashPath(path) {
    window.location.replace(stripHash(window.location.href) + '#' + path);
}
function createHashHistory(props) {
    if (props === void 0) {
        props = {};
    }
    !canUseDOM ? "production" !== "production" ? (0, _tinyinvariant.default)(false, 'Hash history needs a DOM') : (0, _tinyinvariant.default)(false) : void 0;
    var globalHistory = window.history;
    var canGoWithoutReload = supportsGoWithoutReloadUsingHash();
    var _props = props, _props$getUserConfirm = _props.getUserConfirmation, getUserConfirmation = _props$getUserConfirm === void 0 ? getConfirmation : _props$getUserConfirm, _props$hashType = _props.hashType, hashType = _props$hashType === void 0 ? 'slash' : _props$hashType;
    var basename = props.basename ? stripTrailingSlash(addLeadingSlash(props.basename)) : '';
    var _HashPathCoders$hashT = HashPathCoders[hashType], encodePath = _HashPathCoders$hashT.encodePath, decodePath = _HashPathCoders$hashT.decodePath;
    function getDOMLocation() {
        var path = decodePath(getHashPath());
        "production" !== "production" ? (0, _tinywarning.default)(!basename || hasBasename(path, basename), 'You are attempting to use a basename on a page whose URL path does not begin ' + 'with the basename. Expected path "' + path + '" to begin with "' + basename + '".') : void 0;
        if (basename) path = stripBasename(path, basename);
        return createLocation(path);
    }
    var transitionManager = createTransitionManager();
    function setState(nextState) {
        (0, _extends.default)(history, nextState);
        history.length = globalHistory.length;
        transitionManager.notifyListeners(history.location, history.action);
    }
    var forceNextPop = false;
    var ignorePath = null;
    function locationsAreEqual$$1(a, b) {
        return a.pathname === b.pathname && a.search === b.search && a.hash === b.hash;
    }
    function handleHashChange() {
        var path = getHashPath();
        var encodedPath = encodePath(path);
        if (path !== encodedPath) {
            // Ensure we always have a properly-encoded hash.
            replaceHashPath(encodedPath);
        } else {
            var location = getDOMLocation();
            var prevLocation = history.location;
            if (!forceNextPop && locationsAreEqual$$1(prevLocation, location)) return; // A hashchange doesn't always == location change.
            if (ignorePath === createPath(location)) return; // Ignore this change; we already setState in push/replace.
            ignorePath = null;
            handlePop(location);
        }
    }
    function handlePop(location) {
        if (forceNextPop) {
            forceNextPop = false;
            setState();
        } else {
            var action = 'POP';
            transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
                if (ok) {
                    setState({
                        action: action,
                        location: location
                    });
                } else {
                    revertPop(location);
                }
            });
        }
    }
    function revertPop(fromLocation) {
        var toLocation = history.location; // TODO: We could probably make this more reliable by
        // keeping a list of paths we've seen in sessionStorage.
        // Instead, we just default to 0 for paths we don't know.
        var toIndex = allPaths.lastIndexOf(createPath(toLocation));
        if (toIndex === -1) toIndex = 0;
        var fromIndex = allPaths.lastIndexOf(createPath(fromLocation));
        if (fromIndex === -1) fromIndex = 0;
        var delta = toIndex - fromIndex;
        if (delta) {
            forceNextPop = true;
            go(delta);
        }
    } // Ensure the hash is encoded properly before doing anything else.
    var path = getHashPath();
    var encodedPath = encodePath(path);
    if (path !== encodedPath) replaceHashPath(encodedPath);
    var initialLocation = getDOMLocation();
    var allPaths = [
        createPath(initialLocation)
    ]; // Public interface
    function createHref(location) {
        var baseTag = document.querySelector('base');
        var href = '';
        if (baseTag && baseTag.getAttribute('href')) {
            href = stripHash(window.location.href);
        }
        return href + '#' + encodePath(basename + createPath(location));
    }
    function push(path, state) {
        "production" !== "production" ? (0, _tinywarning.default)(state === undefined, 'Hash history cannot push state; it is ignored') : void 0;
        var action = 'PUSH';
        var location = createLocation(path, undefined, undefined, history.location);
        transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
            if (!ok) return;
            var path = createPath(location);
            var encodedPath = encodePath(basename + path);
            var hashChanged = getHashPath() !== encodedPath;
            if (hashChanged) {
                // We cannot tell if a hashchange was caused by a PUSH, so we'd
                // rather setState here and ignore the hashchange. The caveat here
                // is that other hash histories in the page will consider it a POP.
                ignorePath = path;
                pushHashPath(encodedPath);
                var prevIndex = allPaths.lastIndexOf(createPath(history.location));
                var nextPaths = allPaths.slice(0, prevIndex + 1);
                nextPaths.push(path);
                allPaths = nextPaths;
                setState({
                    action: action,
                    location: location
                });
            } else {
                "production" !== "production" ? (0, _tinywarning.default)(false, 'Hash history cannot PUSH the same path; a new entry will not be added to the history stack') : void 0;
                setState();
            }
        });
    }
    function replace(path, state) {
        "production" !== "production" ? (0, _tinywarning.default)(state === undefined, 'Hash history cannot replace state; it is ignored') : void 0;
        var action = 'REPLACE';
        var location = createLocation(path, undefined, undefined, history.location);
        transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
            if (!ok) return;
            var path = createPath(location);
            var encodedPath = encodePath(basename + path);
            var hashChanged = getHashPath() !== encodedPath;
            if (hashChanged) {
                // We cannot tell if a hashchange was caused by a REPLACE, so we'd
                // rather setState here and ignore the hashchange. The caveat here
                // is that other hash histories in the page will consider it a POP.
                ignorePath = path;
                replaceHashPath(encodedPath);
            }
            var prevIndex = allPaths.indexOf(createPath(history.location));
            if (prevIndex !== -1) allPaths[prevIndex] = path;
            setState({
                action: action,
                location: location
            });
        });
    }
    function go(n) {
        "production" !== "production" ? (0, _tinywarning.default)(canGoWithoutReload, 'Hash history go(n) causes a full page reload in this browser') : void 0;
        globalHistory.go(n);
    }
    function goBack() {
        go(-1);
    }
    function goForward() {
        go(1);
    }
    var listenerCount = 0;
    function checkDOMListeners(delta) {
        listenerCount += delta;
        if (listenerCount === 1 && delta === 1) {
            window.addEventListener(HashChangeEvent$1, handleHashChange);
        } else if (listenerCount === 0) {
            window.removeEventListener(HashChangeEvent$1, handleHashChange);
        }
    }
    var isBlocked = false;
    function block(prompt) {
        if (prompt === void 0) {
            prompt = false;
        }
        var unblock = transitionManager.setPrompt(prompt);
        if (!isBlocked) {
            checkDOMListeners(1);
            isBlocked = true;
        }
        return function() {
            if (isBlocked) {
                isBlocked = false;
                checkDOMListeners(-1);
            }
            return unblock();
        };
    }
    function listen(listener) {
        var unlisten = transitionManager.appendListener(listener);
        checkDOMListeners(1);
        return function() {
            checkDOMListeners(-1);
            unlisten();
        };
    }
    var history = {
        length: globalHistory.length,
        action: 'POP',
        location: initialLocation,
        createHref: createHref,
        push: push,
        replace: replace,
        go: go,
        goBack: goBack,
        goForward: goForward,
        block: block,
        listen: listen
    };
    return history;
}
function clamp(n, lowerBound, upperBound) {
    return Math.min(Math.max(n, lowerBound), upperBound);
}
/**
 * Creates a history object that stores locations in memory.
 */ function createMemoryHistory(props) {
    if (props === void 0) {
        props = {};
    }
    var _props = props, getUserConfirmation = _props.getUserConfirmation, _props$initialEntries = _props.initialEntries, initialEntries = _props$initialEntries === void 0 ? [
        '/'
    ] : _props$initialEntries, _props$initialIndex = _props.initialIndex, initialIndex = _props$initialIndex === void 0 ? 0 : _props$initialIndex, _props$keyLength = _props.keyLength, keyLength = _props$keyLength === void 0 ? 6 : _props$keyLength;
    var transitionManager = createTransitionManager();
    function setState(nextState) {
        (0, _extends.default)(history, nextState);
        history.length = history.entries.length;
        transitionManager.notifyListeners(history.location, history.action);
    }
    function createKey() {
        return Math.random().toString(36).substr(2, keyLength);
    }
    var index = clamp(initialIndex, 0, initialEntries.length - 1);
    var entries = initialEntries.map(function(entry) {
        return typeof entry === 'string' ? createLocation(entry, undefined, createKey()) : createLocation(entry, undefined, entry.key || createKey());
    }); // Public interface
    var createHref = createPath;
    function push(path, state) {
        "production" !== "production" ? (0, _tinywarning.default)(!(typeof path === 'object' && path.state !== undefined && state !== undefined), 'You should avoid providing a 2nd state argument to push when the 1st ' + 'argument is a location-like object that already has state; it is ignored') : void 0;
        var action = 'PUSH';
        var location = createLocation(path, state, createKey(), history.location);
        transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
            if (!ok) return;
            var prevIndex = history.index;
            var nextIndex = prevIndex + 1;
            var nextEntries = history.entries.slice(0);
            if (nextEntries.length > nextIndex) {
                nextEntries.splice(nextIndex, nextEntries.length - nextIndex, location);
            } else {
                nextEntries.push(location);
            }
            setState({
                action: action,
                location: location,
                index: nextIndex,
                entries: nextEntries
            });
        });
    }
    function replace(path, state) {
        "production" !== "production" ? (0, _tinywarning.default)(!(typeof path === 'object' && path.state !== undefined && state !== undefined), 'You should avoid providing a 2nd state argument to replace when the 1st ' + 'argument is a location-like object that already has state; it is ignored') : void 0;
        var action = 'REPLACE';
        var location = createLocation(path, state, createKey(), history.location);
        transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
            if (!ok) return;
            history.entries[history.index] = location;
            setState({
                action: action,
                location: location
            });
        });
    }
    function go(n) {
        var nextIndex = clamp(history.index + n, 0, history.entries.length - 1);
        var action = 'POP';
        var location = history.entries[nextIndex];
        transitionManager.confirmTransitionTo(location, action, getUserConfirmation, function(ok) {
            if (ok) {
                setState({
                    action: action,
                    location: location,
                    index: nextIndex
                });
            } else {
                // Mimic the behavior of DOM histories by
                // causing a render after a cancelled POP.
                setState();
            }
        });
    }
    function goBack() {
        go(-1);
    }
    function goForward() {
        go(1);
    }
    function canGo(n) {
        var nextIndex = history.index + n;
        return nextIndex >= 0 && nextIndex < history.entries.length;
    }
    function block(prompt) {
        if (prompt === void 0) {
            prompt = false;
        }
        return transitionManager.setPrompt(prompt);
    }
    function listen(listener) {
        return transitionManager.appendListener(listener);
    }
    var history = {
        length: entries.length,
        action: 'POP',
        location: entries[index],
        index: index,
        entries: entries,
        createHref: createHref,
        push: push,
        replace: replace,
        go: go,
        goBack: goBack,
        goForward: goForward,
        canGo: canGo,
        block: block,
        listen: listen
    };
    return history;
}

},
"e687b7f8": function(module, exports, farmRequire, farmDynamicRequire) {
/* MIT license */ var cssKeywords = farmRequire("07db3c6b", true);
// NOTE: conversions should only return primitive values (i.e. arrays, or
//       values that give correct `typeof` results).
//       do not use box values types (i.e. Number(), String(), etc.)
var reverseKeywords = {};
for(var key in cssKeywords){
    if (cssKeywords.hasOwnProperty(key)) {
        reverseKeywords[cssKeywords[key]] = key;
    }
}
var convert = module.exports = {
    rgb: {
        channels: 3,
        labels: 'rgb'
    },
    hsl: {
        channels: 3,
        labels: 'hsl'
    },
    hsv: {
        channels: 3,
        labels: 'hsv'
    },
    hwb: {
        channels: 3,
        labels: 'hwb'
    },
    cmyk: {
        channels: 4,
        labels: 'cmyk'
    },
    xyz: {
        channels: 3,
        labels: 'xyz'
    },
    lab: {
        channels: 3,
        labels: 'lab'
    },
    lch: {
        channels: 3,
        labels: 'lch'
    },
    hex: {
        channels: 1,
        labels: [
            'hex'
        ]
    },
    keyword: {
        channels: 1,
        labels: [
            'keyword'
        ]
    },
    ansi16: {
        channels: 1,
        labels: [
            'ansi16'
        ]
    },
    ansi256: {
        channels: 1,
        labels: [
            'ansi256'
        ]
    },
    hcg: {
        channels: 3,
        labels: [
            'h',
            'c',
            'g'
        ]
    },
    apple: {
        channels: 3,
        labels: [
            'r16',
            'g16',
            'b16'
        ]
    },
    gray: {
        channels: 1,
        labels: [
            'gray'
        ]
    }
};
// hide .channels and .labels properties
for(var model in convert){
    if (convert.hasOwnProperty(model)) {
        if (!('channels' in convert[model])) {
            throw new Error('missing channels property: ' + model);
        }
        if (!('labels' in convert[model])) {
            throw new Error('missing channel labels property: ' + model);
        }
        if (convert[model].labels.length !== convert[model].channels) {
            throw new Error('channel and label counts mismatch: ' + model);
        }
        var channels = convert[model].channels;
        var labels = convert[model].labels;
        delete convert[model].channels;
        delete convert[model].labels;
        Object.defineProperty(convert[model], 'channels', {
            value: channels
        });
        Object.defineProperty(convert[model], 'labels', {
            value: labels
        });
    }
}
convert.rgb.hsl = function(rgb) {
    var r = rgb[0] / 255;
    var g = rgb[1] / 255;
    var b = rgb[2] / 255;
    var min = Math.min(r, g, b);
    var max = Math.max(r, g, b);
    var delta = max - min;
    var h;
    var s;
    var l;
    if (max === min) {
        h = 0;
    } else if (r === max) {
        h = (g - b) / delta;
    } else if (g === max) {
        h = 2 + (b - r) / delta;
    } else if (b === max) {
        h = 4 + (r - g) / delta;
    }
    h = Math.min(h * 60, 360);
    if (h < 0) {
        h += 360;
    }
    l = (min + max) / 2;
    if (max === min) {
        s = 0;
    } else if (l <= 0.5) {
        s = delta / (max + min);
    } else {
        s = delta / (2 - max - min);
    }
    return [
        h,
        s * 100,
        l * 100
    ];
};
convert.rgb.hsv = function(rgb) {
    var rdif;
    var gdif;
    var bdif;
    var h;
    var s;
    var r = rgb[0] / 255;
    var g = rgb[1] / 255;
    var b = rgb[2] / 255;
    var v = Math.max(r, g, b);
    var diff = v - Math.min(r, g, b);
    var diffc = function(c) {
        return (v - c) / 6 / diff + 1 / 2;
    };
    if (diff === 0) {
        h = s = 0;
    } else {
        s = diff / v;
        rdif = diffc(r);
        gdif = diffc(g);
        bdif = diffc(b);
        if (r === v) {
            h = bdif - gdif;
        } else if (g === v) {
            h = 1 / 3 + rdif - bdif;
        } else if (b === v) {
            h = 2 / 3 + gdif - rdif;
        }
        if (h < 0) {
            h += 1;
        } else if (h > 1) {
            h -= 1;
        }
    }
    return [
        h * 360,
        s * 100,
        v * 100
    ];
};
convert.rgb.hwb = function(rgb) {
    var r = rgb[0];
    var g = rgb[1];
    var b = rgb[2];
    var h = convert.rgb.hsl(rgb)[0];
    var w = 1 / 255 * Math.min(r, Math.min(g, b));
    b = 1 - 1 / 255 * Math.max(r, Math.max(g, b));
    return [
        h,
        w * 100,
        b * 100
    ];
};
convert.rgb.cmyk = function(rgb) {
    var r = rgb[0] / 255;
    var g = rgb[1] / 255;
    var b = rgb[2] / 255;
    var c;
    var m;
    var y;
    var k;
    k = Math.min(1 - r, 1 - g, 1 - b);
    c = (1 - r - k) / (1 - k) || 0;
    m = (1 - g - k) / (1 - k) || 0;
    y = (1 - b - k) / (1 - k) || 0;
    return [
        c * 100,
        m * 100,
        y * 100,
        k * 100
    ];
};
/**
 * See https://en.m.wikipedia.org/wiki/Euclidean_distance#Squared_Euclidean_distance
 * */ function comparativeDistance(x, y) {
    return Math.pow(x[0] - y[0], 2) + Math.pow(x[1] - y[1], 2) + Math.pow(x[2] - y[2], 2);
}
convert.rgb.keyword = function(rgb) {
    var reversed = reverseKeywords[rgb];
    if (reversed) {
        return reversed;
    }
    var currentClosestDistance = Infinity;
    var currentClosestKeyword;
    for(var keyword in cssKeywords){
        if (cssKeywords.hasOwnProperty(keyword)) {
            var value = cssKeywords[keyword];
            // Compute comparative distance
            var distance = comparativeDistance(rgb, value);
            // Check if its less, if so set as closest
            if (distance < currentClosestDistance) {
                currentClosestDistance = distance;
                currentClosestKeyword = keyword;
            }
        }
    }
    return currentClosestKeyword;
};
convert.keyword.rgb = function(keyword) {
    return cssKeywords[keyword];
};
convert.rgb.xyz = function(rgb) {
    var r = rgb[0] / 255;
    var g = rgb[1] / 255;
    var b = rgb[2] / 255;
    // assume sRGB
    r = r > 0.04045 ? Math.pow((r + 0.055) / 1.055, 2.4) : r / 12.92;
    g = g > 0.04045 ? Math.pow((g + 0.055) / 1.055, 2.4) : g / 12.92;
    b = b > 0.04045 ? Math.pow((b + 0.055) / 1.055, 2.4) : b / 12.92;
    var x = r * 0.4124 + g * 0.3576 + b * 0.1805;
    var y = r * 0.2126 + g * 0.7152 + b * 0.0722;
    var z = r * 0.0193 + g * 0.1192 + b * 0.9505;
    return [
        x * 100,
        y * 100,
        z * 100
    ];
};
convert.rgb.lab = function(rgb) {
    var xyz = convert.rgb.xyz(rgb);
    var x = xyz[0];
    var y = xyz[1];
    var z = xyz[2];
    var l;
    var a;
    var b;
    x /= 95.047;
    y /= 100;
    z /= 108.883;
    x = x > 0.008856 ? Math.pow(x, 1 / 3) : 7.787 * x + 16 / 116;
    y = y > 0.008856 ? Math.pow(y, 1 / 3) : 7.787 * y + 16 / 116;
    z = z > 0.008856 ? Math.pow(z, 1 / 3) : 7.787 * z + 16 / 116;
    l = 116 * y - 16;
    a = 500 * (x - y);
    b = 200 * (y - z);
    return [
        l,
        a,
        b
    ];
};
convert.hsl.rgb = function(hsl) {
    var h = hsl[0] / 360;
    var s = hsl[1] / 100;
    var l = hsl[2] / 100;
    var t1;
    var t2;
    var t3;
    var rgb;
    var val;
    if (s === 0) {
        val = l * 255;
        return [
            val,
            val,
            val
        ];
    }
    if (l < 0.5) {
        t2 = l * (1 + s);
    } else {
        t2 = l + s - l * s;
    }
    t1 = 2 * l - t2;
    rgb = [
        0,
        0,
        0
    ];
    for(var i = 0; i < 3; i++){
        t3 = h + 1 / 3 * -(i - 1);
        if (t3 < 0) {
            t3++;
        }
        if (t3 > 1) {
            t3--;
        }
        if (6 * t3 < 1) {
            val = t1 + (t2 - t1) * 6 * t3;
        } else if (2 * t3 < 1) {
            val = t2;
        } else if (3 * t3 < 2) {
            val = t1 + (t2 - t1) * (2 / 3 - t3) * 6;
        } else {
            val = t1;
        }
        rgb[i] = val * 255;
    }
    return rgb;
};
convert.hsl.hsv = function(hsl) {
    var h = hsl[0];
    var s = hsl[1] / 100;
    var l = hsl[2] / 100;
    var smin = s;
    var lmin = Math.max(l, 0.01);
    var sv;
    var v;
    l *= 2;
    s *= l <= 1 ? l : 2 - l;
    smin *= lmin <= 1 ? lmin : 2 - lmin;
    v = (l + s) / 2;
    sv = l === 0 ? 2 * smin / (lmin + smin) : 2 * s / (l + s);
    return [
        h,
        sv * 100,
        v * 100
    ];
};
convert.hsv.rgb = function(hsv) {
    var h = hsv[0] / 60;
    var s = hsv[1] / 100;
    var v = hsv[2] / 100;
    var hi = Math.floor(h) % 6;
    var f = h - Math.floor(h);
    var p = 255 * v * (1 - s);
    var q = 255 * v * (1 - s * f);
    var t = 255 * v * (1 - s * (1 - f));
    v *= 255;
    switch(hi){
        case 0:
            return [
                v,
                t,
                p
            ];
        case 1:
            return [
                q,
                v,
                p
            ];
        case 2:
            return [
                p,
                v,
                t
            ];
        case 3:
            return [
                p,
                q,
                v
            ];
        case 4:
            return [
                t,
                p,
                v
            ];
        case 5:
            return [
                v,
                p,
                q
            ];
    }
};
convert.hsv.hsl = function(hsv) {
    var h = hsv[0];
    var s = hsv[1] / 100;
    var v = hsv[2] / 100;
    var vmin = Math.max(v, 0.01);
    var lmin;
    var sl;
    var l;
    l = (2 - s) * v;
    lmin = (2 - s) * vmin;
    sl = s * vmin;
    sl /= lmin <= 1 ? lmin : 2 - lmin;
    sl = sl || 0;
    l /= 2;
    return [
        h,
        sl * 100,
        l * 100
    ];
};
// http://dev.w3.org/csswg/css-color/#hwb-to-rgb
convert.hwb.rgb = function(hwb) {
    var h = hwb[0] / 360;
    var wh = hwb[1] / 100;
    var bl = hwb[2] / 100;
    var ratio = wh + bl;
    var i;
    var v;
    var f;
    var n;
    // wh + bl cant be > 1
    if (ratio > 1) {
        wh /= ratio;
        bl /= ratio;
    }
    i = Math.floor(6 * h);
    v = 1 - bl;
    f = 6 * h - i;
    if ((i & 0x01) !== 0) {
        f = 1 - f;
    }
    n = wh + f * (v - wh); // linear interpolation
    var r;
    var g;
    var b;
    switch(i){
        default:
        case 6:
        case 0:
            r = v;
            g = n;
            b = wh;
            break;
        case 1:
            r = n;
            g = v;
            b = wh;
            break;
        case 2:
            r = wh;
            g = v;
            b = n;
            break;
        case 3:
            r = wh;
            g = n;
            b = v;
            break;
        case 4:
            r = n;
            g = wh;
            b = v;
            break;
        case 5:
            r = v;
            g = wh;
            b = n;
            break;
    }
    return [
        r * 255,
        g * 255,
        b * 255
    ];
};
convert.cmyk.rgb = function(cmyk) {
    var c = cmyk[0] / 100;
    var m = cmyk[1] / 100;
    var y = cmyk[2] / 100;
    var k = cmyk[3] / 100;
    var r;
    var g;
    var b;
    r = 1 - Math.min(1, c * (1 - k) + k);
    g = 1 - Math.min(1, m * (1 - k) + k);
    b = 1 - Math.min(1, y * (1 - k) + k);
    return [
        r * 255,
        g * 255,
        b * 255
    ];
};
convert.xyz.rgb = function(xyz) {
    var x = xyz[0] / 100;
    var y = xyz[1] / 100;
    var z = xyz[2] / 100;
    var r;
    var g;
    var b;
    r = x * 3.2406 + y * -1.5372 + z * -0.4986;
    g = x * -0.9689 + y * 1.8758 + z * 0.0415;
    b = x * 0.0557 + y * -0.2040 + z * 1.0570;
    // assume sRGB
    r = r > 0.0031308 ? 1.055 * Math.pow(r, 1.0 / 2.4) - 0.055 : r * 12.92;
    g = g > 0.0031308 ? 1.055 * Math.pow(g, 1.0 / 2.4) - 0.055 : g * 12.92;
    b = b > 0.0031308 ? 1.055 * Math.pow(b, 1.0 / 2.4) - 0.055 : b * 12.92;
    r = Math.min(Math.max(0, r), 1);
    g = Math.min(Math.max(0, g), 1);
    b = Math.min(Math.max(0, b), 1);
    return [
        r * 255,
        g * 255,
        b * 255
    ];
};
convert.xyz.lab = function(xyz) {
    var x = xyz[0];
    var y = xyz[1];
    var z = xyz[2];
    var l;
    var a;
    var b;
    x /= 95.047;
    y /= 100;
    z /= 108.883;
    x = x > 0.008856 ? Math.pow(x, 1 / 3) : 7.787 * x + 16 / 116;
    y = y > 0.008856 ? Math.pow(y, 1 / 3) : 7.787 * y + 16 / 116;
    z = z > 0.008856 ? Math.pow(z, 1 / 3) : 7.787 * z + 16 / 116;
    l = 116 * y - 16;
    a = 500 * (x - y);
    b = 200 * (y - z);
    return [
        l,
        a,
        b
    ];
};
convert.lab.xyz = function(lab) {
    var l = lab[0];
    var a = lab[1];
    var b = lab[2];
    var x;
    var y;
    var z;
    y = (l + 16) / 116;
    x = a / 500 + y;
    z = y - b / 200;
    var y2 = Math.pow(y, 3);
    var x2 = Math.pow(x, 3);
    var z2 = Math.pow(z, 3);
    y = y2 > 0.008856 ? y2 : (y - 16 / 116) / 7.787;
    x = x2 > 0.008856 ? x2 : (x - 16 / 116) / 7.787;
    z = z2 > 0.008856 ? z2 : (z - 16 / 116) / 7.787;
    x *= 95.047;
    y *= 100;
    z *= 108.883;
    return [
        x,
        y,
        z
    ];
};
convert.lab.lch = function(lab) {
    var l = lab[0];
    var a = lab[1];
    var b = lab[2];
    var hr;
    var h;
    var c;
    hr = Math.atan2(b, a);
    h = hr * 360 / 2 / Math.PI;
    if (h < 0) {
        h += 360;
    }
    c = Math.sqrt(a * a + b * b);
    return [
        l,
        c,
        h
    ];
};
convert.lch.lab = function(lch) {
    var l = lch[0];
    var c = lch[1];
    var h = lch[2];
    var a;
    var b;
    var hr;
    hr = h / 360 * 2 * Math.PI;
    a = c * Math.cos(hr);
    b = c * Math.sin(hr);
    return [
        l,
        a,
        b
    ];
};
convert.rgb.ansi16 = function(args) {
    var r = args[0];
    var g = args[1];
    var b = args[2];
    var value = 1 in arguments ? arguments[1] : convert.rgb.hsv(args)[2]; // hsv -> ansi16 optimization
    value = Math.round(value / 50);
    if (value === 0) {
        return 30;
    }
    var ansi = 30 + (Math.round(b / 255) << 2 | Math.round(g / 255) << 1 | Math.round(r / 255));
    if (value === 2) {
        ansi += 60;
    }
    return ansi;
};
convert.hsv.ansi16 = function(args) {
    // optimization here; we already know the value and don't need to get
    // it converted for us.
    return convert.rgb.ansi16(convert.hsv.rgb(args), args[2]);
};
convert.rgb.ansi256 = function(args) {
    var r = args[0];
    var g = args[1];
    var b = args[2];
    // we use the extended greyscale palette here, with the exception of
    // black and white. normal palette only has 4 greyscale shades.
    if (r === g && g === b) {
        if (r < 8) {
            return 16;
        }
        if (r > 248) {
            return 231;
        }
        return Math.round((r - 8) / 247 * 24) + 232;
    }
    var ansi = 16 + 36 * Math.round(r / 255 * 5) + 6 * Math.round(g / 255 * 5) + Math.round(b / 255 * 5);
    return ansi;
};
convert.ansi16.rgb = function(args) {
    var color = args % 10;
    // handle greyscale
    if (color === 0 || color === 7) {
        if (args > 50) {
            color += 3.5;
        }
        color = color / 10.5 * 255;
        return [
            color,
            color,
            color
        ];
    }
    var mult = (~~(args > 50) + 1) * 0.5;
    var r = (color & 1) * mult * 255;
    var g = (color >> 1 & 1) * mult * 255;
    var b = (color >> 2 & 1) * mult * 255;
    return [
        r,
        g,
        b
    ];
};
convert.ansi256.rgb = function(args) {
    // handle greyscale
    if (args >= 232) {
        var c = (args - 232) * 10 + 8;
        return [
            c,
            c,
            c
        ];
    }
    args -= 16;
    var rem;
    var r = Math.floor(args / 36) / 5 * 255;
    var g = Math.floor((rem = args % 36) / 6) / 5 * 255;
    var b = rem % 6 / 5 * 255;
    return [
        r,
        g,
        b
    ];
};
convert.rgb.hex = function(args) {
    var integer = ((Math.round(args[0]) & 0xFF) << 16) + ((Math.round(args[1]) & 0xFF) << 8) + (Math.round(args[2]) & 0xFF);
    var string = integer.toString(16).toUpperCase();
    return '000000'.substring(string.length) + string;
};
convert.hex.rgb = function(args) {
    var match = args.toString(16).match(/[a-f0-9]{6}|[a-f0-9]{3}/i);
    if (!match) {
        return [
            0,
            0,
            0
        ];
    }
    var colorString = match[0];
    if (match[0].length === 3) {
        colorString = colorString.split('').map(function(char) {
            return char + char;
        }).join('');
    }
    var integer = parseInt(colorString, 16);
    var r = integer >> 16 & 0xFF;
    var g = integer >> 8 & 0xFF;
    var b = integer & 0xFF;
    return [
        r,
        g,
        b
    ];
};
convert.rgb.hcg = function(rgb) {
    var r = rgb[0] / 255;
    var g = rgb[1] / 255;
    var b = rgb[2] / 255;
    var max = Math.max(Math.max(r, g), b);
    var min = Math.min(Math.min(r, g), b);
    var chroma = max - min;
    var grayscale;
    var hue;
    if (chroma < 1) {
        grayscale = min / (1 - chroma);
    } else {
        grayscale = 0;
    }
    if (chroma <= 0) {
        hue = 0;
    } else if (max === r) {
        hue = (g - b) / chroma % 6;
    } else if (max === g) {
        hue = 2 + (b - r) / chroma;
    } else {
        hue = 4 + (r - g) / chroma + 4;
    }
    hue /= 6;
    hue %= 1;
    return [
        hue * 360,
        chroma * 100,
        grayscale * 100
    ];
};
convert.hsl.hcg = function(hsl) {
    var s = hsl[1] / 100;
    var l = hsl[2] / 100;
    var c = 1;
    var f = 0;
    if (l < 0.5) {
        c = 2.0 * s * l;
    } else {
        c = 2.0 * s * (1.0 - l);
    }
    if (c < 1.0) {
        f = (l - 0.5 * c) / (1.0 - c);
    }
    return [
        hsl[0],
        c * 100,
        f * 100
    ];
};
convert.hsv.hcg = function(hsv) {
    var s = hsv[1] / 100;
    var v = hsv[2] / 100;
    var c = s * v;
    var f = 0;
    if (c < 1.0) {
        f = (v - c) / (1 - c);
    }
    return [
        hsv[0],
        c * 100,
        f * 100
    ];
};
convert.hcg.rgb = function(hcg) {
    var h = hcg[0] / 360;
    var c = hcg[1] / 100;
    var g = hcg[2] / 100;
    if (c === 0.0) {
        return [
            g * 255,
            g * 255,
            g * 255
        ];
    }
    var pure = [
        0,
        0,
        0
    ];
    var hi = h % 1 * 6;
    var v = hi % 1;
    var w = 1 - v;
    var mg = 0;
    switch(Math.floor(hi)){
        case 0:
            pure[0] = 1;
            pure[1] = v;
            pure[2] = 0;
            break;
        case 1:
            pure[0] = w;
            pure[1] = 1;
            pure[2] = 0;
            break;
        case 2:
            pure[0] = 0;
            pure[1] = 1;
            pure[2] = v;
            break;
        case 3:
            pure[0] = 0;
            pure[1] = w;
            pure[2] = 1;
            break;
        case 4:
            pure[0] = v;
            pure[1] = 0;
            pure[2] = 1;
            break;
        default:
            pure[0] = 1;
            pure[1] = 0;
            pure[2] = w;
    }
    mg = (1.0 - c) * g;
    return [
        (c * pure[0] + mg) * 255,
        (c * pure[1] + mg) * 255,
        (c * pure[2] + mg) * 255
    ];
};
convert.hcg.hsv = function(hcg) {
    var c = hcg[1] / 100;
    var g = hcg[2] / 100;
    var v = c + g * (1.0 - c);
    var f = 0;
    if (v > 0.0) {
        f = c / v;
    }
    return [
        hcg[0],
        f * 100,
        v * 100
    ];
};
convert.hcg.hsl = function(hcg) {
    var c = hcg[1] / 100;
    var g = hcg[2] / 100;
    var l = g * (1.0 - c) + 0.5 * c;
    var s = 0;
    if (l > 0.0 && l < 0.5) {
        s = c / (2 * l);
    } else if (l >= 0.5 && l < 1.0) {
        s = c / (2 * (1 - l));
    }
    return [
        hcg[0],
        s * 100,
        l * 100
    ];
};
convert.hcg.hwb = function(hcg) {
    var c = hcg[1] / 100;
    var g = hcg[2] / 100;
    var v = c + g * (1.0 - c);
    return [
        hcg[0],
        (v - c) * 100,
        (1 - v) * 100
    ];
};
convert.hwb.hcg = function(hwb) {
    var w = hwb[1] / 100;
    var b = hwb[2] / 100;
    var v = 1 - b;
    var c = v - w;
    var g = 0;
    if (c < 1) {
        g = (v - c) / (1 - c);
    }
    return [
        hwb[0],
        c * 100,
        g * 100
    ];
};
convert.apple.rgb = function(apple) {
    return [
        apple[0] / 65535 * 255,
        apple[1] / 65535 * 255,
        apple[2] / 65535 * 255
    ];
};
convert.rgb.apple = function(rgb) {
    return [
        rgb[0] / 255 * 65535,
        rgb[1] / 255 * 65535,
        rgb[2] / 255 * 65535
    ];
};
convert.gray.rgb = function(args) {
    return [
        args[0] / 100 * 255,
        args[0] / 100 * 255,
        args[0] / 100 * 255
    ];
};
convert.gray.hsl = convert.gray.hsv = function(args) {
    return [
        0,
        0,
        args[0]
    ];
};
convert.gray.hwb = function(gray) {
    return [
        0,
        100,
        gray[0]
    ];
};
convert.gray.cmyk = function(gray) {
    return [
        0,
        0,
        0,
        gray[0]
    ];
};
convert.gray.lab = function(gray) {
    return [
        gray[0],
        0,
        0
    ];
};
convert.gray.hex = function(gray) {
    var val = Math.round(gray[0] / 100 * 255) & 0xFF;
    var integer = (val << 16) + (val << 8) + val;
    var string = integer.toString(16).toUpperCase();
    return '000000'.substring(string.length) + string;
};
convert.rgb.gray = function(rgb) {
    var val = (rgb[0] + rgb[1] + rgb[2]) / 3;
    return [
        val / 255 * 100
    ];
};

},
"f14759ef": function(module, exports, farmRequire, farmDynamicRequire) {
var conversions = farmRequire("e687b7f8", true);
var route = farmRequire("bcd6de30", true);
var convert = {};
var models = Object.keys(conversions);
function wrapRaw(fn) {
    var wrappedFn = function(args) {
        if (args === undefined || args === null) {
            return args;
        }
        if (arguments.length > 1) {
            args = Array.prototype.slice.call(arguments);
        }
        return fn(args);
    };
    // preserve .conversion property if there is one
    if ('conversion' in fn) {
        wrappedFn.conversion = fn.conversion;
    }
    return wrappedFn;
}
function wrapRounded(fn) {
    var wrappedFn = function(args) {
        if (args === undefined || args === null) {
            return args;
        }
        if (arguments.length > 1) {
            args = Array.prototype.slice.call(arguments);
        }
        var result = fn(args);
        // we're assuming the result is an array here.
        // see notice in conversions.js; don't use box types
        // in conversion functions.
        if (typeof result === 'object') {
            for(var len = result.length, i = 0; i < len; i++){
                result[i] = Math.round(result[i]);
            }
        }
        return result;
    };
    // preserve .conversion property if there is one
    if ('conversion' in fn) {
        wrappedFn.conversion = fn.conversion;
    }
    return wrappedFn;
}
models.forEach(function(fromModel) {
    convert[fromModel] = {};
    Object.defineProperty(convert[fromModel], 'channels', {
        value: conversions[fromModel].channels
    });
    Object.defineProperty(convert[fromModel], 'labels', {
        value: conversions[fromModel].labels
    });
    var routes = route(fromModel);
    var routeModels = Object.keys(routes);
    routeModels.forEach(function(toModel) {
        var fn = routes[toModel];
        convert[fromModel][toModel] = wrapRounded(fn);
        convert[fromModel][toModel].raw = wrapRaw(fn);
    });
});
module.exports = convert;

},
"fc636423": function(module, exports, farmRequire, farmDynamicRequire) {
var baseIsMatch = farmRequire("d30c21d7", true), getMatchData = farmRequire("c894d30e", true), matchesStrictComparable = farmRequire("1e09a1df", true);
/**
 * The base implementation of `_.matches` which doesn't clone `source`.
 *
 * @private
 * @param {Object} source The object of property values to match.
 * @returns {Function} Returns the new spec function.
 */ function baseMatches(source) {
    var matchData = getMatchData(source);
    if (matchData.length == 1 && matchData[0][2]) {
        return matchesStrictComparable(matchData[0][0], matchData[0][1]);
    }
    return function(object) {
        return object === source || baseIsMatch(object, source, matchData);
    };
}
module.exports = baseMatches;

},});