import { useState } from "react";
import "./main.css";
import reactLogo from "/react.svg";
import FarmLogo from "./assets/logo.png";
// import { a } from './a.js'
// import { Button } from 'antd'
// import { HappyProvider } from '@ant-design/happy-work-theme';
export function Main() {
  const [count, setCount] = useState(0);
  // console.log(a);

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
      {/* <HappyProvider> */}
        {/* <Button type="primary">Primary Button</Button> */}
      {/* </HappyProvider> */}
      <h1>Farm + react</h1>
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
