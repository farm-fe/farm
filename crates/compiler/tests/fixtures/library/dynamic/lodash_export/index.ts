import lodash, { merge } from './loadash.cjs';

console.log(lodash, merge);

import('./bundle2').then(mod => {
  console.log(mod)
})
import('./bundle3').then(mod => {
  console.log(mod)
})