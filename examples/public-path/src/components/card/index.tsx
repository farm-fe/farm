import React, { PropsWithChildren } from 'react';
import { defineFarmConfig } from '../../original-sourcemap/config';
import './index.css';

export function FarmCard(props: PropsWithChildren) {
  const config = defineFarmConfig({});
  return (
    <div className="card">
      <div className="card-content">{props.children}</div>
    </div>
  );
}
