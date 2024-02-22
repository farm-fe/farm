(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_6ec8.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"1cb2548c": function(module, exports, farmRequire, farmDynamicRequire) {
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
    isArray: function() {
        return isArray;
    },
    isSSR: function() {
        return isSSR;
    }
});
function isArray(val) {
    return Object.prototype.toString.call(val) === "[object Array]";
}
const isSSR = function() {
    try {
        return !(typeof window !== "undefined" && document !== undefined);
    } catch (e) {
        return true;
    }
}();

},
"40a43159": function(module, exports, farmRequire, farmDynamicRequire) {
// https://stackoverflow.com/questions/68424114/next-js-how-to-fetch-localstorage-data-before-client-side-rendering
// 解决 nextJS 无法获取初始localstorage问题
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
const _is = farmRequire("1cb2548c");
const _react = farmRequire("a0fc9dfd");
const getDefaultStorage = (key)=>{
    if (!_is.isSSR) {
        return localStorage.getItem(key);
    } else {
        return undefined;
    }
};
function useStorage(key, defaultValue) {
    const [storedValue, setStoredValue] = (0, _react.useState)(getDefaultStorage(key) || defaultValue);
    const setStorageValue = (value)=>{
        if (!_is.isSSR) {
            localStorage.setItem(key, value);
            if (value !== storedValue) {
                setStoredValue(value);
            }
        }
    };
    const removeStorage = ()=>{
        if (!_is.isSSR) {
            localStorage.removeItem("key");
        }
    };
    (0, _react.useEffect)(()=>{
        const storageValue = localStorage.getItem(key);
        if (storageValue) {
            setStoredValue(storageValue);
        }
    }, []);
    return [
        storedValue,
        setStorageValue,
        removeStorage
    ];
}
const _default = useStorage;

},
"4dbeb4c8": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "login.form.title": "Login to Arco Design Pro",
        "login.form.userName.errMsg": "Username cannot be empty",
        "login.form.password.errMsg": "Password cannot be empty",
        "login.form.login.errMsg": "Login error, refresh and try again",
        "login.form.userName.placeholder": "Username: admin",
        "login.form.password.placeholder": "Password: admin",
        "login.form.rememberPassword": "Remember password",
        "login.form.forgetPassword": "Forgot password",
        "login.form.login": "login",
        "login.form.register": "register account",
        "login.banner.slogan1": "Out-of-the-box high-quality template",
        "login.banner.subSlogan1": "Rich page templates, covering most typical business scenarios",
        "login.banner.slogan2": "Built-in solutions to common problems",
        "login.banner.subSlogan2": "Internationalization, routing configuration, state management everything",
        "login.banner.slogan3": "Access visualization enhancement tool AUX",
        "login.banner.subSlogan3": "Realize flexible block development"
    },
    "zh-CN": {
        "login.form.title": "登录 Arco Design Pro",
        "login.form.userName.errMsg": "用户名不能为空",
        "login.form.password.errMsg": "密码不能为空",
        "login.form.login.errMsg": "登录出错，轻刷新重试",
        "login.form.userName.placeholder": "用户名：admin",
        "login.form.password.placeholder": "密码：admin",
        "login.form.rememberPassword": "记住密码",
        "login.form.forgetPassword": "忘记密码",
        "login.form.login": "登录",
        "login.form.register": "注册账号",
        "login.banner.slogan1": "开箱即用的高质量模板",
        "login.banner.subSlogan1": "丰富的的页面模板，覆盖大多数典型业务场景",
        "login.banner.slogan2": "内置了常见问题的解决方案",
        "login.banner.subSlogan2": "国际化，路由配置，状态管理应有尽有",
        "login.banner.slogan3": "接入可视化增强工具AUX",
        "login.banner.subSlogan3": "实现灵活的区块式开发"
    }
};
const _default = i18n;

},
"534a00b8": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return LoginForm;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _useStorage = /*#__PURE__*/ _interop_require_default._(farmRequire("40a43159"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("4dbeb4c8"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("8ac200a7"));
function LoginForm() {
    const formRef = (0, _react.useRef)();
    const [errorMessage, setErrorMessage] = (0, _react.useState)('');
    const [loading, setLoading] = (0, _react.useState)(false);
    const [loginParams, setLoginParams, removeLoginParams] = (0, _useStorage.default)('loginParams');
    const t = (0, _useLocale.default)(_locale.default);
    const [rememberPassword, setRememberPassword] = (0, _react.useState)(!!loginParams);
    function afterLoginSuccess(params) {
        // 记住密码
        if (rememberPassword) {
            setLoginParams(JSON.stringify(params));
        } else {
            removeLoginParams();
        }
        // 记录登录状态
        localStorage.setItem('userStatus', 'login');
        // 跳转首页
        window.location.href = '/';
    }
    function login(params) {
        setErrorMessage('');
        setLoading(true);
        _axios.default.post('/api/user/login', params).then((res)=>{
            const { status, msg } = res.data;
            if (status === 'ok') {
                afterLoginSuccess(params);
            } else {
                setErrorMessage(msg || t['login.form.login.errMsg']);
            }
        }).finally(()=>{
            setLoading(false);
        });
    }
    function onSubmitClick() {
        formRef.current.validate().then((values)=>{
            login(values);
        });
    }
    // 读取 localStorage，设置初始值
    (0, _react.useEffect)(()=>{
        const rememberPassword = !!loginParams;
        setRememberPassword(rememberPassword);
        if (formRef.current && rememberPassword) {
            const parseParams = JSON.parse(loginParams);
            formRef.current.setFieldsValue(parseParams);
        }
    }, [
        loginParams
    ]);
    return _react.default.createElement("div", {
        className: _indexmoduleless.default['login-form-wrapper']
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['login-form-title']
    }, t['login.form.title']), _react.default.createElement("div", {
        className: _indexmoduleless.default['login-form-sub-title']
    }, t['login.form.title']), _react.default.createElement("div", {
        className: _indexmoduleless.default['login-form-error-msg']
    }, errorMessage), _react.default.createElement(_webreact.Form, {
        className: _indexmoduleless.default['login-form'],
        layout: "vertical",
        ref: formRef,
        initialValues: {
            userName: 'admin',
            password: 'admin'
        }
    }, _react.default.createElement(_webreact.Form.Item, {
        field: "userName",
        rules: [
            {
                required: true,
                message: t['login.form.userName.errMsg']
            }
        ]
    }, _react.default.createElement(_webreact.Input, {
        prefix: _react.default.createElement(_icon.IconUser, null),
        placeholder: t['login.form.userName.placeholder'],
        onPressEnter: onSubmitClick
    })), _react.default.createElement(_webreact.Form.Item, {
        field: "password",
        rules: [
            {
                required: true,
                message: t['login.form.password.errMsg']
            }
        ]
    }, _react.default.createElement(_webreact.Input.Password, {
        prefix: _react.default.createElement(_icon.IconLock, null),
        placeholder: t['login.form.password.placeholder'],
        onPressEnter: onSubmitClick
    })), _react.default.createElement(_webreact.Space, {
        size: 16,
        direction: "vertical"
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['login-form-password-actions']
    }, _react.default.createElement(_webreact.Checkbox, {
        checked: rememberPassword,
        onChange: setRememberPassword
    }, t['login.form.rememberPassword']), _react.default.createElement(_webreact.Link, null, t['login.form.forgetPassword'])), _react.default.createElement(_webreact.Button, {
        type: "primary",
        long: true,
        onClick: onSubmitClick,
        loading: loading
    }, t['login.form.login']), _react.default.createElement(_webreact.Button, {
        type: "text",
        long: true,
        className: _indexmoduleless.default['login-form-register-btn']
    }, t['login.form.register']))));
}

},
"8ac200a7": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "banner": `banner-9fb39238`,
    "banner-inner": `banner-inner-9fb39238`,
    "carousel": `carousel-9fb39238`,
    "carousel-image": `carousel-image-9fb39238`,
    "carousel-item": `carousel-item-9fb39238`,
    "carousel-sub-title": `carousel-sub-title-9fb39238`,
    "carousel-title": `carousel-title-9fb39238`,
    "container": `container-9fb39238`,
    "content": `content-9fb39238`,
    "footer": `footer-9fb39238`,
    "login-form-error-msg": `login-form-error-msg-9fb39238`,
    "login-form-password-actions": `login-form-password-actions-9fb39238`,
    "login-form-register-btn": `login-form-register-btn-9fb39238`,
    "login-form-sub-title": `login-form-sub-title-9fb39238`,
    "login-form-title": `login-form-title-9fb39238`,
    "login-form-wrapper": `login-form-wrapper-9fb39238`,
    "logo": `logo-9fb39238`,
    "logo-text": `logo-text-9fb39238`
};

},
"96ecd90f": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _logosvg = /*#__PURE__*/ _interop_require_default._(farmRequire("c83e07be"));
const _Footer = /*#__PURE__*/ _interop_require_default._(farmRequire("d8012317"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _banner = /*#__PURE__*/ _interop_require_default._(farmRequire("efe06641"));
const _form = /*#__PURE__*/ _interop_require_default._(farmRequire("534a00b8"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("8ac200a7"));
function Login() {
    (0, _react.useEffect)(()=>{
        document.body.setAttribute('arco-theme', 'light');
    }, []);
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.container
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default.logo
    }, _react.default.createElement(_logosvg.default, null), _react.default.createElement("div", {
        className: _indexmoduleless.default['logo-text']
    }, "Arco Design Pro")), _react.default.createElement("div", {
        className: _indexmoduleless.default.banner
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['banner-inner']
    }, _react.default.createElement(_banner.default, null))), _react.default.createElement("div", {
        className: _indexmoduleless.default.content
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default['content-inner']
    }, _react.default.createElement(_form.default, null)), _react.default.createElement("div", {
        className: _indexmoduleless.default.footer
    }, _react.default.createElement(_Footer.default, null))));
}
Login.displayName = 'LoginPage';
const _default = Login;

},
"c83e07be": function(module, exports, farmRequire, farmDynamicRequire) {
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
const SvgLogo = (props)=>_react.createElement("svg", {
        width: 33,
        height: 33,
        viewBox: "0 0 33 33",
        fill: "none",
        xmlns: "http://www.w3.org/2000/svg",
        ...props
    }, _react.createElement("g", {
        clipPath: "url(#clip0)"
    }, _react.createElement("path", {
        fillRule: "evenodd",
        clipRule: "evenodd",
        d: "M5.37754 16.9795L12.7498 9.43027C14.7163 7.41663 17.9428 7.37837 19.9564 9.34482C19.9852 9.37297 20.0137 9.40145 20.0418 9.43027L20.1221 9.51243C22.1049 11.5429 22.1049 14.7847 20.1221 16.8152L12.7498 24.3644C10.7834 26.378 7.55686 26.4163 5.54322 24.4498C5.5144 24.4217 5.48592 24.3932 5.45777 24.3644L5.37754 24.2822C3.39468 22.2518 3.39468 19.0099 5.37754 16.9795Z",
        fill: "#12D2AC"
    }), _react.createElement("path", {
        fillRule: "evenodd",
        clipRule: "evenodd",
        d: "M20.0479 9.43034L27.3399 16.8974C29.3674 18.9735 29.3674 22.2883 27.3399 24.3644C25.3735 26.3781 22.147 26.4163 20.1333 24.4499C20.1045 24.4217 20.076 24.3933 20.0479 24.3644L12.7558 16.8974C10.7284 14.8213 10.7284 11.5065 12.7558 9.43034C14.7223 7.4167 17.9488 7.37844 19.9624 9.34489C19.9912 9.37304 20.0197 9.40152 20.0479 9.43034Z",
        fill: "#307AF2"
    }), _react.createElement("path", {
        fillRule: "evenodd",
        clipRule: "evenodd",
        d: "M20.1321 9.52163L23.6851 13.1599L16.3931 20.627L9.10103 13.1599L12.6541 9.52163C14.6707 7.45664 17.9794 7.4174 20.0444 9.434C20.074 9.46286 20.1032 9.49207 20.1321 9.52163Z",
        fill: "#0057FE"
    })), _react.createElement("defs", null, _react.createElement("clipPath", {
        id: "clip0"
    }, _react.createElement("rect", {
        width: 26,
        height: 19,
        fill: "white",
        transform: "translate(3.5 7)"
    }))));
const _default = SvgLogo;

},
"ceaa0c46": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "footer": `footer-7f1486e4`
};

},
"d8012317": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("ceaa0c46"));
function Footer(props = {}) {
    const { className, ...restProps } = props;
    return _react.default.createElement(_webreact.Layout.Footer, {
        className: (0, _classnames.default)(_indexmoduleless.default.footer, className),
        ...restProps
    }, "Arco Design Pro");
}
const _default = Footer;

},
"efe06641": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return LoginBanner;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("4dbeb4c8"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("8ac200a7"));
function LoginBanner() {
    const t = (0, _useLocale.default)(_locale.default);
    const data = [
        {
            slogan: t['login.banner.slogan1'],
            subSlogan: t['login.banner.subSlogan1'],
            image: 'http://p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/6c85f43aed61e320ebec194e6a78d6d3.png~tplv-uwbnlip3yd-png.png'
        },
        {
            slogan: t['login.banner.slogan2'],
            subSlogan: t['login.banner.subSlogan2'],
            image: 'http://p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/6c85f43aed61e320ebec194e6a78d6d3.png~tplv-uwbnlip3yd-png.png'
        },
        {
            slogan: t['login.banner.slogan3'],
            subSlogan: t['login.banner.subSlogan3'],
            image: 'http://p1-arco.byteimg.com/tos-cn-i-uwbnlip3yd/6c85f43aed61e320ebec194e6a78d6d3.png~tplv-uwbnlip3yd-png.png'
        }
    ];
    return _react.default.createElement(_webreact.Carousel, {
        className: _indexmoduleless.default.carousel,
        animation: "fade"
    }, data.map((item, index)=>_react.default.createElement("div", {
            key: `${index}`
        }, _react.default.createElement("div", {
            className: _indexmoduleless.default['carousel-item']
        }, _react.default.createElement("div", {
            className: _indexmoduleless.default['carousel-title']
        }, item.slogan), _react.default.createElement("div", {
            className: _indexmoduleless.default['carousel-sub-title']
        }, item.subSlogan), _react.default.createElement("img", {
            alt: "banner-image",
            className: _indexmoduleless.default['carousel-image'],
            src: item.image
        })))));
}

},});