(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'step_index_b411.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"5e674e64": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("3e1d370b"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("97d57bdd"));
const { Title, Paragraph } = _webreact.Typography;
function StepForm() {
    const t = (0, _useLocale.default)(_locale.default);
    const [current, setCurrent] = (0, _react.useState)(1);
    const [form] = _webreact.Form.useForm();
    const viewForm = ()=>{
        const values = form.getFields();
        form.setFields(values);
        setCurrent(1);
    };
    const reCreateForm = ()=>{
        form.resetFields();
        setCurrent(1);
    };
    const toNext = async ()=>{
        try {
            await form.validate();
            setCurrent(current + 1);
        } catch (_) {}
    };
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.container
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(Title, {
        heading: 5
    }, t['stepForm.desc.basicInfo']), _react.default.createElement("div", {
        className: _indexmoduleless.default.wrapper
    }, _react.default.createElement(_webreact.Steps, {
        current: current,
        lineless: true
    }, _react.default.createElement(_webreact.Steps.Step, {
        title: t['stepForm.title.basicInfo'],
        description: t['stepForm.desc.basicInfo']
    }), _react.default.createElement(_webreact.Steps.Step, {
        title: t['stepForm.title.channel'],
        description: t['stepForm.desc.channel']
    }), _react.default.createElement(_webreact.Steps.Step, {
        title: t['stepForm.title.created'],
        description: t['stepForm.desc.created']
    })), _react.default.createElement(_webreact.Form, {
        form: form,
        className: _indexmoduleless.default.form
    }, current === 1 && _react.default.createElement(_webreact.Form.Item, {
        noStyle: true
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.basicInfo.name'],
        required: true,
        field: "basic.name",
        rules: [
            {
                required: true,
                message: t['stepForm.basicInfo.name.required']
            },
            {
                validator: (value, callback)=>{
                    if (!/^[\u4e00-\u9fa5a-zA-Z0-9]{1,20}$/g.test(value)) {
                        callback(t['stepForm.basicInfo.name.placeholder']);
                    }
                }
            }
        ]
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['stepForm.basicInfo.name.placeholder']
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.basicInfo.channelType'],
        required: true,
        initialValue: "app",
        field: "basic.channelType",
        rules: [
            {
                required: true,
                message: t['stepForm.basicInfo.channelType.required']
            }
        ]
    }, _react.default.createElement(_webreact.Select, null, _react.default.createElement(_webreact.Select.Option, {
        value: "app"
    }, "APP通用渠道"), _react.default.createElement(_webreact.Select.Option, {
        value: "site"
    }, "网页通用渠道"), _react.default.createElement(_webreact.Select.Option, {
        value: "game"
    }, "游戏通用渠道"))), _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.basicInfo.time'],
        required: true,
        field: "basic.time",
        rules: [
            {
                required: true,
                message: t['stepForm.basicInfo.time.required']
            }
        ]
    }, _react.default.createElement(_webreact.DatePicker.RangePicker, {
        style: {
            width: '100%'
        }
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.basicInfo.link'],
        required: true,
        extra: t['stepForm.basicInfo.link.tips'],
        field: "basic.link",
        initialValue: 'https://arco.design',
        rules: [
            {
                required: true
            }
        ]
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['stepForm.basicInfo.link.placeholder']
    }))), current === 2 && _react.default.createElement(_webreact.Form.Item, {
        noStyle: true
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.channel.source'],
        required: true,
        field: "channel.source",
        rules: [
            {
                required: true,
                message: t['stepForm.channel.source.required']
            }
        ]
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['stepForm.channel.source.placeholder']
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.channel.media'],
        required: true,
        field: "channel.media",
        rules: [
            {
                required: true,
                message: t['stepForm.channel.media.required']
            }
        ]
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['stepForm.channel.media.placeholder']
    })), _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.channel.keywords'],
        required: true,
        field: "channel.keywords",
        initialValue: [
            '今日头条',
            '火山'
        ],
        rules: [
            {
                required: true
            }
        ]
    }, _react.default.createElement(_webreact.InputTag, null)), _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.channel.remind'],
        required: true,
        initialValue: true,
        field: "channel.remind",
        triggerPropName: "checked",
        rules: [
            {
                required: true
            }
        ]
    }, _react.default.createElement(_webreact.Switch, null)), _react.default.createElement(_webreact.Form.Item, {
        label: t['stepForm.channel.content'],
        required: true,
        field: "channel.content",
        rules: [
            {
                required: true,
                message: t['stepForm.channel.content.required']
            }
        ]
    }, _react.default.createElement(_webreact.Input.TextArea, {
        placeholder: t['stepForm.channel.content.placeholder']
    }))), current !== 3 ? _react.default.createElement(_webreact.Form.Item, {
        label: " "
    }, _react.default.createElement(_webreact.Space, null, current === 2 && _react.default.createElement(_webreact.Button, {
        size: "large",
        onClick: ()=>setCurrent(current - 1)
    }, t['stepForm.prev']), current !== 3 && _react.default.createElement(_webreact.Button, {
        type: "primary",
        size: "large",
        onClick: toNext
    }, t['stepForm.next']))) : _react.default.createElement(_webreact.Form.Item, {
        noStyle: true
    }, _react.default.createElement(_webreact.Result, {
        status: "success",
        title: t['stepForm.created.success.title'],
        subTitle: t['stepForm.created.success.desc'],
        extra: [
            _react.default.createElement(_webreact.Button, {
                key: "reset",
                style: {
                    marginRight: 16
                },
                onClick: viewForm
            }, t['stepForm.created.success.view']),
            _react.default.createElement(_webreact.Button, {
                key: "again",
                type: "primary",
                onClick: reCreateForm
            }, t['stepForm.created.success.again'])
        ]
    })))), current === 3 && _react.default.createElement("div", {
        className: _indexmoduleless.default['form-extra']
    }, _react.default.createElement(Title, {
        heading: 6
    }, t['stepForm.created.extra.title']), _react.default.createElement(Paragraph, {
        type: "secondary"
    }, t['stepForm.created.extra.desc'], _react.default.createElement(_webreact.Button, {
        type: "text"
    }, t['stepForm.created.extra.detail'])))));
}
const _default = StepForm;

},});