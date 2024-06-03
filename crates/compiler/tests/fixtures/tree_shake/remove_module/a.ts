export * from './c';

import { b } from './b';

export const appendB = () => b + 'a';

export function appendA() {
  return 'a';
}
