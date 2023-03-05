
import React, { Suspense } from 'react';
import { CounterButton } from './comps/counter-button';
import { Description } from './comps/description';
import './main.css';

export function Main() {
  return <><CounterButton /><Description /><div></div></>
}
