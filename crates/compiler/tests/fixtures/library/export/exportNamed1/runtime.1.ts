import { a as A, b as B } from './runtime.2';

const a = 'a-runtime.1.ts';
const b = 'b-runtime.1.ts';

function BB() {
  const a = 5;
  const b = 6;
  console.log(a, b);
}

console.log(a, b, A, B);

export { a, b };
