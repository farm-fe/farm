import { Provider } from 'react-redux';
import React from 'react';
import { createRoot } from 'react-dom/client';
import { Main } from './main';
import { store } from './store';

import './index.scss';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(
  <Provider store={store}>
    <Main />
  </Provider>
);
