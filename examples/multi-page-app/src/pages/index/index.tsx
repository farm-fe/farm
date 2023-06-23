import React from 'react';
import { createRoot } from 'react-dom/client';
import './index.scss';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(
  <>
    <a href="/about">About</a>
    <div>Index page</div>
  </>
);
