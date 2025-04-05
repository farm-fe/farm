import React from 'react';
import logo from './logo.svg';
import { useState } from 'react';

const App = () => {
  const [count, setCount] = useState(0);

  return (
    <div className="text-center">
      <header className="bg-slate-700 min-h-screen flex flex-col items-center justify-center text-[calc(10px + 2vmin)] text-white">
        <img
          src={logo}
          className="animate-spin h-[20vmin] pointer-events-none"
          alt="logo"
        />
        <p>
          Edit <code>src/App.tsx</code> and save to reload.
        </p>
        <a
          className="text-blue-300 shadow"
          href="https://github.com/tailwindcss/tailwindcss"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn Tailwindcss
        </a>
        <button className="mt-6" onClick={() => setCount(count + 1)}>
          Count: {count}
        </button>
      </header>
    </div>
  );
};

export default App;
