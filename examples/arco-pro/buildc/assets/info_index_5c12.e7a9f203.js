(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'info_index_5c12.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"9e13e96b": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _icon = farmRequire("f988cd7d");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("0a457a53"));
function UserInfoHeader(props) {
    const { userInfo = {}, loading } = props;
    const loadingNode = _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 1,
            style: {
                width: '100px',
                height: '20px',
                marginBottom: '-4px'
            },
            width: [
                '100%'
            ]
        },
        animation: true
    });
    const loadingImgNode = _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 0
        },
        image: {
            style: {
                width: '64px',
                height: '64px'
            },
            shape: 'circle'
        },
        animation: true
    });
    return _react.default.createElement("div", {
        className: _indexmoduleless.default.header
    }, _react.default.createElement(_webreact.Space, {
        size: 8,
        direction: "vertical",
        align: "center",
        className: _indexmoduleless.default['header-content']
    }, loading ? loadingImgNode : _react.default.createElement(_webreact.Avatar, {
        size: 64,
        triggerIcon: _react.default.createElement(_icon.IconCamera, null)
    }, _react.default.createElement("img", {
        src: userInfo.avatar
    })), _react.default.createElement("div", {
        className: _indexmoduleless.default.username
    }, loading ? loadingNode : userInfo.name), _react.default.createElement("div", {
        className: _indexmoduleless.default['user-msg']
    }, _react.default.createElement(_webreact.Space, {
        size: 18
    }, _react.default.createElement("div", null, _react.default.createElement(_icon.IconUser, null), _react.default.createElement("span", {
        className: _indexmoduleless.default['user-msg-text']
    }, loading ? loadingNode : userInfo.jobName)), _react.default.createElement("div", null, _react.default.createElement(_icon.IconHome, null), _react.default.createElement("span", {
        className: _indexmoduleless.default['user-msg-text']
    }, loading ? loadingNode : userInfo.organizationName)), _react.default.createElement("div", null, _react.default.createElement(_icon.IconLocation, null), _react.default.createElement("span", {
        className: _indexmoduleless.default['user-msg-text']
    }, loading ? loadingNode : userInfo.locationName))))));
}
const _default = UserInfoHeader;

},
"f9dd033a": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _webreact = farmRequire("050d455e");
const _axios = /*#__PURE__*/ _interop_require_default._(farmRequire("7e09a585"));
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _indexmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("0a457a53"));
const { Paragraph } = _webreact.Typography;
function LatestNews() {
    const [data, setData] = (0, _react.useState)(new Array(4).fill({}));
    const [loading, setLoading] = (0, _react.useState)(true);
    const getData = async ()=>{
        const { data } = await _axios.default.get('/api/user/latestNews').finally(()=>setLoading(false));
        setData(data);
    };
    (0, _react.useEffect)(()=>{
        getData();
    }, []);
    return _react.default.createElement(_webreact.List, {
        dataSource: data,
        render: (item, index)=>{
            return _react.default.createElement(_webreact.List.Item, {
                key: index,
                style: {
                    padding: '24px 20px 24px 0px'
                }
            }, loading ? _react.default.createElement(_webreact.Skeleton, {
                animation: true,
                text: {
                    width: [
                        '60%',
                        '90%'
                    ],
                    rows: 2
                },
                image: {
                    shape: 'circle'
                }
            }) : _react.default.createElement(_webreact.List.Item.Meta, {
                className: _indexmoduleless.default['list-meta-ellipsis'],
                avatar: _react.default.createElement(_webreact.Avatar, {
                    size: 48
                }, _react.default.createElement("img", {
                    src: item.avatar
                })),
                title: item.title,
                description: _react.default.createElement(Paragraph, {
                    ellipsis: {
                        rows: 1
                    },
                    type: "secondary",
                    style: {
                        fontSize: '12px',
                        margin: 0
                    }
                }, item.description)
            }));
        }
    });
}
const _default = LatestNews;

},});