import React, { PropsWithChildren } from 'react';
import './index.less';

export function FarmCard(props: PropsWithChildren) {
  return (
    <div className="card">
      <div className="card-content">{props.children}</div>
    </div>
  );
}
