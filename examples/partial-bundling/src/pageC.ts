import util2 from './util2';
import util3 from './util3';
import vendor2 from 'vendor2';

import('./dynamic').then((res) => {
  console.log(res.default);
});

export default 'pageC';

console.log(util3, util2, vendor2);
