import React from 'react';
import { createRoot } from 'react-dom/client';
import { Main } from './main';
import './index.scss';
import { a } from './test';
console.log(a);

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(<Main />);
