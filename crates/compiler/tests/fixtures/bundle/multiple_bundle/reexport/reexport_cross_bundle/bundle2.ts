import { foo_str1, foo_str2 } from './foo';
import './bundle2-foo';

console.log({ foo_str1, foo_str2 });

export const bundle_str1 = 'bundle str1';
export const bundle_str2 = 'bundle str2';