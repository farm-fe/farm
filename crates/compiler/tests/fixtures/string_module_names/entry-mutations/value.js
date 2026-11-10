export let value = 1;
export let big = 1n;
export let rest;
export default value;
export function mutate(observe) {
  const check = expected => observe(expected, big, rest);
  if (value++ !== 1) throw new Error('postfix'); check(2);
  if (++value !== 3) throw new Error('prefix'); check(3);
  value += 2; check(5);
  let calls = 0;
  value ||= ++calls; check(5);
  value &&= 6; check(6);
  value ??= ++calls; check(6);
  if (calls !== 0) throw new Error('short circuit');
  value = null; value ??= 7; check(7);
  ({ value = 8 } = {}); check(8);
  ({ value, ...rest } = { value: 9, extra: 1 }); check(9);
  [value, ...rest] = [10, 11, 12]; check(10);
  for (value of [13, 14]) check(value);
  for (value in { a: 0, b: 0 }) check(value);
  let ignored;
  const broken = { [Symbol.iterator]() { let next = 0; return { next() { if (next++) throw new Error('iterator'); return { value: 42, done: false }; } }; } };
  try { [value, ignored] = broken; } catch (error) { if (error.message !== 'iterator') throw error; }
  check(42);
  ([{ x: value = 43 }] = [{}]); check(43);
  function shadow() { let value = 99; value++; return value; }
  if (shadow() !== 100) throw new Error('shadow'); check(43);
  if (big++ !== 1n) throw new Error('big postfix'); check(43);
}
