import './style.css';
import typescriptLogo from './typescript.svg';
import Logo from './assets/logo.png';
import { setupCounter } from './counter.ts';

// Use contextBridge
window.ipcRenderer.on('main-process-message', (_event, message) => {
  console.log('Main process active push message:', message);
});

const appElement = document.querySelector<HTMLDivElement>(
  '#app'
) as HTMLDivElement;

appElement.innerHTML = `
  <div>
    <a href="https://farmfe.org/" target="_blank">
      <img src="${Logo}" class="logo" alt="Vite logo" />
    </a>
    <a href="https://www.typescriptlang.org/" target="_blank">
      <img src="${typescriptLogo}" class="logo vanilla" alt="TypeScript logo" />
    </a>
    <h1>Electron + Farm + TypeScript</h1>
    <div class="card">
      <button id="counter" type="button"></button>
    </div>
    <p class="read-the-docs">
      Click on the Farm and TypeScript logos to learn more
    </p>
  </div>
`;

setupCounter(document.querySelector('#counter') as HTMLButtonElement);
