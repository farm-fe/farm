import { h } from './vue-core';
export * from './vue-core';

export function render() {
  return h('div', {}, 'hello world');
}