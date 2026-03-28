import { add, multiply } from './math';

export function main() {
  return add(1, 2) + ' ' + multiply(3, 4);
}

export const loadGreeting = () => import('./greeting');

export { add, multiply } from './math';
