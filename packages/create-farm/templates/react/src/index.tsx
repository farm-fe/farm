import React from 'react';
import { createRoot } from 'react-dom/client';
import { Main } from './main';

const container = document.querySelector('#root');

if (container) {
  const root = createRoot(container);
  root.render(<Main />);
}
