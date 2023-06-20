import React from 'react';
import { createRoot } from 'react-dom/client';
import './index.scss';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(
  <>
    <a href="/">Index</a>
    <div>about page</div>
  </>
);
