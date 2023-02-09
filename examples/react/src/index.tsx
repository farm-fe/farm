import React from 'react';
import { createRoot } from 'react-dom/client';

import { Main } from './main';
const container = document.querySelector('#root')!;
const root = createRoot(container);

console.log(123)
root.render(<Main />)