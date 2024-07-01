import React, { Suspense } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter as Router, useRoutes } from 'react-router-dom';
import './index.css';
import routes from '~react-pages';

const container = document.querySelector('#root');
const root = createRoot(container);

function App() {
  console.log('this is a react app');
  return <Suspense fallback={<p>Loading...</p>}>{useRoutes(routes)}</Suspense>;
}

root.render(
  <Router>
    <App />
  </Router>,
);
