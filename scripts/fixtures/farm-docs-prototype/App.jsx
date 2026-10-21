import snapshot from 'docs:snapshot';
import React, { useState } from 'react';
import './style.css';
import styles from './style.module.css';

export default function App() {
  const [count, setCount] = useState(0);
  return <main data-snapshot={snapshot.snapshotId}>
    <h1>Farm Docs prototype</h1>
    <p className={styles.message}>Static content rendered by the Farm server bundle.</p>
    <button onClick={() => setCount(value => value + 1)}>Count {count}</button>
  </main>;
}
