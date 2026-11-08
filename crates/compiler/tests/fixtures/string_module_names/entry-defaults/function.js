export let value = 1;
export {value as "mutable-value"};
export default function read() { return value; }
export function increment() { value++; }
