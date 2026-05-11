import React, { useState } from "react";
import "./main.css";
import reactLogo from "./assets/react.svg";
import FarmLogo from "./assets/logo.png";

export type WorkerCaseStatus = "ok" | "error";

export interface WorkerCaseResult {
  caseName: string;
  status: WorkerCaseStatus;
  detail: string;
}

interface MainProps {
  workerResults: WorkerCaseResult[];
  running: boolean;
}

export function Main({ workerResults, running }: MainProps) {
  const [count, setCount] = useState(0);

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
      <section className="worker-cases">
        <h2>Worker Cases</h2>
        {running && <p className="worker-running">Running worker checks...</p>}
        <ul>
          {workerResults.map((item) => (
            <li key={item.caseName} className={item.status === "ok" ? "worker-ok" : "worker-error"}>
              <strong>{item.caseName}</strong>: {item.detail}
            </li>
          ))}
        </ul>
      </section>
    </>
  );
}
