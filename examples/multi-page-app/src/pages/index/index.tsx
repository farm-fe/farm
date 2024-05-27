import React from 'react';
import { createRoot } from 'react-dom/client';
import './index.scss';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(
  <>
    <p><a href="/about?type=1&id=farm">About?</a></p>
    <p><a href="/about#hash">About#</a></p>
    <div>Index page</div>
  </>
);
