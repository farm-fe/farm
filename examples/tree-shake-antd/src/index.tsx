import React from 'react';
import { createRoot } from 'react-dom/client';
import './global';
import { Button } from '../build/index';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(<Button className="test-antd-button" onClick={() => {
  console.log('antd button clicked')
}}>Antd Button</Button>);
