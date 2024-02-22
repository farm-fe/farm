(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'welcome_index_5e2d.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"4477845a": function(module, exports, farmRequire, farmDynamicRequire) {
// https://github.com/feross/clipboard-copy/blob/master/index.js
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return clipboard;
    }
});
function clipboard(text) {
    if (navigator.clipboard) {
        return navigator.clipboard.writeText(text).catch(function(err) {
            throw err !== undefined ? err : new DOMException("The request is not allowed", "NotAllowedError");
        });
    }
    const span = document.createElement("span");
    span.textContent = text;
    span.style.whiteSpace = "pre";
    document.body.appendChild(span);
    const selection = window.getSelection();
    const range = window.document.createRange();
    selection.removeAllRanges();
    range.selectNode(span);
    selection.addRange(range);
    let success = false;
    try {
        success = window.document.execCommand("copy");
    } catch (err) {
        // eslint-disable-next-line
        console.log("error", err);
    }
    selection.removeAllRanges();
    window.document.body.removeChild(span);
    return success ? Promise.resolve() : Promise.reject(new DOMException("The request is not allowed", "NotAllowedError"));
}

},
"8f54af56": function(module, exports, farmRequire, farmDynamicRequire) {
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
const i18n = {
    "en-US": {
        "welcome.title.welcome": "Welcome",
        "welcome.invite": "Arco Cli commands can be used to install materials from the material market, and we sincerely invite you to experience it.",
        "welcome.usage": "Usage",
        "welcome.step.title.pickup": "Select materials from the material market",
        "welcome.step.title.install": "Install",
        "welcome.step.title.result": "Result",
        "welcome.step.content.pickup": "For example, if you want the workplace page of pro, you can get the package name of the material from the material details",
        "welcome.step.content.install": "After getting the package name, you can install the material through the following command.",
        "welcome.step.content.result": "Then, you get a workplace page easily.",
        "welcome.title.material": "For more materials, please check the following link",
        "welcome.link.material-pro": "Arco Design Pro material collection",
        "welcome.link.material-all": "All materials"
    },
    "zh-CN": {
        "welcome.title.welcome": "欢迎",
        "welcome.invite": "通过 Arco Cli 命令可以安装物料市场的物料，诚邀您体验。",
        "welcome.usage": "使用方式",
        "welcome.step.title.pickup": "从物料市场选择物料",
        "welcome.step.title.install": "安装物料",
        "welcome.step.title.result": "成果",
        "welcome.step.content.pickup": "例如您看中了 pro 的 workplace 页面，可以从物料详情中获得该物料的包名",
        "welcome.step.content.install": "得到包名后，您就可以通过如下命令安装该物料",
        "welcome.step.content.result": "这样您就能轻松获得一个 workplace 页面",
        "welcome.title.material": "更多物料请查看以下链接",
        "welcome.link.material-pro": "Arco Design Pro 物料合集",
        "welcome.link.material-all": "所有物料"
    }
};
const _default = i18n;

},});