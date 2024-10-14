import React from 'react';
import { render } from 'react-dom';
import { Main } from './main';
import './index.css'


const container = document.querySelector('#root');

render(<Main />,container);
