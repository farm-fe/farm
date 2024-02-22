(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'setting_index_a95e.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"86e03dfa": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("63276218"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("b1d6eea6"));
function Security() {
    const t = (0, _useLocale.default)(_locale.default);
    const userInfo = (0, _reactredux.useSelector)((state)=>{
        return state.userInfo || {};
    });
    const data = [
        {
            title: t['userSetting.security.password'],
            value: t['userSetting.security.password.tips']
        },
        {
            title: t['userSetting.security.question'],
            value: '',
            placeholder: t['userSetting.security.question.placeholder']
        },
        {
            title: t['userSetting.security.phone'],
            value: userInfo.phoneNumber ? `${t['userSetting.security.phone.tips']} ${userInfo.phoneNumber}` : ''
        },
        {
            title: t['userSetting.security.email'],
            value: '',
            placeholder: t['userSetting.security.email.placeholder']
        }
    ];
    return _react.default.createElement("div", {
        className: _indexmoduleless.default['security']
    }, data.map((item, index)=>_react.default.createElement("div", {
            className: _indexmoduleless.default['security-item'],
            key: index
        }, _react.default.createElement("span", {
            className: _indexmoduleless.default['security-item-title']
        }, item.title), _react.default.createElement("div", {
            className: _indexmoduleless.default['security-item-content']
        }, _react.default.createElement("span", {
            className: (0, _classnames.default)({
                [`${_indexmoduleless.default['security-item-placeholder']}`]: !item.value
            })
        }, item.value || item.placeholder), _react.default.createElement("span", null, _react.default.createElement(_webreact.Button, {
            type: "text"
        }, item.value ? t['userSetting.btn.edit'] : t['userSetting.btn.set']))))));
}
const _default = Security;

},
"975f2322": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _reactredux = farmRequire("e429bf23");
const _header = /*#__PURE__*/ _interop_require_default._(farmRequire("d1fafdb7"));
const _info = /*#__PURE__*/ _interop_require_default._(farmRequire("5a428c1e"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("63276218"));
farmRequire("52e1b7e4");
const _security = /*#__PURE__*/ _interop_require_default._(farmRequire("86e03dfa"));
const _verified = /*#__PURE__*/ _interop_require_default._(farmRequire("6f090fd3"));
function UserInfo() {
    const t = (0, _useLocale.default)(_locale.default);
    const userInfo = (0, _reactredux.useSelector)((state)=>state.userInfo);
    const loading = (0, _reactredux.useSelector)((state)=>state.userLoading);
    const [activeTab, setActiveTab] = (0, _react.useState)('basic');
    return _react.default.createElement("div", null, _react.default.createElement(_webreact.Card, {
        style: {
            padding: '14px 20px'
        }
    }, _react.default.createElement(_header.default, {
        userInfo: userInfo,
        loading: loading
    })), _react.default.createElement(_webreact.Card, {
        style: {
            marginTop: '16px'
        }
    }, _react.default.createElement(_webreact.Tabs, {
        activeTab: activeTab,
        onChange: setActiveTab,
        type: "rounded"
    }, _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "basic",
        title: t['userSetting.title.basicInfo']
    }, _react.default.createElement(_info.default, {
        loading: loading
    })), _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "security",
        title: t['userSetting.title.security']
    }, _react.default.createElement(_security.default, null)), _react.default.createElement(_webreact.Tabs.TabPane, {
        key: "verified",
        title: t['userSetting.label.verified']
    }, _react.default.createElement(_verified.default, null)))));
}
const _default = UserInfo;

},
"d1fafdb7": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return Info;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("63276218"));
const _headermoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("92de8e23"));
function Info({ userInfo = {}, loading }) {
    const t = (0, _useLocale.default)(_locale.default);
    const [avatar, setAvatar] = (0, _react.useState)('');
    function onAvatarChange(_, file) {
        setAvatar(file.originFile ? URL.createObjectURL(file.originFile) : '');
    }
    (0, _react.useEffect)(()=>{
        setAvatar(userInfo.avatar);
    }, [
        userInfo
    ]);
    const loadingImg = _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 0
        },
        style: {
            width: '100px',
            height: '100px'
        },
        animation: true
    });
    const loadingNode = _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 1
        },
        animation: true
    });
    return _react.default.createElement("div", {
        className: _headermoduleless.default['info-wrapper']
    }, _react.default.createElement(_webreact.Upload, {
        showUploadList: false,
        onChange: onAvatarChange
    }, loading ? loadingImg : _react.default.createElement(_webreact.Avatar, {
        size: 100,
        triggerIcon: _react.default.createElement(_icon.IconCamera, null),
        className: _headermoduleless.default['info-avatar']
    }, avatar ? _react.default.createElement("img", {
        src: avatar
    }) : _react.default.createElement(_icon.IconPlus, null))), _react.default.createElement(_webreact.Descriptions, {
        className: _headermoduleless.default['info-content'],
        column: 2,
        colon: "：",
        labelStyle: {
            textAlign: 'right'
        },
        data: [
            {
                label: t['userSetting.label.name'],
                value: loading ? loadingNode : userInfo.name
            },
            {
                label: t['userSetting.label.verified'],
                value: loading ? loadingNode : _react.default.createElement("span", null, userInfo.verified ? _react.default.createElement(_webreact.Tag, {
                    color: "green",
                    className: _headermoduleless.default['verified-tag']
                }, t['userSetting.value.verified']) : _react.default.createElement(_webreact.Tag, {
                    color: "red",
                    className: _headermoduleless.default['verified-tag']
                }, t['userSetting.value.notVerified']), _react.default.createElement(_webreact.Link, {
                    role: "button",
                    className: _headermoduleless.default['edit-btn']
                }, t['userSetting.btn.edit']))
            },
            {
                label: t['userSetting.label.accountId'],
                value: loading ? loadingNode : userInfo.accountId
            },
            {
                label: t['userSetting.label.phoneNumber'],
                value: loading ? loadingNode : _react.default.createElement("span", null, userInfo.phoneNumber, _react.default.createElement(_webreact.Link, {
                    role: "button",
                    className: _headermoduleless.default['edit-btn']
                }, t['userSetting.btn.edit']))
            },
            {
                label: t['userSetting.label.registrationTime'],
                value: loading ? loadingNode : userInfo.registrationTime
            }
        ]
    }));
}

},});