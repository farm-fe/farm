/// Transform asy module to meet the requirements of farm runtime
/// Example, transform:
/// ```js
/// const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
/// const _sync = _interop_require_default._(farmRequire("475776c7"));
/// const _dep2 = _interop_require_default._(farmRequire("ea236e3d"));
/// ```
/// To:
/// ```js
/// let [_interop_require_default, _sync, _dep2] = await Promise.all([
///   farmRequire("@swc/helpers/_/_interop_require_default"),
///   farmDynamicRequire("475776c7"),
///   farmRequire("ea236e3d")
/// ]);
/// _interop_require_default = _interop_require_default;
/// _sync = _interop_require_default._(_sync);
/// _dep2 = _interop_require_default._(_dep2);
/// ```

pub fn transform_async_module() {}
