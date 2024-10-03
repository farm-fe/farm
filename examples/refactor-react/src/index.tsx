import React from 'react';
import { createRoot } from 'react-dom/client';
import { Main } from './main';
import './index.css'


const container = document.querySelector('#root');
// biome-ignore lint/style/noNonNullAssertion: <explanation>
const root = createRoot(container!);

root.render(<Main />)
