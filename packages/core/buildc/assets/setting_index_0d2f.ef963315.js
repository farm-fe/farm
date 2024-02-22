(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'setting_index_0d2f.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"52e1b7e4": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
(0, _setupMock.default)({
    setup: ()=>{
        // 保存个人信息
        _mockjs.default.mock(new RegExp("/api/user/saveInfo"), ()=>{
            return "ok";
        });
        // 实名认证信息
        _mockjs.default.mock(new RegExp("/api/user/verified/enterprise"), ()=>{
            return _mockjs.default.mock({
                accountType: "企业账号",
                isVerified: true,
                verifiedTime: _mockjs.default.Random.datetime("yyyy-MM-dd HH:mm:ss"),
                legalPersonName: _mockjs.default.Random.cfirst() + "**",
                certificateType: "中国身份证",
                certificationNumber: /[1-9]{3}[*]{12}[0-9]{3}/,
                enterpriseName: _mockjs.default.Random.ctitle(),
                enterpriseCertificateType: "企业营业执照",
                organizationCode: /[1-9]{1}[*]{7}[0-9]{1}/
            });
        });
        _mockjs.default.mock(new RegExp("/api/user/verified/authList"), ()=>{
            return new Array(3).fill("0").map(()=>({
                    authType: "企业证件认证",
                    authContent: `企业证件认证，法人姓名${_mockjs.default.Random.cfirst()}**`,
                    authStatus: _mockjs.default.Random.natural(0, 1),
                    createdTime: _mockjs.default.Random.datetime("yyyy-MM-dd HH:mm:ss")
                }));
        });
    }
});

},
"92de8e23": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "edit-btn": `edit-btn-c7eab509`,
    "info-avatar": `info-avatar-c7eab509`,
    "info-content": `info-content-c7eab509`,
    "info-wrapper": `info-wrapper-c7eab509`,
    "verified-tag": `verified-tag-c7eab509`
};

},
"b1d6eea6": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "content": `content-6d4dcde2`,
    "info-avatar": `info-avatar-6d4dcde2`,
    "info-form": `info-form-6d4dcde2`,
    "security": `security-6d4dcde2`,
    "security-item": `security-item-6d4dcde2`,
    "security-item-content": `security-item-content-6d4dcde2`,
    "security-item-placeholder": `security-item-placeholder-6d4dcde2`,
    "security-item-title": `security-item-title-6d4dcde2`,
    "sidebar": `sidebar-6d4dcde2`,
    "verified": `verified-6d4dcde2`,
    "verified-enterprise": `verified-enterprise-6d4dcde2`,
    "wrapper": `wrapper-6d4dcde2`
};

},});