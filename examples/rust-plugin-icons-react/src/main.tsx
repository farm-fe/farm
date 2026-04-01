import React, { useState } from "react";
import "./main.css";
import FarmLogo from "./assets/logo.png";
import VueLogoIconifyRaw from '~icons/logos/vue?raw'
import RemoteComponent from "~icons/remote/react";
import ReactLogoIconify from '~icons/logos/react?width=2em&height=2em'
import ReactLogoComponent from "./assets/react.svg?component";
import LocalReactLogo from "~icons/local/react";
export function Main() {
  const [count, setCount] = useState(0);
  console.log("rendering Main component")
  console.log(VueLogoIconifyRaw);
  return (
    <>
      <div>
        <a href="https://farmfe.org/" target="_blank">
          <img src={FarmLogo} className="logo" alt="Farm logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          {/* <img src={reactLogo} className="logo react" alt="React logo" /> */}
        </a>
        <div style={{ display: 'flex', alignItems: "center" }}>
          <div className="i-logos-react text-100px text-#00D8FF"></div>
          <ReactLogoIconify className="text-100px text-#00D8FF" />
          <ReactLogoComponent className="text-100px h-1em w-1em" />
          <LocalReactLogo />
          <RemoteComponent className="text-100px h-1em w-1em" />
          <div dangerouslySetInnerHTML={
            {
              __html: VueLogoIconifyRaw
            }
          } className="text-100px h-1em w-1em"></div>
        </div>
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
