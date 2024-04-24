import { useState } from 'preact/hooks';
import preactLogo from './assets/preact.svg';
import FarmLogo from './assets/logo.png';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';

function App() {
  const [greetMsg, setGreetMsg] = useState('');
  const [name, setName] = useState('');

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke('greet', { name }));
  }

  return (
    <div class="container">
      <h1>Welcome to Tauri!</h1>

      <div class="row">
        <a href="https://farmfe.org/" target="_blank">
          <img src={FarmLogo} class="logo" alt="Farm logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://preactjs.com" target="_blank">
          <img src={preactLogo} class="logo preact" alt="Preact logo" />
        </a>
      </div>

      <p>Click on the Tauri, Farm, and Preact logos to learn more.</p>

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onInput={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <p>{greetMsg}</p>
    </div>
  );
}

export default App;
