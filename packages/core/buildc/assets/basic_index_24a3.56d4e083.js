(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'basic_index_24a3.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"3fec3bea": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("e9860c88"));
function ProfileItem(props) {
    const t = (0, _useLocale.default)(_locale.default);
    const { title, data, type, loading } = props;
    const blockDataList = [];
    blockDataList.push({
        title: t[`basicProfile.title.${type}Video`],
        data: [
            {
                label: t['basicProfile.label.video.mode'],
                value: data?.video?.mode || '-'
            },
            {
                label: t['basicProfile.label.video.acquisition.resolution'],
                value: data?.video?.acquisition.resolution || '-'
            },
            {
                label: t['basicProfile.label.video.acquisition.frameRate'],
                value: `${data?.video?.acquisition.frameRate || '-'} fps`
            },
            {
                label: t['basicProfile.label.video.encoding.resolution'],
                value: data?.video?.encoding.resolution || '-'
            },
            {
                label: t['basicProfile.label.video.encoding.rate.min'],
                value: `${data?.video?.encoding.rate.min || '-'} bps`
            },
            {
                label: t['basicProfile.label.video.encoding.rate.max'],
                value: `${data?.video?.encoding.rate.max || '-'} bps`
            },
            {
                label: t['basicProfile.label.video.encoding.rate.default'],
                value: `${data?.video?.encoding.rate.default || '-'} bps`
            },
            {
                label: t['basicProfile.label.video.encoding.frameRate'],
                value: `${data?.video?.encoding.frameRate || '-'} fpx`
            },
            {
                label: t['basicProfile.label.video.encoding.profile'],
                value: data?.video?.encoding.profile || '-'
            }
        ]
    });
    blockDataList.push({
        title: t[`basicProfile.title.${type}Audio`],
        data: [
            {
                label: t['basicProfile.label.audio.mode'],
                value: data?.audio?.mode || '-'
            },
            {
                label: t['basicProfile.label.audio.acquisition.channels'],
                value: `${data?.audio?.acquisition.channels || '-'} ${t['basicProfile.unit.audio.channels']}`
            },
            {
                label: t['basicProfile.label.audio.encoding.channels'],
                value: `${data?.audio?.encoding.channels || '-'} ${t['basicProfile.unit.audio.channels']}`
            },
            {
                label: t['basicProfile.label.audio.encoding.rate'],
                value: `${data?.audio?.encoding.rate || '-'} kbps`
            },
            {
                label: t['basicProfile.label.audio.encoding.profile'],
                value: data?.audio?.encoding.profile || '-'
            }
        ]
    });
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement("div", null, blockDataList.map(({ title: blockTitle, data: blockData }, index)=>_react.default.createElement(_webreact.Descriptions, {
            key: `${index}`,
            colon: ":",
            labelStyle: {
                textAlign: 'right',
                width: 200,
                paddingRight: 10
            },
            valueStyle: {
                width: 400
            },
            title: blockTitle,
            data: loading ? blockData.map((item)=>({
                    ...item,
                    value: _react.default.createElement(_webreact.Skeleton, {
                        text: {
                            rows: 1,
                            style: {
                                width: '200px'
                            }
                        },
                        animation: true
                    })
                })) : blockData,
            style: index > 0 ? {
                marginTop: '20px'
            } : {}
        }))));
}
const _default = ProfileItem;

},});