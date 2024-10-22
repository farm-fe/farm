import { createSignal } from 'solid-js';

import logo from './logo.svg?url';
import styles from './App.module.css';

import type { Component } from 'solid-js';

import Button from './button';

const App: Component = () => {
  const [count, setCount] = createSignal(0);

  return (
    <div class={styles.App}>
      <header class={styles.header}>
        <img src={logo} class={styles.logo} alt="logo" />
        <p>
          Edit<code>src/App.tsx</code>1232131231231221312321 and save to reload.
        </p>
        <a
          class={styles.link}
          href="https://github.com/solidjs/solid"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn Solid
        </a>
        <button class={styles.counter} onClick={() => setCount(count() + 1)}>
          Count parent: {count()}
        </button>

        <Button />
      </header>
    </div>
  );
};

export default App;
