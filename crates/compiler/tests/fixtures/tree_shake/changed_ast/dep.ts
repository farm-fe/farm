export const Suspense = function () {
  console.log('Suspense');
}

export const renderToString = function () {
  console.log('renderToString');
}

export default {
  createElement(comp, ...args) {
    console.log(comp(), args);
  },
  lazy: (promise) => {
    console.log('lazy', promise);
  }
}