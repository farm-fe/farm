/* @refresh reload */
import { render, Pro } from 'solid-js/web';
import { Router, Route } from '@solidjs/router';

import './index.css';
import App from './App';
import { JSXElement } from 'solid-js';

const root = document.getElementById('root');

if (__DEV__ && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?'
  );
}

const Layout = ({ children }: { children: JSXElement }) => (
  <div style={{ 'text-align': 'center' }}>
    <a href="/" style={{ 'margin-right': '10px' }}>
      Home
    </a>
    <a href="/about">About</a>
    {children}
  </div>
);

render(
  () => (
    <Router>
      <Route
        path="/"
        component={() => (
          <Layout>
            <App />
          </Layout>
        )}
      />
      <Route
        path="/about"
        component={() => (
          <Layout>
            <div>About</div>
          </Layout>
        )}
      />
    </Router>
  ),
  root!
);
