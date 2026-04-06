import { useState, useEffect } from "react";
import "./main.css";
import reactLogo from "./assets/react.svg";
import FarmLogo from "./assets/logo.png";

export function Main() {
  const [count, setCount] = useState(0);
  const [wasmResult, setWasmResult] = useState('Loading WASM result...');

  useEffect(() => {
    const json = `{
  "name": "example",
  "version": "1.0.0",
  "type": "module"
}
`
    import("json_typegen_wasm").then(async (wasmMod) => {
      const mod = await wasmMod;
      const defaultExport = await (mod as { default?: unknown }).default;
      const maybeRun =
        typeof (defaultExport as { run?: unknown } | undefined)?.run ===
        'function'
          ? (defaultExport as {
              run: (name: string, input: string, options: string) => string;
            }).run
          : undefined;

      if (typeof maybeRun === 'function') {
        try {
          const interfaces = maybeRun(
            'Root',
            json,
            JSON.stringify({ output_mode: 'typescript' })
          );
          setWasmResult(interfaces);
        } catch {
          const payloadSummary =
            defaultExport && typeof defaultExport === 'object'
              ? `Resolved WASM payload keys: ${Object.keys(defaultExport).join(', ')}`
              : `Resolved WASM payload: ${String(defaultExport)}`;
          setWasmResult(payloadSummary);
        }
        return;
      }

      const payloadSummary =
        defaultExport && typeof defaultExport === 'object'
          ? `Resolved WASM payload keys: ${Object.keys(defaultExport).join(', ')}`
          : `Resolved WASM payload: ${String(defaultExport)}`;
      setWasmResult(payloadSummary);
    }).catch((e) => {
      console.warn('WASM initialization error:', e);
      setWasmResult(`WASM initialization error: ${String(e)}`);
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
        <pre data-testid="wasm-result">{wasmResult}</pre>
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
