import React, { PropsWithChildren } from 'react';
import './index.css';

export function FarmCard(props: PropsWithChildren<{ to: string }>) {
  return (
    <div className="card">
      <div className="card-content">{props.children}</div>
    </div>
  );
}
