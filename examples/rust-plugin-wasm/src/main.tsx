import { useState } from "react";
import "./main.css";
import reactLogo from "./assets/react.svg";
import FarmLogo from "./assets/logo.png";
import init from "./assets/json_typegen_wasm_bg.wasm?init";
import { greet } from "rust-wasm"
import { run } from "json_typegen_wasm"

export function Main() {
  greet();
  const [count, setCount] = useState(0);

  const transformToInterface = async (json: string) => {
    return run(
      'Root',
      json,
      JSON.stringify({
        output_mode: 'typescript'
      })
    )
  }

  const json = `{
  "name": "playground",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "farm start",
    "start": "farm start",
    "build": "farm build",
    "preview": "farm preview",
    "clean": "farm clean"
  },
  "dependencies": {
    "@farmfe/plugin-wasm": "workspace:*",
    "clsx": "^1.2.1",
    "json_typegen_wasm": "^0.7.0",
    "react": "18",
    "react-dom": "18"
  },
  "devDependencies": {
    "@farmfe/cli": "^1.0.2",
    "@farmfe/core": "^1.3.0",
    "@farmfe/plugin-react": "^1.2.0",
    "@types/react": "18",
    "@types/react-dom": "18",
    "core-js": "^3.36.1",
    "react-refresh": "^0.14.0"
  }
}
`

  const transform = async () => {
    const interfaces = await transformToInterface(json)
    console.log('%c [ interface ]-46', 'font-size:13px; background:rgba(66, 184, 131, 0.2); color:#05a15b;', interfaces)
  }

  transform();

  init({}).then((wasm) => {
    console.log('Loaded json_typegen_wasm by wasm init: ', wasm)
  })
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
