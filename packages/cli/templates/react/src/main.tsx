import React from 'react';
import { CounterButton } from './comps/counter-button';
import { Description } from './comps/description';
import './main.css';

export function Main() {
  return (
    <>
      <div className='button-wrapper'>
        <CounterButton />
      </div>
      <div>
        <Description />
      </div>
    </>
  );
}
