import { useState } from "preact/hooks";
import preactLogo from "./assets/preact.svg";
import FarmLogo from "./assets/logo.png";
import "./app.css";

import { Page } from "@patternfly/react-core";

export function App() {
  const [count, setCount] = useState(0);
  console.log("hello, patternfly!");
  return (
    <>
      <Page>
        <h1>hello, patternfly!</h1>
        <div>
          <a href="https://farmfe.org/" target="_blank">
            <img src={FarmLogo} class="logo" alt="Farm logo" />
          </a>
          <a href="https://preactjs.com" target="_blank">
            <img src={preactLogo} class="logo preact" alt="Preact logo" />
          </a>
        </div>
        <h1>Farm + Preact 123</h1>
        <div class="card">
          <button onClick={() => setCount((count) => count + 1)}>
            count is {count}
          </button>
          <p>
            Edit <code>src/app.tsx</code> and save to test HMR
          </p>
        </div>
        <p class="read-the-docs">
          Click on the Farm and Preact logos to learn more
        </p>
      </Page>
    </>
  );
}
