import React from './react';
import ReactDom from './react-dom';

import { App } from './app';

ReactDom.render(
  <App />,
  document.getElementById('root')
);

// // ---->
// const react = _interop_default_(farmRequire('./react'));
// const util1 =  _interop_default_(farmRequire('./util1.cjs'));

// function common1() {
//   return 'common';
// }
// common1();
// var commonInner = common1;

// function common() {
//   return 'common-outer';
// }

// function App() {
//   return (
//     react.createElement(
//       'div',
//       null,
//       [
//         react.createElement(
//           'h1',
//           null,
//           util1.util1() + util1.util2()
//         ),
//         react.createElement(
//           'h1',
//           null,
//           commonInner()
//         )
//       ]
//     )
//   );
// }