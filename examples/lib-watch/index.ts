import { watch } from './main.ts';

console.log(watch);

import('./dynamic').then((mod) => {
  console.log('mod.default', mod.default);
});
