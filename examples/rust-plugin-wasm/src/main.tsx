import { useState } from "react";
import "./main.css";
import reactLogo from "./assets/react.svg";
import FarmLogo from "./assets/logo.png";
import init from "./assets/json_typegen_wasm_bg.wasm?init";
import { run } from "json_typegen_wasm"

export function Main() {
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
  "name": "example",
  "version": "1.0.0",
  "type": "module"
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
