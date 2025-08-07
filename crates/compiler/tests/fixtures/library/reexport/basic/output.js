//index.js:
 var foo = "foo" + 1;
var b_js_namespace_farm_internal_ = {
    get a () {
        return a_js_namespace_farm_internal_;
    },
    get bar () {
        return bar;
    },
    get baz () {
        return d_js_default;
    },
    get f () {
        return f;
    },
    get f_renamed () {
        return f_local;
    },
    get foo () {
        return foo;
    },
    get ns () {
        return e_js_namespace_farm_internal_;
    },
    __esModule: true
};
var e = 4;
var e_js_default = function() {
    console.log("e" + e);
};
var e_js_namespace_farm_internal_ = {
    default: e_js_default,
    e: e,
    __esModule: true
};
var d_js_default = 3;
var d_js_namespace_farm_internal_ = {
    default: d_js_default,
    e: e,
    __esModule: true
};
var bar = "bar" + 2;
var f = 5;
var f_local = "f";
var f_d = "f_d";
var a_js_namespace_farm_internal_ = {
    get a () {
        return a_js_namespace_farm_internal_;
    },
    get bar () {
        return bar;
    },
    get baz () {
        return d_js_default;
    },
    get f () {
        return f;
    },
    get f_renamed () {
        return f_local;
    },
    get foo () {
        return foo;
    },
    get ns () {
        return e_js_namespace_farm_internal_;
    },
    __esModule: true
};
console.log(Object.entries(a_js_namespace_farm_internal_), Object.entries(b_js_namespace_farm_internal_));
export { a_js_namespace_farm_internal_ as a, b_js_namespace_farm_internal_ as b, bar as bar, d_js_default as baz, f as f, f_local as f_renamed, foo as foo, e_js_namespace_farm_internal_ as ns };
