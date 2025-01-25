import { name as names } from './test';
import type { base } from './test';
export const a: number = 1;

// 我是奥特曼
export function b<T extends string>(name: string): T {
  return name + names + '123' as T;
}
