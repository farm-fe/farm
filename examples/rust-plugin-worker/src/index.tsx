import { createRoot } from 'react-dom/client';
import { Main } from './main';
// import TestWorker from "./worker/test.worker?worker"
import './index.css'

// console.log(TestWorker);
// const worker = new TestWorker();
const worker = new Worker(new URL("/src/worker/test.worker.ts",import.meta.url));
worker.postMessage([5, 5]);
worker.onmessage = (e) => {
  console.log('test worker', e.data);
}
const worker2 = new Worker(new URL("./worker/vue.worker.ts",import.meta.url))


worker2.postMessage([2, 3]);
worker2.onmessage = (e) => {
  console.log('vue worker', e.data);
}

const container = document.querySelector('#root');
const root = createRoot(container!);

root.render(<Main />);
