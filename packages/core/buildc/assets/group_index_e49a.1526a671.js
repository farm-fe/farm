(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'group_index_e49a.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"9afc8098": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("121216b0"));
farmRequire("a84302f3");
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("da4e9fba"));
function GroupForm() {
    const t = (0, _useLocale.default)(_locale.default);
    const formRef = (0, _react.useRef)();
    const [loading, setLoading] = (0, _react.useState)(false);
    function submit(data) {
        setLoading(true);
        _axios.default.post('/api/groupForm', {
            data
        }).then(()=>{
            _webreact.Message.success(t['groupForm.submitSuccess']);
        }).finally(()=>{
            setLoading(false);
        });
    }
    function handleSubmit() {
        formRef.current.validate().then((values)=>{
            submit(values);
        });
    }
    function handleReset() {
        formRef.current.resetFields();
    }
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.container
    }, _react.default.createElement(_webreact.Form, {
        layout: "vertical",
        ref: formRef,
        className: _indexmoduleless.default['form-group']
    }, _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['groupForm.title.video']), _react.default.createElement(_webreact.Grid.Row, {
        gutter: 80
    }, _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.mode'],
        field: "video.mode",
        initialValue: 'custom'
    }, _react.default.createElement(_webreact.Select, {
        placeholder: t['groupForm.placeholder.video.mode']
    }, _react.default.createElement(_webreact.Select.Option, {
        value: "custom"
    }, "自定义"), _react.default.createElement(_webreact.Select.Option, {
        value: "mode1"
    }, "模式1"), _react.default.createElement(_webreact.Select.Option, {
        value: "mode2"
    }, "模式2")))), _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.acquisition.resolution'],
        field: "video.acquisition.resolution"
    }, _react.default.createElement(_webreact.Select, {
        placeholder: t['groupForm.placeholder.video.acquisition.resolution']
    }, _react.default.createElement(_webreact.Select.Option, {
        value: "resolution1"
    }, "分辨率1"), _react.default.createElement(_webreact.Select.Option, {
        value: "resolution2"
    }, "分辨率2"), _react.default.createElement(_webreact.Select.Option, {
        value: "resolution3"
    }, "分辨率3")))), _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.acquisition.frameRate'],
        field: "video.acquisition.frameRate"
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['groupForm.placeholder.video.acquisition.frameRate'],
        addAfter: "fps"
    })))), _react.default.createElement(_webreact.Grid.Row, {
        gutter: 80
    }, _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.encoding.resolution'],
        field: "video.encoding.resolution"
    }, _react.default.createElement(_webreact.Select, {
        placeholder: t['groupForm.placeholder.video.encoding.resolution']
    }, _react.default.createElement(_webreact.Select.Option, {
        value: "resolution1"
    }, "分辨率1"), _react.default.createElement(_webreact.Select.Option, {
        value: "resolution2"
    }, "分辨率2"), _react.default.createElement(_webreact.Select.Option, {
        value: "resolution3"
    }, "分辨率3")))), _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.encoding.rate.min'],
        field: "video.encoding.rate.min"
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['groupForm.placeholder.video.encoding.rate.min'],
        addAfter: "bps"
    }))), _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.encoding.rate.max'],
        field: "video.encoding.rate.max"
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['groupForm.placeholder.video.encoding.rate.max'],
        addAfter: "bps"
    })))), _react.default.createElement(_webreact.Grid.Row, {
        gutter: 80
    }, _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.encoding.rate.default'],
        field: "video.encoding.rate.default"
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['groupForm.placeholder.video.encoding.rate.default'],
        addAfter: "bps"
    }))), _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.encoding.frameRate'],
        field: "video.encoding.frameRate"
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['groupForm.placeholder.video.encoding.frameRate'],
        addAfter: "fps"
    }))), _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.video.encoding.profile'],
        field: "video.encoding.profile"
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['groupForm.placeholder.video.encoding.profile'],
        addAfter: "bps"
    }))))), _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['groupForm.title.audio']), _react.default.createElement(_webreact.Grid.Row, {
        gutter: 80
    }, _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.audio.mode'],
        initialValue: 'custom',
        field: "audio.mode"
    }, _react.default.createElement(_webreact.Select, {
        placeholder: t['groupForm.placeholder.audio.mode']
    }, _react.default.createElement(_webreact.Select.Option, {
        value: "custom"
    }, "自定义"), _react.default.createElement(_webreact.Select.Option, {
        value: "mode1"
    }, "模式1"), _react.default.createElement(_webreact.Select.Option, {
        value: "mode2"
    }, "模式2")))), _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.audio.acquisition.channels'],
        field: "audio.acquisition.channels"
    }, _react.default.createElement(_webreact.Select, {
        placeholder: t['groupForm.placeholder.audio.acquisition.channels']
    }, _react.default.createElement(_webreact.Select.Option, {
        value: "1"
    }, "1"), _react.default.createElement(_webreact.Select.Option, {
        value: "2"
    }, "2"), _react.default.createElement(_webreact.Select.Option, {
        value: "3"
    }, "3")))), _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.audio.encoding.rate'],
        field: "audio.encoding.rate"
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['groupForm.placeholder.audio.encoding.rate'],
        addAfter: "bps"
    })))), _react.default.createElement(_webreact.Grid.Row, {
        gutter: 80
    }, _react.default.createElement(_webreact.Grid.Col, {
        span: 8
    }, _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.audio.encoding.profile'],
        field: "audio.encoding.profile"
    }, _react.default.createElement(_webreact.Input, {
        placeholder: t['groupForm.placeholder.audio.encoding.profile'],
        addAfter: "fps"
    }))))), _react.default.createElement(_webreact.Card, {
        style: {
            marginBottom: '40px'
        }
    }, _react.default.createElement(_webreact.Typography.Title, {
        heading: 6
    }, t['groupForm.title.explanation']), _react.default.createElement(_webreact.Form.Item, {
        label: t['groupForm.form.label.explanation'],
        field: "audio.explanation"
    }, _react.default.createElement(_webreact.Input.TextArea, {
        placeholder: t['groupForm.placeholder.explanation']
    })))), _react.default.createElement("div", {
        className: _indexmoduleless.default.actions
    }, _react.default.createElement(_webreact.Space, null, _react.default.createElement(_webreact.Button, {
        onClick: handleReset,
        size: "large"
    }, t['groupForm.reset']), _react.default.createElement(_webreact.Button, {
        type: "primary",
        onClick: handleSubmit,
        loading: loading,
        size: "large"
    }, t['groupForm.submit']))));
}
const _default = GroupForm;

},});