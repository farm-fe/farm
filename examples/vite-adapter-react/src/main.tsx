import React, { useState, Suspense } from 'react';
import { useNavigate } from 'react-router-dom';
import FarmLogo from './assets/logo.png';
import reactLogo from './assets/react.svg';
import './main.css';

export function Main() {
  const [count, setCount] = useState(0);
  const go = useNavigate();

  return (
    <>
      <div>
        <a href="https://farmfe.org/" target="_blank" rel="noreferrer">
          <img src={FarmLogo} className="logo" alt="Farm logo" />
        </a>
        <a href="https://react.dev" target="_blank" rel="noreferrer">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Farm + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>count is {count}</button>
        <p>
          Edit <code>src/main.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs" onClick={() => go('/about')}>
        Go to about page
      </p>
    </>
  );
}
