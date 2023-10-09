import { createSignal } from 'solid-js';
import styles from './App.module.css';

import type { Component } from 'solid-js';

const App: Component = () => {
  const [count, setCount] = createSignal(0);

  return (
    <div class={styles.App}>
      <button class={styles.counter} onClick={() => setCount(count() + 1)}>
        Count: {count()}
      </button>
    </div>
  );
};

export default App;
