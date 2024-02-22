(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'basic_index_2451.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"4e842562": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _setupMock = /*#__PURE__*/ _interop_require_default._(farmRequire("aad0a7a8"));
const _mockjs = /*#__PURE__*/ _interop_require_default._(farmRequire("acd7d5db"));
(0, _setupMock.default)({
    setup: ()=>{
        _mockjs.default.mock(new RegExp("/api/basicProfile"), ()=>{
            return {
                status: 2,
                video: {
                    mode: "自定义",
                    acquisition: {
                        resolution: "720*1280",
                        frameRate: 15
                    },
                    encoding: {
                        resolution: "720*1280",
                        rate: {
                            min: 300,
                            max: 800,
                            default: 1500
                        },
                        frameRate: 15,
                        profile: "high"
                    }
                },
                audio: {
                    mode: "自定义",
                    acquisition: {
                        channels: 8
                    },
                    encoding: {
                        channels: 8,
                        rate: 128,
                        profile: "ACC-LC"
                    }
                }
            };
        });
        _mockjs.default.mock(new RegExp("/api/adjustment"), ()=>{
            return new Array(2).fill("0").map(()=>({
                    contentId: `${_mockjs.default.Random.pick([
                        "视频类",
                        "音频类"
                    ])}${_mockjs.default.Random.natural(1000, 9999)}`,
                    content: "视频参数变更，音频参数变更",
                    status: _mockjs.default.Random.natural(0, 1),
                    updatedTime: _mockjs.default.Random.datetime("yyyy-MM-dd HH:mm:ss")
                }));
        });
    }
});

},
"f0e1de71": function(module, exports, farmRequire, farmDynamicRequire) {
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
    "container": `container-59a65dd6`,
    "steps": `steps-59a65dd6`
};

},});