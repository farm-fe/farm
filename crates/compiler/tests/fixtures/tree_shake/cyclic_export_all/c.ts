export const c1 = 1;
export const c2 = 1;
export const c3 = 1;

import * as aaaa from './b';
export { c3 as c4 } from './b';

var b = aaaa.c1;
