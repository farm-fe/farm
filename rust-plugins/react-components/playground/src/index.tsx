import React from 'react';
import { createRoot } from 'react-dom/client';
import { Main } from './test';
import './index.css'


const container = document.querySelector('#root');
const root = createRoot(container);

root.render(<Main />);
