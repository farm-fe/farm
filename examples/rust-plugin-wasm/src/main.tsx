import { useState, useEffect } from "react";
import "./main.css";
import reactLogo from "./assets/react.svg";
import FarmLogo from "./assets/logo.png";

export function Main() {
  const [count, setCount] = useState(0);

  useEffect(() => {
    const json = `{
  "name": "example",
  "version": "1.0.0",
  "type": "module"
}
`
    import("json_typegen_wasm").then(async ({ default: init, run }) => {
      await init();
      const interfaces = run('Root', json, JSON.stringify({ output_mode: 'typescript' }));
      console.log('%c [ interface ]-46', 'font-size:13px; background:rgba(66, 184, 131, 0.2); color:#05a15b;', interfaces);
    }).catch((e) => {
      console.warn('WASM initialization error:', e);
    });
  }, []);

  return (
    <>
      <div>
        <a href="https://farmfe.org/" target="_blank">
          <img src={FarmLogo} className="logo" alt="Farm logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Farm + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <p>
          Edit <code>src/main.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Farm and React logos to learn more
      </p>
    </>
  );
}
