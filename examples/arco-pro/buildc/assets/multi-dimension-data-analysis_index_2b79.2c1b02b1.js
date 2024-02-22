(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'multi-dimension-data-analysis_index_2b79.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"7027c3d5": function(module, exports, farmRequire, farmDynamicRequire) {
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
const _bizcharts = farmRequire("f36f3472");
const _react = farmRequire("a0fc9dfd");
const _reactredux = farmRequire("e429bf23");
const defaultDarkTheme = _bizcharts.G2.getTheme("dark");
_bizcharts.G2.registerTheme("darkTheme", {
    ...defaultDarkTheme,
    background: "transparent"
});
function useBizTheme() {
    const theme = (0, _reactredux.useSelector)((state)=>state.theme);
    const themeName = theme === "dark" ? "darkTheme" : "light";
    const [themeObj, setThemeObj] = (0, _react.useState)(_bizcharts.G2.getTheme(themeName));
    (0, _react.useEffect)(()=>{
        const themeName = theme === "dark" ? "darkTheme" : "light";
        const newTheme = _bizcharts.G2.getTheme(themeName);
        setThemeObj(newTheme);
    }, [
        theme
    ]);
    return themeObj;
}
const _default = useBizTheme;

},});