import { name as names } from './test';

export const a: number = 1;

export function b<T extends string>(name: string): T {
  return name + names + '123' as T;
}
