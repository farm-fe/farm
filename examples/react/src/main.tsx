
import React, { Suspense } from 'react';
import { CounterButton } from './comps/counter-button';
import { Description } from './comps/description';
import './main.css';

const Clock = React.lazy(() => import('./comps/clock'));

export function Main() {
  return <Suspense fallback={'loading...'}><CounterButton /><Description /><div><Clock /></div></Suspense>
}
