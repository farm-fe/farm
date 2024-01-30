import { A2, A1 } from './exportAll';

export default function B1() {
  console.log('b1');
}

export function B2() {
  A2();
  A1();
  console.log('b2');
}
