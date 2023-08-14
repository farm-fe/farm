import type { TestBase } from './test';
import { a } from './bbb';
console.log('我是打包哦');
console.log(a);

export interface Test extends TestBase {
  count: number;
}

// export { testFn } from './comment'
// export { Decorator } from './decorator'
// export { test, method } from './test'
// export { addOne, add } from '@/js-test.js'
// export { ESClass } from './es-class'
// export { manualDts } from './manual-dts'

// export type { User } from './types'
