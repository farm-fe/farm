import './style.css';
import Logo from './assets/logo.png';
import { setupCounter } from './counter.ts';
import typescriptLogo from './typescript.svg';
import { read_the_docs, card, logo } from './styles.css.ts';

const app = document.querySelector('#app') as HTMLDivElement;

app.innerHTML = `
  <div>
    <a href="https://farmfe.org/" target="_blank">
      <img src="${Logo}" class="${logo}" alt="Vite logo" />
    </a>
    <a href="https://www.typescriptlang.org/" target="_blank">
      <img src="${typescriptLogo}" class="${logo} vanilla" alt="TypeScript logo" />
    </a>
    <h1>Farm + TypeScript</h1>
    <div class="${card}">
      <button id="counter" type="button"></button>
    </div>
    <p class="${read_the_docs}">
      Click on the Farm and TypeScript logos to learn more
    </p>
  </div>
`;

setupCounter(document.querySelector('#counter') as HTMLButtonElement);
