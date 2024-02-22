(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'info_index_055b.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"6fe47e30": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _project = /*#__PURE__*/ _interop_require_default._(farmRequire("bb0fa4ff"));
function MyProject() {
    const [data, setData] = (0, _react.useState)(new Array(6).fill({}));
    const [loading, setLoading] = (0, _react.useState)(true);
    const { Row, Col } = _webreact.Grid;
    const getData = async ()=>{
        setLoading(true);
        const { data } = await _axios.default.get('/api/user/projectList').finally(()=>{
            setLoading(false);
        });
        setData(data);
    };
    (0, _react.useEffect)(()=>{
        getData();
    }, []);
    return _react.default.createElement(Row, {
        gutter: 12
    }, data.map((item, index)=>_react.default.createElement(Col, {
            key: index,
            span: 8,
            style: index > data.length - 4 && index < data.length ? {
                marginTop: '16px'
            } : {}
        }, _react.default.createElement(_project.default, {
            ...item,
            loading: loading
        }))));
}
const _default = MyProject;

},
"8e5823c1": function(module, exports, farmRequire, farmDynamicRequire) {
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
const { Text } = _webreact.Typography;
function MyTeam() {
    const [data, setData] = (0, _react.useState)(new Array(4).fill({}));
    const [loading, setLoading] = (0, _react.useState)(true);
    const getData = async ()=>{
        const { data } = await _axios.default.get('/api/users/teamList').finally(()=>setLoading(false));
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
                style: index !== data.length - 1 ? {
                    padding: '8px 0px'
                } : {
                    padding: '8px 0px 0px 0px'
                }
            }, loading ? _react.default.createElement(_webreact.Skeleton, {
                animation: true,
                text: {
                    width: [
                        '80%',
                        '20%'
                    ],
                    rows: 2
                },
                image: {
                    shape: 'circle'
                }
            }) : _react.default.createElement(_webreact.List.Item.Meta, {
                avatar: _react.default.createElement(_webreact.Avatar, {
                    size: 48
                }, _react.default.createElement("img", {
                    src: item.avatar
                })),
                title: item.name,
                description: _react.default.createElement(Text, {
                    type: "secondary",
                    style: {
                        fontSize: '12px'
                    }
                }, `共${(item.members || 0).toLocaleString()}人`)
            }));
        }
    });
}
const _default = MyTeam;

},
"bb0fa4ff": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _blocksmoduleless = /*#__PURE__*/ _interop_require_default._(farmRequire("b8e34522"));
const { Text, Title } = _webreact.Typography;
function ProjectCard(props) {
    const { loading, contributors } = props;
    return _react.default.createElement(_webreact.Card, {
        className: _blocksmoduleless.default['project-wrapper'],
        bordered: true,
        size: "small"
    }, loading ? _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 1
        },
        animation: true
    }) : _react.default.createElement(Title, {
        heading: 6
    }, props.title), loading ? _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 1
        },
        animation: true,
        style: {
            marginTop: '4px'
        }
    }) : _react.default.createElement(Text, {
        type: "secondary",
        ellipsis: true,
        style: {
            margin: '0'
        }
    }, props.enTitle), _react.default.createElement("div", {
        className: _blocksmoduleless.default.avatar
    }, loading ? _react.default.createElement(_webreact.Skeleton, {
        text: {
            rows: 1
        },
        animation: true
    }) : _react.default.createElement(_react.default.Fragment, null, _react.default.createElement(_webreact.Avatar.Group, {
        size: 24
    }, (contributors || []).map((item, index)=>_react.default.createElement(_webreact.Avatar, {
            key: index
        }, _react.default.createElement("img", {
            src: item.avatar
        })))), _react.default.createElement(Text, {
        type: "secondary",
        className: _blocksmoduleless.default.more
    }, "等", (props.contributorsLength || 0).toLocaleString(), "人"))));
}
const _default = ProjectCard;

},});