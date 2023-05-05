import React, { PropsWithChildren } from 'react';
import './index.scss';

export function ButtonAction(props: PropsWithChildren<{ to: string }>) {
  return (
    <a className="farm-button" href={props.to} target="_blank">
      {props.children}
    </a>
  );
}
