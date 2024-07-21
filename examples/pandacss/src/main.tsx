import React, { useState } from "react";
import {css} from "@styled-system/css";
export function Main() {
  const [count, setCount] = useState(0);

  return (
    <>
      <h1>Farm + React</h1>
      <div className={css({
          bg: "red.500",
          color: "white",
          w: "md"
      })}>
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
