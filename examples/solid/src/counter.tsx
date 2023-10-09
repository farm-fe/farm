import { createSignal } from 'solid-js';
import { customElement } from 'solid-element';

customElement('my-counter', () => {
  const [count, setCount] = createSignal(0);
  return (
    <>
      <div>
        <button onClick={() => setCount(count() - 1)}>-</button>
        <span>{count()}</span>
        <button onClick={() => setCount(count() + 1)}>+</button>
      </div>
    </>
  );
});