import { a as A, b as B, default as C } from './As';

import namedDefault, { a, b, c, d } from './named';

console.log(namedDefault, a, b, c, d, A, B, C);
