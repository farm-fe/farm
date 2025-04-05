// ---->
function (module, exports, farmRequire, farmDynamicRequire) {
  var react = _interop_default_(farmRequire('./react'));
  var util1 =  _interop_default_(farmRequire('./util1.cjs'));

  // ---- content of common.mjs ----
  function common1() {
    return 'common';
  }
  common1();
  var commonInner = common1;
  // ---- end of common.mjs ----

  function common() {
    return 'common-outer';
  }

  module.o(exports, 'App', function App() {
    return (
      react.createElement(
        'div',
        null,
        [
          react.createElement(
            'h1',
            null,
            util1.util1() + util1.util2()
          ),
          react.createElement(
            'h1',
            null,
            commonInner()
          )
        ]
      )
    );
  })
}
