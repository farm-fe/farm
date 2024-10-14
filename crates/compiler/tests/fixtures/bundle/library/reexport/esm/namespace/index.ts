export * as ns from './foo';

import * as ns from './foo';

const foo = 123;

console.log(ns.default, ns.foo, foo);