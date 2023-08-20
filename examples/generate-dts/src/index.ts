import type { TestBase } from './test';
import { a } from './bbb';
console.log(a);

export interface Test extends TestBase {
  count: number;
}
