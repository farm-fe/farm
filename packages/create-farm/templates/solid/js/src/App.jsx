import { createSignal } from "solid-js";
import solidLogo from "./assets/solid.svg";
import farmLogo from "./assets/logo.png";
import "./App.css";

function App() {
  const [count, setCount] = createSignal(0);

  return (
    <>
      <div>
        <a href="https://farmfe.org/" target="_blank">
          <img src={farmLogo} class="logo" alt="Farm logo" />
        </a>
        <a href="https://solidjs.com" target="_blank">
          <img src={solidLogo} class="logo solid" alt="Solid logo" />
        </a>
      </div>
      <h1>Farm + Solid</h1>
      <div class="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count()}
        </button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p class="read-the-docs">
        Click on the Farm and Solid logos to learn more
      </p>
    </>
  );
}

export default App;
