import { bundle_str1 } from './bundle2';

console.log(bundle_str1);

export const index_foo = 'index foo';
export const index_bar = 'index bar';

const foo_str1 = 123;
const foo_str3 = 'index-foo_str3';
console.log(foo_str1, foo_str3);

export default 'index default';

// export * from './bundle2';