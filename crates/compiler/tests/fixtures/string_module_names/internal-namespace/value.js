export let value = 1;
export { value as "live-value" };
export function increment() { value++; }
