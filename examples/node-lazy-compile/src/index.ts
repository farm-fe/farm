console.log('script start');

import('./dynamic.js').then((mod) => {
  console.log('111' + mod.a);
});
