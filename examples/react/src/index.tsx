import React from 'react';
import { createRoot } from 'react-dom/client';
import { Main } from './main';
import './index.scss';

const container = document.querySelector('#root');
const root = createRoot(container);
console.log(import.meta);

root.render(<Main />);
