(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'card_index_3c0e.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"72346ccd": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _PermissionWrapper = /*#__PURE__*/ _interop_require_default._(farmRequire("60420157"));
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _icon = farmRequire("f988cd7d");
const _classnames = /*#__PURE__*/ _interop_require_default._(farmRequire("e2a3c978"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("72edb2f7"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("35c3ed68"));
const IconList = [
    _icon.IconStarFill,
    _icon.IconThumbUpFill,
    _icon.IconSunFill,
    _icon.IconFaceSmileFill,
    _icon.IconPenFill
].map((Tag, index)=>_react.default.createElement(Tag, {
        key: index
    }));
const { Paragraph } = _webreact.Typography;
function CardBlock(props) {
    const { type, card = {} } = props;
    const [visible, setVisible] = (0, _react.useState)(false);
    const [status, setStatus] = (0, _react.useState)(card.status);
    const [loading, setLoading] = (0, _react.useState)(props.loading);
    const t = (0, _useLocale.default)(_locale.default);
    const changeStatus = async ()=>{
        setLoading(true);
        await new Promise((resolve)=>setTimeout(()=>{
                setStatus(status !== 1 ? 1 : 0);
                resolve(null);
            }, 1000)).finally(()=>setLoading(false));
    };
    (0, _react.useEffect)(()=>{
        setLoading(props.loading);
    }, [
        props.loading
    ]);
    (0, _react.useEffect)(()=>{
        if (card.status !== status) {
            setStatus(card.status);
        }
    }, [
        card.status
    ]);
    const getTitleIcon = ()=>{
        if (type === 'service' && typeof card.icon === 'number') {
            return _react.default.createElement("div", {
                className: _indexmoduleless.default.icon
            }, IconList[card.icon % IconList.length]);
        }
        return null;
    };
    const getButtonGroup = ()=>{
        if (type === 'quality') {
            return _react.default.createElement(_react.default.Fragment, null, _react.default.createElement(_PermissionWrapper.default, {
                requiredPermissions: [
                    {
                        resource: /^menu.list.*/,
                        actions: [
                            'read'
                        ]
                    }
                ]
            }, _react.default.createElement(_webreact.Button, {
                type: "primary",
                style: {
                    marginLeft: '12px'
                },
                loading: loading
            }, t['cardList.options.qualityInspection'])), _react.default.createElement(_PermissionWrapper.default, {
                requiredPermissions: [
                    {
                        resource: /^menu.list.*/,
                        actions: [
                            'write'
                        ]
                    }
                ]
            }, _react.default.createElement(_webreact.Button, {
                loading: loading
            }, t['cardList.options.remove'])));
        }
        if (type === 'service') {
            return _react.default.createElement(_react.default.Fragment, null, status === 1 ? _react.default.createElement(_webreact.Button, {
                loading: loading,
                onClick: changeStatus
            }, t['cardList.options.cancel']) : _react.default.createElement(_webreact.Button, {
                type: "outline",
                loading: loading,
                onClick: changeStatus
            }, status === 0 ? t['cardList.options.subscribe'] : t['cardList.options.renewal']));
        }
        return _react.default.createElement(_webreact.Switch, {
            checked: !!status,
            loading: loading,
            onChange: changeStatus
        });
    };
    const getStatus = ()=>{
        if (type === 'rules' && status) {
            return _react.default.createElement(_webreact.Tag, {
                color: "green",
                icon: _react.default.createElement(_icon.IconCheckCircleFill, null),
                className: _indexmoduleless.default.status,
                size: "small"
            }, t['cardList.tag.activated']);
        }
        switch(status){
            case 1:
                return _react.default.createElement(_webreact.Tag, {
                    color: "green",
                    icon: _react.default.createElement(_icon.IconCheckCircleFill, null),
                    className: _indexmoduleless.default.status,
                    size: "small"
                }, t['cardList.tag.opened']);
            case 2:
                return _react.default.createElement(_webreact.Tag, {
                    color: "red",
                    icon: _react.default.createElement(_icon.IconCloseCircleFill, null),
                    className: _indexmoduleless.default.status,
                    size: "small"
                }, t['cardList.tag.expired']);
            default:
                return null;
        }
    };
    const getContent = ()=>{
        if (loading) {
            return _react.default.createElement(_webreact.Skeleton, {
                text: {
                    rows: type !== 'quality' ? 3 : 2
                },
                animation: true,
                className: _indexmoduleless.default['card-block-skeleton']
            });
        }
        if (type !== 'quality') {
            return _react.default.createElement(Paragraph, null, card.description);
        }
        return _react.default.createElement(_webreact.Descriptions, {
            column: 2,
            data: [
                {
                    label: '待质检数',
                    value: card.qualityCount
                },
                {
                    label: '积压时长',
                    value: `${card.duration}s`
                },
                {
                    label: '待抽检数',
                    value: card.randomCount
                }
            ]
        });
    };
    const className = (0, _classnames.default)(_indexmoduleless.default['card-block'], _indexmoduleless.default[`${type}-card`]);
    return _react.default.createElement(_webreact.Card, {
        bordered: true,
        className: className,
        size: "small",
        title: loading ? _react.default.createElement(_webreact.Skeleton, {
            animation: true,
            text: {
                rows: 1,
                width: [
                    '100%'
                ]
            },
            style: {
                width: '120px',
                height: '24px'
            },
            className: _indexmoduleless.default['card-block-skeleton']
        }) : _react.default.createElement(_react.default.Fragment, null, _react.default.createElement("div", {
            className: (0, _classnames.default)(_indexmoduleless.default.title, {
                [_indexmoduleless.default['title-more']]: visible
            })
        }, getTitleIcon(), card.title, getStatus(), _react.default.createElement(_webreact.Dropdown, {
            droplist: _react.default.createElement(_webreact.Menu, null, [
                '操作1',
                '操作2'
            ].map((item, key)=>_react.default.createElement(_webreact.Menu.Item, {
                    key: key.toString()
                }, item))),
            trigger: "click",
            onVisibleChange: setVisible,
            popupVisible: visible
        }, _react.default.createElement("div", {
            className: _indexmoduleless.default.more
        }, _react.default.createElement(_icon.IconMore, null)))), _react.default.createElement("div", {
            className: _indexmoduleless.default.time
        }, card.time))
    }, _react.default.createElement("div", {
        className: _indexmoduleless.default.content
    }, getContent()), _react.default.createElement("div", {
        className: _indexmoduleless.default.extra
    }, getButtonGroup()));
}
const _default = CardBlock;

},});