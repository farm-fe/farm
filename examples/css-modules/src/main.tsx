import React from 'react';
import './main.css';

import { Child } from './child';

export function Main() {
  return (
    <div className={'main'}>
      main <Child />
    </div>
  );
}
