import React, { useState } from "react";
import "./main.css";
import ReactLogo from "./assets/react.svg";
import logo from "./assets/react.svg?img";
import FarmLogo from "./assets/logo.png";
export function Main() {
  const [count, setCount] = useState(0);
  console.log("rendering Main component")
  return (
    <>
      <div>
        <ReactLogo
          width={80}
          height={80}
          onClick={() => console.log('clicked')}
        />
        <a href="https://farmfe.org/" target="_blank">
          <img src={FarmLogo} className="logo" alt="Farm logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={logo} className="logo react" alt="React logo" />
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
