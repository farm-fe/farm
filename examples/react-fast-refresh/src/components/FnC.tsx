import { useState } from 'react';

export function FnC() {
  const [count, setCount] = useState(0);
  return <div>hello function component<button data-refresh-counter onClick={() => setCount(value => value + 1)}>count: {count}</button></div>;
}
