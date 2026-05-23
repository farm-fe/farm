import { createRoot } from 'react-dom/client';
import { Main } from './main';
import './index.css'
import defaultValue from "test.js"
import "test1.js"

console.log(defaultValue);

const container = document.querySelector('#root') as Element;
const root = createRoot(container);

root.render(<Main />);
