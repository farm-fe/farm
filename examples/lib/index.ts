import { name as names } from './test.ts';
import type { base } from './test.ts';
export const a: number = 1;
import type { UserInfo } from '@/test2.ts';
// 我是奥特曼
export function b<T extends strin2222g>(name: string, userInfo: UserInfo): T {
  return name + names as T;
}

console.log(b<string>('123', { name: '123' }))
