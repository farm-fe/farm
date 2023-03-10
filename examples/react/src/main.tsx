
import React from 'react';
import { CounterButton } from './comps/counter-button';
import { Description } from './comps/description';
import './main.css';

import { Title } from './comps/title';

export function Main() {
  return <><Title /><CounterButton /><Description /><div></div></>
}
