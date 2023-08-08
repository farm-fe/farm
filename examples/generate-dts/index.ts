import { example } from './generate.js';
import type { myTuple } from './type.js';
interface obj {
  name: string;
  age: number;
}
const res: myTuple = [1, 2, 3, '4'];
export const obj: obj = {
  name: 'erkelost',
  age: 18
};

console.log(example, obj, res);
