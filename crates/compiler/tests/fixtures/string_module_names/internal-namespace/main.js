import * as ns from './value.js';
import { value, increment } from './value.js';
globalThis.__farmNamespace = ns;
export function verify() {
  const key = globalThis.__farmNamespaceKey || 'value';
  const before = ns[key];
  increment();
  if (ns[key] !== value || ns['live-value'] !== value) throw new Error('namespace snapshot');
  return [before, ns[key], value, typeof Object.getOwnPropertyDescriptor(ns, key).get === 'function'];
}
