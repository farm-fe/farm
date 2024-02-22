(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'setting_index_e598.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"5a428c1e": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _context = farmRequire("64285704");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("63276218"));
function InfoForm({ loading }) {
    const t = (0, _useLocale.default)(_locale.default);
    const [form] = _webreact.Form.useForm();
    const { lang } = (0, _react.useContext)(_context.GlobalContext);
    const handleSave = async ()=>{
        try {
            await form.validate();
            _webreact.Message.success('userSetting.saveSuccess');
        } catch (_) {}
    };
    const handleReset = ()=>{
        form.resetFields();
    };
    const loadingNode = (rows = 1)=>{
        return _react.default.createElement(_webreact.Skeleton, {
            text: {
                rows,
                width: new Array(rows).fill('100%')
            },
            animation: true
        });
    };
    return _react.default.createElement(_webreact.Form, {
        style: {
            width: '500px',
            marginTop: '6px'
        },
        form: form,
        labelCol: {
            span: lang === 'en-US' ? 7 : 6
        },
        wrapperCol: {
            span: lang === 'en-US' ? 17 : 18
        }
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['userSetting.info.email'],
        field: "email",
        rules: [
            {
                type: 'email',
                required: true,
                message: t['userSetting.info.email.placeholder']
            }
        ]
    }, loading ? loadingNode() : _react.default.createElement(_webreact.Input, {
        placeholder: t['userSetting.info.email.placeholder']
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['userSetting.info.nickName'],
        field: "nickName",
        rules: [
            {
                required: true,
                message: t['userSetting.info.nickName.placeholder']
            }
        ]
    }, loading ? loadingNode() : _react.default.createElement(_webreact.Input, {
        placeholder: t['userSetting.info.nickName.placeholder']
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['userSetting.info.area'],
        field: "rangeArea",
        rules: [
            {
                required: true,
                message: t['userSetting.info.area.placeholder']
            }
        ]
    }, loading ? loadingNode() : _react.default.createElement(_webreact.Select, {
        options: [
            '中国'
        ],
        placeholder: t['userSetting.info.area.placeholder']
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['userSetting.info.location'],
        field: "location",
        initialValue: [
            'BeiJing',
            'BeiJing',
            'HaiDian'
        ],
        rules: [
            {
                required: true
            }
        ]
    }, loading ? loadingNode() : _react.default.createElement(_webreact.Cascader, {
        options: [
            {
                label: '北京市',
                value: 'BeiJing',
                children: [
                    {
                        label: '北京市',
                        value: 'BeiJing',
                        children: [
                            {
                                label: '海淀区',
                                value: 'HaiDian'
                            },
                            {
                                label: '朝阳区',
                                value: 'ChaoYang'
                            }
                        ]
                    }
                ]
            },
            {
                label: '上海市',
                value: 'ShangHai',
                children: [
                    {
                        label: '上海市',
                        value: 'ShangHai',
                        children: [
                            {
                                label: '黄浦区',
                                value: 'HuangPu'
                            },
                            {
                                label: '静安区',
                                value: 'JingAn'
                            }
                        ]
                    }
                ]
            }
        ]
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['userSetting.info.address'],
        field: "address"
    }, loading ? loadingNode() : _react.default.createElement(_webreact.Input, {
        placeholder: t['userSetting.info.address.placeholder']
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['userSetting.info.profile'],
        field: "profile"
    }, loading ? loadingNode(3) : _react.default.createElement(_webreact.Input.TextArea, {
        placeholder: t['userSetting.info.profile.placeholder']
    })), _react.default.createElement(_webreact.Form.Item, {
        label: " "
    }, _react.default.createElement(_webreact.Space, null, _react.default.createElement(_webreact.Button, {
        type: "primary",
        onClick: handleSave
    }, t['userSetting.save']), _react.default.createElement(_webreact.Button, {
        onClick: handleReset
    }, t['userSetting.reset']))));
}
const _default = InfoForm;

},
"6f090fd3": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("63276218"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("b1d6eea6"));
function Verified() {
    const t = (0, _useLocale.default)(_locale.default);
    const [data, setData] = (0, _react.useState)({
        accountType: '',
        isVerified: true,
        verifiedTime: '',
        legalPersonName: '',
        certificateType: '',
        certificationNumber: '',
        enterpriseName: '',
        enterpriseCertificateType: '',
        organizationCode: ''
    });
    const [loading, setLoading] = (0, _react.useState)(true);
    const [tableData, setTableData] = (0, _react.useState)([]);
    const [tableLoading, setTableLoading] = (0, _react.useState)(true);
    const getData = async ()=>{
        const { data } = await _axios.default.get('/api/user/verified/enterprise').finally(()=>setLoading(false));
        setData(data);
        const { data: tableData } = await _axios.default.get('/api/user/verified/authList').finally(()=>setTableLoading(false));
        setTableData(tableData);
    };
    (0, _react.useEffect)(()=>{
        getData();
    }, []);
    const loadingNode = _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 1
        }
    });
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.verified
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['userSetting.verified.enterprise']), _react.default.createElement(_webreact.Descriptions, {
        className: _indexmoduleless.default['verified-enterprise'],
        labelStyle: {
            textAlign: 'right'
        },
        layout: "inline-horizontal",
        colon: "：",
        column: 3,
        data: Object.entries(data).map(([key, value])=>({
                label: t[`userSetting.verified.label.${key}`],
                value: loading ? loadingNode : typeof value === 'boolean' ? value ? _react.default.createElement(_webreact.Tag, {
                    color: "green"
                }, t['userSetting.value.verified']) : _react.default.createElement(_webreact.Tag, {
                    color: "red"
                }, t['userSetting.value.notVerified']) : value
            }))
    }), _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['userSetting.verified.records']), _react.default.createElement(_webreact.Table, {
        columns: [
            {
                title: t['userSetting.verified.authType'],
                dataIndex: 'authType'
            },
            {
                title: t['userSetting.verified.authContent'],
                dataIndex: 'authContent'
            },
            {
                title: t['userSetting.verified.authStatus'],
                dataIndex: 'authStatus',
                render (x) {
                    return x ? _react.default.createElement(_webreact.Badge, {
                        status: "success",
                        text: t['userSetting.verified.status.success']
                    }) : _react.default.createElement("span", null, _react.default.createElement(_webreact.Badge, {
                        status: "processing",
                        text: t['userSetting.verified.status.waiting']
                    }));
                }
            },
            {
                title: t['userSetting.verified.createdTime'],
                dataIndex: 'createdTime'
            },
            {
                title: t['userSetting.verified.operation'],
                headerCellStyle: {
                    paddingLeft: '15px'
                },
                render: (_, x)=>{
                    if (x.authStatus) {
                        return _react.default.createElement(_webreact.Button, {
                            type: "text"
                        }, t['userSetting.verified.operation.view']);
                    }
                    return _react.default.createElement(_webreact.Space, null, _react.default.createElement(_webreact.Button, {
                        type: "text"
                    }, t['userSetting.verified.operation.view']), _react.default.createElement(_webreact.Button, {
                        type: "text"
                    }, t['userSetting.verified.operation.revoke']));
                }
            }
        ],
        data: tableData,
        loading: tableLoading
    }));
}
const _default = Verified;

},});